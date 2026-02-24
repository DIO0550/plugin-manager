use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn plm() -> Command {
    Command::cargo_bin("plm").unwrap()
}

#[test]
fn test_disable_cache_not_found_shows_error_once() {
    let home = TempDir::new().unwrap();
    plm()
        .env("HOME", home.path())
        .args(["disable", "nonexistent-plugin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in cache").count(1))
        .stderr(predicate::str::contains("Hint:"));
}

#[test]
fn test_disable_operation_failure_shows_error_once() {
    let home = TempDir::new().unwrap();
    // Create cache directory with no manifest so disable_plugin fails
    let cache_dir = home.path().join(".plm/cache/plugins/github/broken-plugin");
    fs::create_dir_all(&cache_dir).unwrap();

    plm()
        .env("HOME", home.path())
        .args(["disable", "broken-plugin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to disable plugin").count(1));
}
