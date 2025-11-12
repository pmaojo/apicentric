use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_simulator_start_output() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("--dry-run")
        .arg("simulator")
        .arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run: Would start API simulator"));
}

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("Apicentric is a Rust-based CLI tool")
            .and(predicate::str::contains("Usage: apicentric [OPTIONS] <COMMAND>"))
            .and(predicate::str::contains("Commands:"))
            .and(predicate::str::contains("simulator"))
            .and(predicate::str::contains("contract"))
            .and(predicate::str::contains("ai"))
            .and(predicate::str::contains("help"))
    );
}

#[test]
fn test_simulator_help_output() {
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
    cmd.arg("simulator")
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Manages the API simulator and mock services")
                .and(predicate::str::contains("Commands:"))
                .and(predicate::str::contains("start"))
                .and(predicate::str::contains("stop"))
                .and(predicate::str::contains("status"))
                .and(predicate::str::contains("validate"))
        );
}
