//! Tests for the import and export commands.
use assert_cmd::Command;
use std::fs;
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn test_openapi_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("petstore.json");
    let service_path = temp_dir.path().join("petstore-service.yaml");
    let output_path = temp_dir.path().join("petstore-re-exported.json");

    // Create the petstore.json fixture inside the temp directory
    let petstore_content = r#"{
      "openapi": "3.0.0",
      "info": { "title": "Simple Petstore API", "version": "1.0.0" },
      "paths": {
        "/pets": {
          "get": {
            "summary": "List all pets",
            "responses": { "200": { "description": "An array of pets" } }
          },
          "post": {
            "summary": "Create a pet",
            "responses": { "201": { "description": "Null response" } }
          }
        }
      }
    }"#;
    fs::write(&input_path, petstore_content).unwrap();


    // 1. Import OpenAPI to Apicentric YAML using the unified import command
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
        .arg("import")
        .arg("--input")
        .arg(&input_path)
        .arg("--output")
        .arg(&service_path)
        .assert()
        .success();

    assert!(fs::metadata(&service_path).is_ok(), "Apicentric service file was not created.");

    // 2. Export Apicentric YAML back to OpenAPI using the unified export command
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
        .arg("export")
        .arg("--input")
        .arg(&service_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("openapi")
        .assert()
        .success();

    assert!(fs::metadata(&output_path).is_ok(), "Re-exported OpenAPI file was not created.");

    // 3. Compare the original and re-exported OpenAPI files
    let original_content = fs::read_to_string(&input_path).unwrap();
    let original_json: Value = serde_json::from_str(&original_content).unwrap();

    let exported_content = fs::read_to_string(&output_path).unwrap();
    let exported_json: Value = serde_json::from_str(&exported_content).unwrap();

    assert_eq!(original_json["info"]["title"], exported_json["info"]["title"]);
    assert_eq!(original_json["paths"]["/pets"]["get"]["summary"], exported_json["paths"]["/pets"]["get"]["summary"]);
    assert!(exported_json["paths"]["/pets"]["post"].is_object());
}
