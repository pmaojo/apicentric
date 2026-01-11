use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dockerize_command() {
    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    let service_def_path = input_dir.path().join("test-service.yaml");
    fs::write(
        &service_def_path,
        "name: test-service\nserver:\n  port: 8080",
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_apicentric"));
    cmd.arg("simulator")
        .arg("dockerize")
        .arg("--services")
        .arg(service_def_path.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dockerizing services"));

    assert!(output_dir.path().join("Dockerfile").exists());
    assert!(output_dir.path().join(".dockerignore").exists());
    assert!(output_dir
        .path()
        .join("services/test-service.yaml")
        .exists());

    let dockerfile_content = fs::read_to_string(output_dir.path().join("Dockerfile")).unwrap();
    assert!(dockerfile_content.contains("cargo install apicentric"));
    assert!(dockerfile_content.contains("EXPOSE 8080"));
}
