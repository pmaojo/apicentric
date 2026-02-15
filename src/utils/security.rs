use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use url::Url;

/// Validates a URL to prevent Server-Side Request Forgery (SSRF).
///
/// This function:
/// 1. Parses the URL.
/// 2. Ensures the scheme is HTTP or HTTPS.
/// 3. Resolves the hostname to an IP address.
/// 4. Validates that the IP address is not private, loopback, or link-local.
/// 5. Returns the parsed URL and the resolved SocketAddr.
pub async fn validate_ssrf_url(url_str: &str) -> Result<(Url, SocketAddr), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS schemes are allowed".to_string());
    }

    let host = url.host_str().ok_or("URL must have a host")?.to_string();
    let port = url.port_or_known_default().unwrap_or(80);

    // Resolve DNS
    // Note: We use tokio's lookup_host which performs async DNS resolution
    let addrs = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?;

    // Check all resolved addresses
    for addr in addrs {
        let ip = addr.ip();
        if is_global(ip) {
            // Found a valid public IP
            return Ok((url, addr));
        }
    }

    Err(format!(
        "Host '{}' resolves to a private, loopback, or invalid IP address",
        host
    ))
}

/// Sanitizes a string for CSV export to prevent Formula Injection (CSV Injection).
///
/// This function:
/// 1. Prepends a single quote (') if the field starts with =, +, -, or @ to prevent formula execution.
/// 2. Wraps the field in double quotes if it contains commas, newlines, or double quotes.
/// 3. Escapes internal double quotes by doubling them ("").
///
/// Returns `Cow::Borrowed` if no changes are needed, otherwise `Cow::Owned`.
pub fn sanitize_csv_field(field: &str) -> Cow<'_, str> {
    let needs_quote_prefix = field.starts_with(['=', '+', '-', '@']);
    let needs_wrapping = field.contains([',', '"', '\n', '\r']);

    if !needs_quote_prefix && !needs_wrapping {
        return Cow::Borrowed(field);
    }

    let mut value = String::with_capacity(field.len() + 3);

    if needs_wrapping {
        value.push('"');
    }

    if needs_quote_prefix {
        value.push('\'');
    }

    if needs_wrapping {
        // Escape internal quotes
        value.push_str(&field.replace('"', "\"\""));
        value.push('"');
    } else {
        value.push_str(field);
    }

    Cow::Owned(value)
}

