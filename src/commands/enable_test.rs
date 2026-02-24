use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn plm() -> Command {
    Command::cargo_bin("plm").unwrap()
}

#[test]
fn test_enable_cache_not_found_shows_error_once() {
    let home = TempDir::new().unwrap();
    plm()
        .env("HOME", home.path())
        .args(["enable", "nonexistent-plugin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in cache").count(1));
}

#[test]
fn test_enable_operation_failure_shows_error_once() {
    let home = TempDir::new().unwrap();
    // Create cache directory with no manifest so enable_plugin fails
    let cache_dir = home.path().join(".plm/cache/plugins/github/broken-plugin");
    fs::create_dir_all(&cache_dir).unwrap();

    plm()
        .env("HOME", home.path())
        .args(["enable", "broken-plugin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to enable plugin").count(1));
}
