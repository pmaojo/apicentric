use std::net::IpAddr;
use url::{Url, Host};

/// Validates that a URL is safe to fetch (SSRF protection).
/// Blocks local, private, and link-local addresses.
///
/// Note: This validation is based on the input URL string and performs
/// a best-effort check. It does not prevent DNS rebinding attacks where
/// a public domain resolves to a private IP address.
pub fn validate_ssrf_url(url_str: &str) -> Result<Url, String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("URL must use http or https scheme".to_string());
    }

    match url.host() {
        Some(Host::Domain(domain)) => {
            if domain == "localhost" {
                return Err("Access to localhost is forbidden".to_string());
            }
        }
        Some(Host::Ipv4(ipv4)) => {
            if is_restricted_ip(&IpAddr::V4(ipv4)) {
                 return Err(format!("Access to restricted IP {} is forbidden", ipv4));
            }
        }
        Some(Host::Ipv6(ipv6)) => {
             if is_restricted_ip(&IpAddr::V6(ipv6)) {
                 return Err(format!("Access to restricted IP {} is forbidden", ipv6));
            }
        }
        None => return Err("URL has no host".to_string()),
    }

    Ok(url)
}

fn is_restricted_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_loopback() ||
            ipv4.is_private() ||
            ipv4.is_link_local() ||
            ipv4.is_broadcast() ||
            ipv4.is_documentation() ||
            ipv4.octets() == [0, 0, 0, 0]
        },
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback() ||
            ipv6.is_multicast() ||
            ipv6.is_unicast_link_local() ||
            (ipv6.segments()[0] & 0xfe00) == 0xfc00 // Unique local (fc00::/7)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_ssrf_url("https://google.com").is_ok());
        assert!(validate_ssrf_url("http://example.com/api/v1").is_ok());
        assert!(validate_ssrf_url("https://8.8.8.8").is_ok());
    }

    #[test]
    fn test_invalid_schemes() {
        assert!(validate_ssrf_url("ftp://example.com").is_err());
        assert!(validate_ssrf_url("file:///etc/passwd").is_err());
        assert!(validate_ssrf_url("gopher://localhost").is_err());
    }

    #[test]
    fn test_restricted_hosts() {
        assert!(validate_ssrf_url("http://localhost").is_err());
        assert!(validate_ssrf_url("http://localhost:8080").is_err());
        assert!(validate_ssrf_url("http://127.0.0.1").is_err());
        assert!(validate_ssrf_url("http://10.0.0.1").is_err());
        assert!(validate_ssrf_url("http://192.168.1.1").is_err());
        assert!(validate_ssrf_url("http://169.254.169.254").is_err());
        assert!(validate_ssrf_url("http://0.0.0.0").is_err());
        assert!(validate_ssrf_url("http://[::1]").is_err());
        assert!(validate_ssrf_url("http://[fe80::1]").is_err());
    }
}
