use apicentric::simulator::config::{BehaviorConfig, LatencyConfig};
use apicentric::validation::ConfigValidator;

#[test]
fn test_latency_validation_dos() {
    let config = BehaviorConfig {
        latency: Some(LatencyConfig {
            min_ms: 1000,
            max_ms: 999_999_999, // Extremely high value
        }),
        error_simulation: None,
        rate_limiting: None,
    };

    // This should now fail
    let result = config.validate();
    assert!(result.is_err(), "Validation should reject extremely high latency");

    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.message.contains("too high")), "Error message should mention latency is too high");
}
