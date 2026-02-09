//! Security utilities.
//!
//! This module provides functions for security checks, such as SSRF prevention.

use std::net::{IpAddr, SocketAddr};
use url::Url;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF).
///
/// This function checks if the URL:
/// 1. Uses HTTP or HTTPS scheme.
/// 2. Resolves to a public IP address (blocking private, loopback, link-local, etc.).
///
/// It returns the first resolved `SocketAddr` that is safe, allowing the caller to use it directly
/// (e.g., via `reqwest::ClientBuilder::resolve`) to prevent DNS rebinding attacks.
///
/// # Arguments
///
/// * `url_str` - The URL string to validate.
///
/// # Returns
///
/// The parsed `Url` and the resolved safe `SocketAddr`.
pub async fn validate_ssrf_url(url_str: &str) -> Result<(Url, SocketAddr), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // Check scheme
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    // Check host
    let host = url.host_str().ok_or("URL missing host")?;

    // Resolve host
    // We append the port if present, or default to 80/443 depending on scheme, or just 80 for resolution purposes if we don't care about port verification beyond IP.
    // However, lookup_host needs a port.
    let port = url
        .port_or_known_default()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let address_to_resolve = format!("{}:{}", host, port);

    let addr_iter = tokio::net::lookup_host(&address_to_resolve)
        .await
        .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?;

    // Find the first safe address
    let mut safe_addr = None;
    for addr in addr_iter {
        if !is_private_ip(addr.ip()) {
            safe_addr = Some(addr);
            break;
        }
    }

    match safe_addr {
        Some(addr) => Ok((url, addr)),
        None => Err(format!(
            "Host '{}' resolves to forbidden IP(s)",
            host
        )),
    }
}

/// Checks if an IP address is private, loopback, or otherwise reserved.
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_loopback()
                || ipv4.is_private()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_unspecified()
                // Shared Address Space 100.64.0.0/10
                || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0xC0) == 0x40)
        }
        IpAddr::V6(ipv6) => {
            // Check for IPv4-mapped addresses (::ffff:a.b.c.d)
            if let Some(ipv4) = ipv6.to_ipv4_mapped() {
                return is_private_ip(IpAddr::V4(ipv4));
            }
            ipv6.is_loopback()
                || ipv6.is_unspecified()
                // Unique Local fc00::/7
                || (ipv6.segments()[0] & 0xfe00) == 0xfc00
                // Link Local fe80::/10
                || (ipv6.segments()[0] & 0xffc0) == 0xfe80
                // Documentation 2001:db8::/32
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0xdb8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ssrf_prevention() {
        // 1. Test Private IPs (IPv4)
        assert!(validate_ssrf_url("http://127.0.0.1").await.is_err(), "Should block 127.0.0.1");
        assert!(validate_ssrf_url("http://10.0.0.1").await.is_err(), "Should block 10.0.0.0/8");
        assert!(validate_ssrf_url("http://172.16.0.1").await.is_err(), "Should block 172.16.0.0/12");
        assert!(validate_ssrf_url("http://192.168.1.1").await.is_err(), "Should block 192.168.0.0/16");
        assert!(validate_ssrf_url("http://169.254.169.254").await.is_err(), "Should block Link-Local/AWS Metadata");
        assert!(validate_ssrf_url("http://100.64.0.1").await.is_err(), "Should block Shared Address Space");

        // 2. Test Private IPs (IPv6)
        assert!(validate_ssrf_url("http://[::1]").await.is_err(), "Should block IPv6 Loopback");
        assert!(validate_ssrf_url("http://[fc00::1]").await.is_err(), "Should block IPv6 Unique Local");
        assert!(validate_ssrf_url("http://[fe80::1]").await.is_err(), "Should block IPv6 Link Local");

        // 3. Test Localhost (DNS resolution)
        // This relies on the system having 'localhost' resolving to loopback, which is standard.
        let res = validate_ssrf_url("http://localhost").await;
        assert!(res.is_err(), "Should block localhost domain");

        // 4. Test Invalid Schemes
        assert!(validate_ssrf_url("ftp://example.com").await.is_err(), "Should block non-HTTP/HTTPS schemes");
        assert!(validate_ssrf_url("file:///etc/passwd").await.is_err(), "Should block file scheme");
        assert!(validate_ssrf_url("gopher://example.com").await.is_err(), "Should block gopher scheme");

        // 5. Test Valid Public IP
        // 8.8.8.8 is Google DNS, definitely public.
        let res = validate_ssrf_url("http://8.8.8.8").await;
        assert!(res.is_ok(), "Should allow public IP");
        let (_, addr) = res.unwrap();
        assert_eq!(addr.ip(), "8.8.8.8".parse::<IpAddr>().unwrap());

        // 6. Test Obfuscation / Tricks
        assert!(validate_ssrf_url("http://0.0.0.0").await.is_err(), "Should block 0.0.0.0");
        assert!(validate_ssrf_url("http://2130706433").await.is_err(), "Should block integer IP (127.0.0.1)");
    }
}
