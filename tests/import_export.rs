//! Tests for the import and export commands.
use assert_cmd::Command;
use std::fs;
use serde_json::Value;

#[test]
fn test_openapi_round_trip() {
    let input_path = "tests/fixtures/petstore.json";
    let service_path = "tests/fixtures/petstore-service.yaml";
    let output_path = "tests/fixtures/petstore-re-exported.json";

    // 1. Import OpenAPI to Apicentric YAML
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
        .arg("import-openapi")
        .arg("--input")
        .arg(input_path)
        .arg("--output")
        .arg(service_path)
        .assert()
        .success();

    assert!(fs::metadata(service_path).is_ok(), "Apicentric service file was not created.");

    // 2. Export Apicentric YAML back to OpenAPI
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
        .arg("export-openapi")
        .arg("--input")
        .arg(service_path)
        .arg("--output")
        .arg(output_path)
        .assert()
        .success();

    assert!(fs::metadata(output_path).is_ok(), "Re-exported OpenAPI file was not created.");

    // 3. Compare the original and re-exported OpenAPI files
    let original_content = fs::read_to_string(input_path).unwrap();
    let original_json: Value = serde_json::from_str(&original_content).unwrap();

    let exported_content = fs::read_to_string(output_path).unwrap();
    let exported_json: Value = serde_json::from_str(&exported_content).unwrap();

    assert_eq!(original_json["info"]["title"], exported_json["info"]["title"]);
    assert_eq!(original_json["paths"]["/pets"]["get"]["summary"], exported_json["paths"]["/pets"]["get"]["summary"]);
    assert!(exported_json["paths"]["/pets"]["post"].is_object());

    // Clean up generated files
    fs::remove_file(service_path).unwrap();
    fs::remove_file(output_path).unwrap();
}
