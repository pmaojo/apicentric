use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_simulator_start_output() {
<<<<<<< HEAD
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_apicentric"));
=======
    let mut cmd = Command::cargo_bin("apicentric").unwrap();
>>>>>>> origin/main
    cmd.arg("--dry-run")
        .arg("simulator")
        .arg("start")
        .assert()
        .success()
<<<<<<< HEAD
        .stdout(predicate::str::contains(
            "Dry run: Would start API simulator",
        ));
}
=======
        .stdout(predicate::str::contains("Dry run: Would start API simulator"));
}
>>>>>>> origin/main
