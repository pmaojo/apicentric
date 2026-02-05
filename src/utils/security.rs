use std::net::IpAddr;
use url::Url;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF).
/// Blocks:
/// - Non-HTTP/HTTPS schemes
/// - Localhost/Loopback addresses
/// - Private IP ranges
/// - Link-local addresses
/// - AWS/Cloud metadata service (169.254.169.254)
pub async fn validate_ssrf_url(url_str: &str) -> Result<String, String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    let host_str = url.host_str().ok_or("URL is missing a host")?;

    if host_str.eq_ignore_ascii_case("localhost") {
         return Err("Localhost access is forbidden".to_string());
    }

    // Attempt to parse as IP first
    if let Ok(ip) = host_str.parse::<IpAddr>() {
        validate_ip(ip)?;
    } else {
        // It's a domain name, verify via DNS resolution
        // We append a port to satisfy lookup_host requirements
        let port = url.port_or_known_default().unwrap_or(80);
        let address_str = format!("{}:{}", host_str, port);

        match tokio::net::lookup_host(address_str).await {
            Ok(mut addrs) => {
                // Check all resolved addresses
                if let Some(first_addr) = addrs.next() {
                    validate_ip(first_addr.ip())?;
                    // Check rest
                    for addr in addrs {
                        validate_ip(addr.ip())?;
                    }
                } else {
                    return Err("Could not resolve host".to_string());
                }
            }
            Err(e) => return Err(format!("DNS resolution failed: {}", e)),
        }
    }

    Ok(url_str.to_string())
}

fn validate_ip(ip: IpAddr) -> Result<(), String> {
    match ip {
        IpAddr::V4(ipv4) => {
             // 127.0.0.0/8
            if ipv4.is_loopback() {
                 return Err("Loopback IP is forbidden".to_string());
            }
            // 169.254.0.0/16
            if ipv4.is_link_local() {
                 return Err("Link-local IP is forbidden".to_string());
            }
            // Private ranges
            // 10.0.0.0/8
            if ipv4.octets()[0] == 10 {
                 return Err("Private IP range 10.x.x.x is forbidden".to_string());
            }
            // 172.16.0.0/12
            if ipv4.octets()[0] == 172 && (ipv4.octets()[1] >= 16 && ipv4.octets()[1] <= 31) {
                 return Err("Private IP range 172.16.x.x is forbidden".to_string());
            }
            // 192.168.0.0/16
            if ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168 {
                 return Err("Private IP range 192.168.x.x is forbidden".to_string());
            }
             // 0.0.0.0/8 - Current network (only valid as source address)
            if ipv4.octets()[0] == 0 {
                 return Err("0.0.0.0/8 range is forbidden".to_string());
            }
        }
        IpAddr::V6(ipv6) => {
             if ipv6.is_loopback() {
                 return Err("IPv6 Loopback is forbidden".to_string());
             }
             // Unique local fc00::/7
             if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                 return Err("IPv6 Unique Local address is forbidden".to_string());
             }
             // Link local fe80::/10
              if (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                 return Err("IPv6 Link-local address is forbidden".to_string());
             }
             // :: (unspecified)
             if ipv6.is_unspecified() {
                  return Err("IPv6 Unspecified address is forbidden".to_string());
             }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_ssrf_url_valid() {
        assert!(validate_ssrf_url("https://google.com").await.is_ok());
        assert!(validate_ssrf_url("http://example.com/foo").await.is_ok());
        assert!(validate_ssrf_url("https://8.8.8.8").await.is_ok()); // Public IP
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_invalid_scheme() {
        assert!(validate_ssrf_url("ftp://example.com").await.is_err());
        assert!(validate_ssrf_url("file:///etc/passwd").await.is_err());
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_private_ips() {
        assert!(validate_ssrf_url("http://127.0.0.1").await.is_err());
        assert!(validate_ssrf_url("http://10.0.0.5").await.is_err());
        assert!(validate_ssrf_url("http://192.168.1.1").await.is_err());
        assert!(validate_ssrf_url("http://172.16.0.1").await.is_err());
        assert!(validate_ssrf_url("http://169.254.169.254").await.is_err());
        assert!(validate_ssrf_url("http://0.0.0.0").await.is_err());
        assert!(validate_ssrf_url("http://[::1]").await.is_err());
    }

    #[tokio::test]
    async fn test_validate_ssrf_url_localhost() {
        assert!(validate_ssrf_url("http://localhost").await.is_err());
        assert!(validate_ssrf_url("http://LOCALHOST").await.is_err());
    }
}