/// Checks if an IP address is globally reachable (public).
///
/// Returns `true` if the IP is public.
/// Returns `false` if the IP is private, loopback, link-local, broadcast, documentation, or unspecified.
fn is_global(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Check if loopback (127.0.0.0/8)
            if ipv4.is_loopback() {
                return false;
            }

            // Check if link-local (169.254.0.0/16)
            if ipv4.is_link_local() {
                return false;
            }

            // Check if private
            // 10.0.0.0/8
            if ipv4.octets()[0] == 10 {
                return false;
            }
            // 172.16.0.0/12
            if ipv4.octets()[0] == 172 && (ipv4.octets()[1] >= 16 && ipv4.octets()[1] <= 31) {
                return false;
            }
            // 192.168.0.0/16
            if ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168 {
                return false;
            }

            // Check for documentation addresses
            // 192.0.2.0/24 (TEST-NET-1)
            if ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2 {
                return false;
            }
            // 198.51.100.0/24 (TEST-NET-2)
            if ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100 {
                return false;
            }
            // 203.0.113.0/24 (TEST-NET-3)
            if ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113 {
                return false;
            }

            // Check for broadcast (255.255.255.255)
            if ipv4.is_broadcast() {
                return false;
            }

            // Check for Current Network (0.0.0.0/8) - includes Unspecified (0.0.0.0)
            if ipv4.octets()[0] == 0 {
                return false;
            }

            // Check for Shared Address Space (100.64.0.0/10)
            if ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0b1100_0000 == 0b0100_0000) {
                return false;
            }

            true
        }
        IpAddr::V6(ipv6) => {
            // Check if loopback (::1)
            if ipv6.is_loopback() {
                return false;
            }

            // Check if unspecified (::)
            if ipv6.is_unspecified() {
                return false;
            }

            // Check for IPv4-mapped address (::ffff:a.b.c.d) or IPv4-compatible address (::a.b.c.d)
            if let Some(ipv4) = ipv6.to_ipv4() {
                return is_global(IpAddr::V4(ipv4));
            }

            // Check if unique local (fc00::/7)
            // ipv6.is_unique_local() is unstable, so we check manually
            // fc00::/7 starts with 1111 110x -> fc or fd
            if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                return false;
            }

            // Check if link-local unicast (fe80::/10)
            // ipv6.is_unicast_link_local() is unstable, so we check manually
            // fe80::/10 starts with 1111 1110 10 -> fe8, fe9, fea, feb
            if (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                return false;
            }

            // Check for documentation addresses (2001:db8::/32)
            if ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0xdb8 {
                return false;
            }

            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_global_ipv4() {
        // Public IPs
        assert!(is_global("8.8.8.8".parse().unwrap()));
        assert!(is_global("1.1.1.1".parse().unwrap()));
        assert!(is_global("142.250.190.46".parse().unwrap())); // google.com

        // Private IPs
        assert!(!is_global("10.0.0.1".parse().unwrap()));
        assert!(!is_global("172.16.0.1".parse().unwrap()));
        assert!(!is_global("172.31.255.255".parse().unwrap()));
        assert!(!is_global("192.168.0.1".parse().unwrap()));

        // Loopback
        assert!(!is_global("127.0.0.1".parse().unwrap()));

        // Link-local
        assert!(!is_global("169.254.0.1".parse().unwrap()));

        // Documentation
        assert!(!is_global("192.0.2.1".parse().unwrap()));
        assert!(!is_global("198.51.100.1".parse().unwrap()));
        assert!(!is_global("203.0.113.1".parse().unwrap()));

        // Broadcast
        assert!(!is_global("255.255.255.255".parse().unwrap()));

        // Unspecified
        assert!(!is_global("0.0.0.0".parse().unwrap()));

        // Current Network (0.0.0.0/8)
        assert!(!is_global("0.0.0.1".parse().unwrap()));

        // Shared Address Space
        assert!(!is_global("100.64.0.1".parse().unwrap()));
    }

    #[test]
    fn test_is_global_ipv6() {
        // Public IPs
        assert!(is_global("2001:4860:4860::8888".parse().unwrap())); // google dns

        // Loopback
        assert!(!is_global("::1".parse().unwrap()));

        // Unspecified
        assert!(!is_global("::".parse().unwrap()));

        // Unique Local
        assert!(!is_global("fc00::1".parse().unwrap()));
        assert!(!is_global("fd00::1".parse().unwrap()));

        // Link-local
        assert!(!is_global("fe80::1".parse().unwrap()));

        // Documentation
        assert!(!is_global("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn test_is_global_ipv4_mapped_ipv6() {
        // Mapped Loopback (::ffff:127.0.0.1)
        assert!(!is_global("::ffff:127.0.0.1".parse().unwrap()));

        // Mapped Private (::ffff:192.168.1.1)
        assert!(!is_global("::ffff:192.168.1.1".parse().unwrap()));

        // Mapped Link-Local (::ffff:169.254.1.1)
        assert!(!is_global("::ffff:169.254.1.1".parse().unwrap()));

        // Compatible Loopback (::127.0.0.1) - Deprecated but should be blocked
        assert!(!is_global("::127.0.0.1".parse().unwrap()));

        // Mapped Public (::ffff:8.8.8.8) - Should be allowed
        assert!(is_global("::ffff:8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn test_sanitize_csv_field() {
        // Normal text
        assert_eq!(sanitize_csv_field("hello"), "hello");
        assert_eq!(sanitize_csv_field("123"), "123");

        // Text with commas
        assert_eq!(sanitize_csv_field("hello,world"), "\"hello,world\"");

        // Text with quotes
        assert_eq!(sanitize_csv_field("hello\"world"), "\"hello\"\"world\"");

        // Text with newlines
        assert_eq!(sanitize_csv_field("hello\nworld"), "\"hello\nworld\"");

        // CSV Injection attempts
        assert_eq!(sanitize_csv_field("=cmd"), "'=cmd");
        assert_eq!(sanitize_csv_field("+1+1"), "'+1+1");
        assert_eq!(sanitize_csv_field("-1-1"), "'-1-1");
        assert_eq!(sanitize_csv_field("@echo"), "'@echo");

        // Injection with special chars (should be quoted AND escaped)
        // =cmd,args -> '=cmd,args -> "'=cmd,args"
        assert_eq!(sanitize_csv_field("=cmd,args"), "\"'=cmd,args\"");
    }

    #[test]
    fn test_sanitize_csv_field_optimization() {
        // Should be borrowed (no allocation)
        if let Cow::Owned(_) = sanitize_csv_field("simple_string") {
            panic!("Should be borrowed");
        }

        // Should be owned (allocation needed due to wrapping)
        if let Cow::Borrowed(_) = sanitize_csv_field("complex,string") {
            panic!("Should be owned (wrapping)");
        }

        // Should be owned (allocation needed due to prefix)
        if let Cow::Borrowed(_) = sanitize_csv_field("=injection") {
            panic!("Should be owned (prefix)");
        }
    }
}
