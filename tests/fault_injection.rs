use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_validate_corrupt_yaml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("corrupt.yaml");

    // Create a file with invalid YAML indentation/syntax
    fs::write(&file_path, "
name: MyService
  indentation_error: true
endpoints:
- path: /test
    method: GET
").unwrap();

    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
       .arg("validate")
       .arg("--file")
       .arg(file_path.to_str().unwrap());

    // Should fail with exit code 1
    // Detailed error is printed to stdout by inspect.rs
    // Top level error is printed to stderr by main.rs
    cmd.assert()
       .failure()
       .stdout(predicate::str::contains("Invalid YAML"))
       .stderr(predicate::str::contains("Validation failed"));
}

#[test]
fn test_validate_missing_required_fields() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("missing_fields.yaml");

    // Valid YAML but missing 'name'
    fs::write(&file_path, "
version: '1.0'
endpoints: []
").unwrap();

    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
       .arg("validate")
       .arg("--file")
       .arg(file_path.to_str().unwrap());

    // Should fail with exit code 1
    cmd.assert()
       .failure()
       .stdout(predicate::str::contains("Schema validation failed"))
       .stderr(predicate::str::contains("Validation failed"));
}

#[test]
fn test_import_corrupt_csv() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("bad_data.csv");
    let output_path = dir.path().join("service.yaml");

    // CSV with inconsistent columns - not valid JSON/YAML
    fs::write(&input_path, "id,name,email
1,John,john@example.com
2,Jane
3,Bob,bob@example.com,extra
").unwrap();

    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
       .arg("import")
       .arg("--file")
       .arg(input_path.to_str().unwrap())
       .arg("--output")
       .arg(output_path.to_str().unwrap());

    // Should fail
    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("Could not detect OpenAPI version")
           .or(predicate::str::contains("Invalid YAML/JSON format")));
}

#[test]
fn test_start_missing_service_file() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("new")
       .arg("myservice")
       .arg("--template")
       .arg("non_existent_template_9999");

    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("not found")
           .or(predicate::str::contains("Template error")));
}

#[test]
fn test_cli_missing_subcommand() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    // No args
    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("Missing subcommand"));
}

#[test]
fn test_cli_unknown_argument() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
       .arg("status")
       .arg("--what-is-this");

    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("Unknown argument")
           .and(predicate::str::contains("--what-is-this")));
}

#[test]
fn test_dockerize_missing_files() {
     let mut cmd = Command::cargo_bin("apicentric").unwrap();
     cmd.arg("simulator")
        .arg("dockerize")
        .arg("--services")
        .arg("non_existent.yaml");

     // Should return mapped File system error
     cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File system error")
             .and(predicate::str::contains("Failed to read service file")));
}

#[cfg(feature = "iot")]
#[test]
fn test_twin_run_missing_device() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("twin")
       .arg("run")
       .arg("non_existent_device");

    cmd.assert()
       .failure()
       .stderr(predicate::str::contains("File system error")
           .or(predicate::str::contains("Runtime error")));
}

#[test]
fn test_contract_testing_quiet() {
     // We need a dummy contract and url to test this, but minimal check:
     // If we run test without args it fails.
     // But let's check if --quiet is accepted (not "Unknown argument")
     // We can't easily mock a full test execution here without a running server.
     // But we can check help? No help doesn't show args nicely in manual parser.
     // We can run with invalid args and see if it parses --quiet.

     let mut cmd = Command::cargo_bin("apicentric").unwrap();
     cmd.arg("simulator")
        .arg("test")
        .arg("--path").arg("foo")
        .arg("--url").arg("bar")
        .arg("--quiet");

     // It will fail because file "foo" doesn't exist, but it should NOT fail with "Unknown argument --quiet"
     let assert = cmd.assert();
     assert.stderr(predicate::str::contains("Unknown argument").not());
}
