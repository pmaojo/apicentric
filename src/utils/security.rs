use std::net::IpAddr;
use url::Url;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF).
///
/// This function:
/// 1. Parses the URL.
/// 2. Ensures the scheme is HTTP or HTTPS.
/// 3. Resolves the hostname to IP addresses.
/// 4. Checks if any resolved IP is a private, loopback, or link-local address.
///
/// # Arguments
///
/// * `url_str` - The URL string to validate.
///
/// # Returns
///
/// * `Ok(())` if the URL is safe.
/// * `Err(String)` if the URL is invalid or points to a forbidden destination.
pub async fn validate_ssrf_url(url_str: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    if let Some(host) = url.host_str() {
        // Check for localhost/127.0.0.1 immediately if it's an IP literal or known localhost string
        if host == "localhost" {
            return Err("Access to localhost is forbidden".to_string());
        }

        // We need to add a port if missing to use lookup_host, but we only care about the IP
        let port = url.port_or_known_default().unwrap_or(80);
        let addr_str = format!("{}:{}", host, port);

        match tokio::net::lookup_host(addr_str).await {
            Ok(addrs) => {
                let mut found = false;
                for addr in addrs {
                    found = true;
                    let ip = addr.ip();
                    if is_private_ip(ip) {
                        return Err(format!("Access to private IP {} is forbidden", ip));
                    }
                }
                if !found {
                    return Err(format!("Could not resolve host: {}", host));
                }
            }
            Err(e) => return Err(format!("Failed to resolve host: {}", e)),
        }
    } else {
        return Err("URL is missing host".to_string());
    }

    Ok(())
}

/// Checks if an IP address is private, loopback, or link-local.
///
/// # Arguments
///
/// * `ip` - The IP address to check.
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            // 127.0.0.0/8 - Loopback
            if ip.is_loopback() {
                return true;
            }
            // 169.254.0.0/16 - Link Local
            if ip.is_link_local() {
                return true;
            }
            // 0.0.0.0/8 - Current network (unspecified)
            if ip.is_unspecified() {
                return true;
            }

            // Private ranges (RFC 1918)
            let octets = ip.octets();

            // 10.0.0.0/8
            if octets[0] == 10 {
                return true;
            }
            // 172.16.0.0/12
            if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) {
                return true;
            }
            // 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }

            false
        }
        IpAddr::V6(ip) => {
            // ::1 - Loopback
            if ip.is_loopback() {
                return true;
            }
            // :: - Unspecified
            if ip.is_unspecified() {
                return true;
            }

            // fe80::/10 - Link Local
            // segments()[0] is u16. fe80 is 0xfe80. /10 mask is 0xffc0.
            // 0xfe80 & 0xffc0 = 0xfe80.
            if (ip.segments()[0] & 0xffc0) == 0xfe80 {
                return true;
            }

            // fc00::/7 - Unique Local
            // 0xfc00 & 0xfe00 = 0xfc00
            if (ip.segments()[0] & 0xfe00) == 0xfc00 {
                return true;
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_is_private_ip_v4() {
        // Loopback
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        // Private 10.x
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));

        // Private 172.16-31.x
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))));
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 15, 0, 1)))); // Public
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 32, 0, 1)))); // Public

        // Private 192.168.x
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));

        // Link Local
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 0, 1))));

        // Public
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn test_is_private_ip_v6() {
        // Loopback
        assert!(is_private_ip(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 1
        ))));

        // Unspecified
        assert!(is_private_ip(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 0
        ))));

        // Unique Local (fc00::/7)
        // fd00... is valid unique local
        assert!(is_private_ip(IpAddr::V6(
            "fd12:3456:789a:1::1".parse().unwrap()
        )));

        // Link Local (fe80::/10)
        assert!(is_private_ip(IpAddr::V6("fe80::1".parse().unwrap())));

        // Public
        assert!(!is_private_ip(IpAddr::V6(
            "2001:4860:4860::8888".parse().unwrap()
        )));
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_blocks_localhost() {
        let res = validate_ssrf_url("http://localhost:8080/foo").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("localhost is forbidden"));
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_blocks_loopback_ip() {
        let res = validate_ssrf_url("http://127.0.0.1:8080/foo").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("private IP"));
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_blocks_private_ip() {
        let res = validate_ssrf_url("http://192.168.1.50/foo").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_blocks_non_http() {
        let res = validate_ssrf_url("ftp://example.com/file").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Only HTTP and HTTPS"));
    }
}
