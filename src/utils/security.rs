use std::net::{Ipv4Addr, Ipv6Addr};
use url::Url;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF).
///
/// This function checks if the URL points to a forbidden destination, such as:
/// - Loopback addresses (e.g., 127.0.0.1, ::1)
/// - Private network addresses (e.g., 10.x.x.x, 192.168.x.x)
/// - Link-local addresses (e.g., 169.254.x.x)
/// - The "localhost" hostname
///
/// Note: This performs a basic check on the URL string and does not prevent DNS rebinding attacks
/// unless the HTTP client is also configured to resolve DNS safely.
pub fn validate_ssrf_url(url_str: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // specific check for localhost before any parsing, as url crate might treat it differently depending on context
    if let Some(host_str) = url.host_str() {
        if host_str.eq_ignore_ascii_case("localhost") {
            return Err("Access to localhost is forbidden".to_string());
        }
    }

    match url.host() {
        Some(url::Host::Domain(domain)) => {
            if domain.eq_ignore_ascii_case("localhost") {
                return Err("Access to localhost is forbidden".to_string());
            }
            // We cannot resolve DNS here reliably without potential TOCTOU issues.
            // A comprehensive solution requires a custom DNS resolver in the HTTP client.
            // For now, we allow domains (assuming they point to public IPs) but block obvious local ones.
        }
        Some(url::Host::Ipv4(addr)) => {
            if is_forbidden_ipv4(addr) {
                return Err(format!("Access to IP address {} is forbidden", addr));
            }
        }
        Some(url::Host::Ipv6(addr)) => {
            if is_forbidden_ipv6(addr) {
                return Err(format!("Access to IP address {} is forbidden", addr));
            }
        }
        None => return Err("URL has no host".to_string()),
    }

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    Ok(())
}

fn is_forbidden_ipv4(addr: Ipv4Addr) -> bool {
    let octets = addr.octets();

    // Loopback: 127.0.0.0/8
    if octets[0] == 127 {
        return true;
    }

    // Link-local: 169.254.0.0/16
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }

    // Private Class A: 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }

    // Private Class B: 172.16.0.0/12
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return true;
    }

    // Private Class C: 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }

    // Broadcast: 255.255.255.255
    if addr.is_broadcast() {
        return true;
    }

    // Unspecified: 0.0.0.0
    if addr.is_unspecified() {
        return true;
    }

    false
}

fn is_forbidden_ipv6(addr: Ipv6Addr) -> bool {
    // Loopback: ::1
    if addr.is_loopback() {
        return true;
    }

    // Unspecified: ::
    if addr.is_unspecified() {
        return true;
    }

    // Unique Local: fc00::/7
    // fc00 to fdff
    let segments = addr.segments();
    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }

    // Link-local: fe80::/10
    if (segments[0] & 0xffc0) == 0xfe80 {
        return true;
    }

    // IPv4-mapped IPv6 addresses (::ffff:a.b.c.d)
    if let Some(ipv4) = addr.to_ipv4() {
        return is_forbidden_ipv4(ipv4);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_public_urls() {
        assert!(validate_ssrf_url("https://google.com").is_ok());
        assert!(validate_ssrf_url("http://example.com/api").is_ok());
        assert!(validate_ssrf_url("https://8.8.8.8").is_ok());
    }

    #[test]
    fn test_localhost_blocked() {
        assert!(validate_ssrf_url("http://localhost").is_err());
        assert!(validate_ssrf_url("http://localhost:8080").is_err());
        assert!(validate_ssrf_url("http://127.0.0.1").is_err());
        assert!(validate_ssrf_url("http://127.0.0.1:3000").is_err());
        assert!(validate_ssrf_url("http://[::1]").is_err());
    }

    #[test]
    fn test_private_ips_blocked() {
        // Class A
        assert!(validate_ssrf_url("http://10.0.0.1").is_err());
        // Class B
        assert!(validate_ssrf_url("http://172.16.0.1").is_err());
        assert!(validate_ssrf_url("http://172.31.255.255").is_err());
        // Valid public IP near Class B range
        assert!(validate_ssrf_url("http://172.32.0.1").is_ok());

        // Class C
        assert!(validate_ssrf_url("http://192.168.1.1").is_err());
    }

    #[test]
    fn test_link_local_blocked() {
        // AWS Metadata service
        assert!(validate_ssrf_url("http://169.254.169.254/latest/meta-data/").is_err());
    }

    #[test]
    fn test_schemes() {
        assert!(validate_ssrf_url("ftp://example.com").is_err());
        assert!(validate_ssrf_url("file:///etc/passwd").is_err());
    }
}
