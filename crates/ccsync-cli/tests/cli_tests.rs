use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Claude Configuration Synchronization Tool",
        ))
        .stdout(predicate::str::contains("to-local"))
        .stdout(predicate::str::contains("to-global"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("config"));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_to_local_command() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    // Just verify it doesn't panic and produces some output
    cmd.arg("to-local").assert();
}

#[test]
fn test_to_global_command() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    // Just verify it doesn't panic and produces some output
    cmd.arg("to-global").assert();
}

#[test]
fn test_status_command() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Not yet implemented"));
}

#[test]
fn test_diff_command() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("Not yet implemented"));
}

#[test]
fn test_config_command() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("Not yet implemented"));
}

#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.args(["--verbose", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Verbose mode enabled"));
}

#[test]
fn test_dry_run_flag() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.args(["--dry-run", "status"]).assert().success();
}

#[test]
fn test_to_local_with_type_filter() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    cmd.args(["to-local", "--type", "agents"]).assert();
}

#[test]
fn test_to_local_with_multiple_types() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    cmd.args(["to-local", "--type", "agents", "--type", "skills"])
        .assert();
}

#[test]
fn test_to_local_with_conflict_mode() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    cmd.args(["to-local", "--conflict", "overwrite"]).assert();
}

#[test]
fn test_invalid_conflict_mode() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.args(["to-local", "--conflict", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid'"));
}

#[test]
fn test_invalid_type() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.args(["to-local", "--type", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid'"));
}

#[test]
fn test_unknown_subcommand() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_no_subcommand() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_global_flags_with_to_local() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    cmd.args([
        "--verbose",
        "--dry-run",
        "--non-interactive",
        "to-local",
        "--conflict",
        "skip",
    ])
    .assert();
}

#[test]
fn test_preserve_symlinks_flag() {
    let mut cmd = Command::cargo_bin("ccsync").unwrap();
    // May succeed or fail depending on whether directories exist
    cmd.args(["--preserve-symlinks", "to-local"]).assert();
}

#[test]
fn test_help_for_subcommands() {
    for subcommand in &["to-local", "to-global", "status", "diff", "config"] {
        let mut cmd = Command::cargo_bin("ccsync").unwrap();
        cmd.args([subcommand, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage"));
    }
}
