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
        .stdout(predicate::str::contains(
            "Import components from a Claude Code Plugin",
        ));
}

#[test]
fn test_install_scope_help_mentions_tui_selection() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Deployment scope (if not specified, TUI selection)",
        ));
}

#[test]
fn test_import_scope_help_mentions_tui_selection() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Deployment scope (if not specified, TUI selection)",
        ));
}

#[test]
fn test_sync_scope_help_mentions_both_default() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Scope to sync (if not specified, both personal and project)",
        ));
}

#[test]
fn test_list_json_conflicts_with_simple() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["list", "--json", "--simple"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--json' cannot be used with '--simple'",
        ));
}

#[test]
fn test_list_outdated_conflicts_with_simple() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["list", "--outdated", "--simple"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--outdated' cannot be used with '--simple'",
        ));
}

#[test]
fn test_list_outdated_allows_json() {
    Command::cargo_bin("plm")
        .unwrap()
        .args(["list", "--outdated", "--json"])
        .assert()
        .success();
}
