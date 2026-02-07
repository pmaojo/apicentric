use std::net::IpAddr;
use url::Url;

/// Validates that a URL is safe to fetch (SSRF protection).
/// Blocks requests to private, loopback, and link-local addresses.
pub async fn validate_ssrf_url(url_str: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    // Only allow HTTP and HTTPS
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    let host = url.host_str().ok_or("URL is missing a host")?;

    // Resolve hostname to IP addresses
    let port = url.port_or_known_default().unwrap_or(80);
    let addr_str = format!("{}:{}", host, port);

    // Use tokio::net::lookup_host to resolve DNS asynchronously
    let addrs = tokio::net::lookup_host(&addr_str)
        .await
        .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?;

    for addr in addrs {
        let ip = addr.ip();
        if is_private_ip(&ip) {
            return Err(format!("Access to private IP '{}' is forbidden", ip));
        }
    }

    Ok(())
}

/// Checks if an IP address is private, loopback, or link-local.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 127.0.0.0/8 (Loopback)
            if octets[0] == 127 { return true; }
            // 10.0.0.0/8 (Private)
            if octets[0] == 10 { return true; }
            // 172.16.0.0/12 (Private)
            if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 { return true; }
            // 192.168.0.0/16 (Private)
            if octets[0] == 192 && octets[1] == 168 { return true; }
            // 169.254.0.0/16 (Link-local)
            if octets[0] == 169 && octets[1] == 254 { return true; }
            // 0.0.0.0/8 (Current network)
            if octets[0] == 0 { return true; }
            // 100.64.0.0/10 (Carrier-grade NAT) - arguably private but routed on internet sometimes.
            // Safe to block for general SSRF.
            if octets[0] == 100 && (octets[1] & 0xC0) == 0x40 { return true; }
            false
        }
        IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            // ::1 (Loopback)
            if ipv6.is_loopback() { return true; }
            // fc00::/7 (Unique Local)
            if (segments[0] & 0xfe00) == 0xfc00 { return true; }
            // fe80::/10 (Link-local)
            if (segments[0] & 0xffc0) == 0xfe80 { return true; }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_ssrf_url() {
        // Public IPs (Google DNS)
        assert!(validate_ssrf_url("http://8.8.8.8").await.is_ok());
        assert!(validate_ssrf_url("https://1.1.1.1").await.is_ok());

        // Private IPs
        assert!(validate_ssrf_url("http://127.0.0.1").await.is_err());
        assert!(validate_ssrf_url("http://localhost").await.is_err());
        assert!(validate_ssrf_url("http://10.0.0.1").await.is_err());
        assert!(validate_ssrf_url("http://192.168.1.1").await.is_err());
        assert!(validate_ssrf_url("http://169.254.169.254").await.is_err()); // Metadata service

        // Invalid schemes
        assert!(validate_ssrf_url("ftp://example.com").await.is_err());
        assert!(validate_ssrf_url("file:///etc/passwd").await.is_err());
    }
}
