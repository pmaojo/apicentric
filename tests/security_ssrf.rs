use apicentric::utils::validate_ssrf_url;
use std::env;

#[tokio::test]
async fn test_validate_ssrf_url_private_ip() {
    // 127.0.0.1 is private
    let url = "http://127.0.0.1";
    let result = validate_ssrf_url(url).await;
    assert!(result.is_err(), "Should block private IP by default");

    // Set env var to allow private IPs
    unsafe {
        env::set_var("APICENTRIC_ALLOW_PRIVATE_IPS", "true");
    }

    // It should STILL block it currently because the feature is not implemented
    let result_allowed = validate_ssrf_url(url).await;

    // This assertion confirms the fix: we can now use this validator for local dev
    // when explicitly allowed.
    assert!(
        result_allowed.is_ok(),
        "Should allow private IP when env var is set"
    );

    unsafe {
        env::remove_var("APICENTRIC_ALLOW_PRIVATE_IPS");
    }
}
