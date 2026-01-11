use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_simulator_start_output() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_apicentric"));
    cmd.arg("--dry-run")
        .arg("simulator")
        .arg("start")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dry run: Would start API simulator",
        ));
}
