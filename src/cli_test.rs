//! CLI help output integration tests

use super::{Cli, Command as CliCommand};
use assert_cmd::prelude::*;
use clap::Parser;
use predicates::prelude::*;
use std::process::Command as ProcessCommand;

#[test]
fn test_root_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Plugin Manager CLI"));
}

#[test]
fn test_target_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["target", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage target environments"));
}

#[test]
fn test_marketplace_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["marketplace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage plugin marketplaces"));
}

#[test]
fn test_install_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SOURCE FORMATS"));
}

#[test]
fn test_list_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OUTPUT FORMATS"));
}

#[test]
fn test_info_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["info", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SECTIONS DISPLAYED"));
}

#[test]
fn test_enable_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["enable", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Enable a plugin"));
}

#[test]
fn test_disable_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["disable", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Disable a plugin"));
}

#[test]
fn test_uninstall_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["uninstall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completely remove"));
}

#[test]
fn test_update_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check for and apply updates"));
}

#[test]
fn test_init_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate plugin templates"));
}

#[test]
fn test_sync_help() {
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Synchronize plugins"));
}

#[test]
fn test_import_help() {
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
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
    ProcessCommand::cargo_bin("plm")
        .unwrap()
        .args(["list", "--outdated", "--json"])
        .assert()
        .success();
}

#[test]
fn cli_no_args_yields_command_none() {
    let cli = Cli::try_parse_from(["plm"]).expect("plm 単体起動はパース成功する");
    assert!(cli.command.is_none());
    assert!(!cli.verbose);
}

#[test]
fn cli_verbose_only_yields_command_none() {
    let cli =
        Cli::try_parse_from(["plm", "--verbose"]).expect("plm --verbose 単体起動はパース成功する");
    assert!(cli.command.is_none());
    assert!(cli.verbose);
}

#[test]
fn cli_managed_explicit_yields_command_managed() {
    let cli = Cli::try_parse_from(["plm", "managed"]).expect("plm managed はパース成功する");
    assert!(matches!(cli.command, Some(CliCommand::Managed)));
}

#[test]
fn cli_install_subcommand_still_parses() {
    let cli = Cli::try_parse_from(["plm", "install", "owner/repo"])
        .expect("plm install owner/repo はパース成功する");
    assert!(matches!(cli.command, Some(CliCommand::Install(_))));
}

#[test]
fn cli_unknown_flag_still_errors() {
    let result = Cli::try_parse_from(["plm", "--no-such-flag"]);
    assert!(result.is_err(), "未知フラグは従来通り clap エラー");
}

#[test]
fn cli_help_flag_exits_via_clap() {
    let err = Cli::try_parse_from(["plm", "--help"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
}

#[test]
fn cli_version_flag_still_errors() {
    let result = Cli::try_parse_from(["plm", "--version"]);
    assert!(result.is_err(), "--version は従来通り未知引数エラー");
}

#[test]
fn cli_managed_help_displays_help() {
    let err = Cli::try_parse_from(["plm", "managed", "--help"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
}
