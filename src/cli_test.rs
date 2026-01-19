//! CLI help output integration tests

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_root_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Plugin Manager CLI"));
}

#[test]
fn test_target_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["target", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage target environments"));
}

#[test]
fn test_marketplace_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["marketplace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage plugin marketplaces"));
}

#[test]
fn test_install_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SOURCE FORMATS"));
}

#[test]
fn test_list_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OUTPUT FORMATS"));
}

#[test]
fn test_info_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SECTIONS DISPLAYED"));
}

#[test]
fn test_enable_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["enable", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enable a plugin"));
}

#[test]
fn test_disable_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["disable", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Disable a plugin"));
}

#[test]
fn test_uninstall_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["uninstall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completely remove"));
}

#[test]
fn test_update_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check for and apply updates"));
}

#[test]
fn test_init_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate plugin templates"));
}

#[test]
fn test_sync_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Synchronize plugins"));
}

#[test]
fn test_import_help() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Import existing Claude Code"));
}
