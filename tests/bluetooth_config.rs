#[cfg(feature = "bluetooth")]
#[tokio::test]
async fn test_bluetooth_config_parsing() {
    use apicentric::simulator::config::{UnifiedConfig, ServiceDefinition};

    let yaml = r#"
name: HeartRateMonitor
bluetooth:
  local_name: "HR Monitor 3000"
  services:
    - uuid: "180D"
      primary: true
      characteristics:
        - uuid: "2A37"
          name: "Heart Rate Measurement"
          properties: ["notify"]
          value: "00"
"#;

    let unified: UnifiedConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    let def = ServiceDefinition::from(unified);

    assert!(def.bluetooth.is_some());
    let bt = def.bluetooth.unwrap();
    assert_eq!(bt.local_name, "HR Monitor 3000");
    assert_eq!(bt.services.len(), 1);
    assert_eq!(bt.services[0].uuid, "180D");
    assert_eq!(bt.services[0].characteristics.len(), 1);
    assert_eq!(bt.services[0].characteristics[0].properties[0], apicentric::simulator::config::bluetooth::CharacteristicProperty::Notify);
}
