use url::Url;
use std::net::IpAddr;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF) attacks.
///
/// This function enforces the following security rules:
/// 1. Only HTTP and HTTPS schemes are allowed.
/// 2. Access to "localhost" is blocked.
/// 3. Access to Loopback addresses (127.0.0.0/8, ::1) is blocked.
/// 4. Access to Link-local addresses (169.254.0.0/16, fe80::/10) is blocked.
/// 5. Access to Private IP ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, fc00::/7) is blocked.
/// 6. Access to Unspecified addresses (0.0.0.0, ::) is blocked.
/// 7. Access to IPv4-mapped IPv6 addresses resolving to restricted IPs is blocked.
pub fn validate_ssrf_url(input_url: &str) -> Result<Url, String> {
    let url = Url::parse(input_url).map_err(|e| format!("Invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    match url.host() {
        Some(url::Host::Domain(domain)) => {
            if domain.eq_ignore_ascii_case("localhost") {
                return Err("Localhost access is restricted".to_string());
            }
        }
        Some(url::Host::Ipv4(ip)) => {
            if is_restricted_ip(IpAddr::V4(ip)) {
                return Err(format!("Restricted IP address: {}", ip));
            }
        }
        Some(url::Host::Ipv6(ip)) => {
            if is_restricted_ip(IpAddr::V6(ip)) {
                return Err(format!("Restricted IP address: {}", ip));
            }
        }
        None => return Err("URL must have a host".to_string()),
    }

    Ok(url)
}

fn is_restricted_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            if ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_private()
                || ipv4.is_unspecified()
                || ipv4.is_broadcast()
            {
                return true;
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() || ipv6.is_unspecified() {
                return true;
            }

            // Check if it's an IPv4-mapped address (::ffff:1.2.3.4) or compatible (::1.2.3.4)
            if let Some(ipv4) = ipv6.to_ipv4() {
                return is_restricted_ip(IpAddr::V4(ipv4));
            }

            let segments = ipv6.segments();

            // Unique local (private): fc00::/7
            if (segments[0] & 0xfe00) == 0xfc00 {
                return true;
            }

            // Link-local: fe80::/10
            if (segments[0] & 0xffc0) == 0xfe80 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_public_url() {
        assert!(validate_ssrf_url("https://example.com/api.yaml").is_ok());
        assert!(validate_ssrf_url("http://google.com").is_ok());
        assert!(validate_ssrf_url("https://8.8.8.8/config").is_ok());
    }

    #[test]
    fn test_block_localhost() {
        assert!(validate_ssrf_url("http://localhost:8080").is_err());
        assert!(validate_ssrf_url("https://LOCALHOST/").is_err());
    }

    #[test]
    fn test_block_loopback_ip() {
        assert!(validate_ssrf_url("http://127.0.0.1/").is_err());
        assert!(validate_ssrf_url("http://127.0.0.1:9000/").is_err());
        assert!(validate_ssrf_url("http://[::1]/").is_err());
    }

    #[test]
    fn test_block_private_ip() {
        assert!(validate_ssrf_url("http://192.168.1.1/").is_err());
        assert!(validate_ssrf_url("http://10.0.0.5/").is_err());
        assert!(validate_ssrf_url("http://172.16.0.1/").is_err());
        assert!(validate_ssrf_url("http://172.31.255.255/").is_err());
        // IPv6 Unique Local
        assert!(validate_ssrf_url("http://[fd00::1]/").is_err());
    }

    #[test]
    fn test_block_link_local() {
        assert!(validate_ssrf_url("http://169.254.169.254/latest/meta-data/").is_err());
        // IPv6 Link Local
        assert!(validate_ssrf_url("http://[fe80::1]/").is_err());
    }

    #[test]
    fn test_block_ipv4_mapped() {
        // IPv4-mapped 127.0.0.1 -> ::ffff:127.0.0.1
        assert!(validate_ssrf_url("http://[::ffff:127.0.0.1]/").is_err());
        // IPv4-mapped 192.168.1.1 -> ::ffff:192.168.1.1
        assert!(validate_ssrf_url("http://[::ffff:c0a8:0101]/").is_err());
    }

    #[test]
    fn test_block_non_http() {
        assert!(validate_ssrf_url("ftp://example.com/file").is_err());
        assert!(validate_ssrf_url("file:///etc/passwd").is_err());
    }
}
