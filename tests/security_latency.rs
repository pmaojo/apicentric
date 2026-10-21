use apicentric::simulator::config::{BehaviorConfig, LatencyConfig};
use apicentric::validation::ConfigValidator;

#[test]
fn test_latency_max_ms_limit() {
    let config = BehaviorConfig {
        latency: Some(LatencyConfig {
            min_ms: 100,
            max_ms: 300_001,
        }),
        error_simulation: None,
        rate_limiting: None,
    };

    let result = config.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.field == "behavior.latency.max_ms"));
}
