use super::*;

#[test]
fn test_args_parsing() {
    use clap::CommandFactory;
    let cmd = Args::command();
    cmd.debug_assert();
}

// Integration tests (binary execution tests)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn plm() -> Command {
    Command::cargo_bin("plm").unwrap()
}

#[test]
fn test_sync_same_target_error() {
    // plm sync --from codex --to codex should fail
    plm()
        .args(["sync", "--from", "codex", "--to", "codex"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("same target"));
}

#[test]
fn test_sync_dry_run_shows_plan() {
    // --dry-run shows plan without actual copy
    plm()
        .args(["sync", "--from", "codex", "--to", "copilot", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would sync"));
}

#[test]
fn test_sync_unsupported_type_skips() {
    // copilot -> codex with command type should skip all
    // Note: Codex doesn't support Command type
    plm()
        .args([
            "sync",
            "--from",
            "copilot",
            "--to",
            "codex",
            "--type",
            "command",
            "--dry-run",
        ])
        .assert()
        .success();
}

#[test]
fn test_sync_project_scope_no_components_zero_exit() {
    // Project scope with no components should exit successfully
    let temp = TempDir::new().unwrap();

    plm()
        .current_dir(temp.path())
        .args([
            "sync", "--from", "codex", "--to", "copilot", "--scope", "project",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("No components to sync"));
}

#[test]
fn test_sync_creates_component() {
    // Actually create and sync a component
    let temp = TempDir::new().unwrap();

    // Create source skill directory structure
    let skill_dir = temp
        .path()
        .join(".codex")
        .join("skills")
        .join("github")
        .join("test-plugin")
        .join("my-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "# Test Skill").unwrap();

    // Sync from codex to copilot
    plm()
        .current_dir(temp.path())
        .args([
            "sync", "--from", "codex", "--to", "copilot", "--type", "skill", "--scope", "project",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced"));

    // Verify the skill was copied to copilot
    let target_skill = temp
        .path()
        .join(".github")
        .join("skills")
        .join("github")
        .join("test-plugin")
        .join("my-skill");
    assert!(target_skill.exists(), "Skill should be synced to copilot");
}

#[test]
fn test_sync_help_output() {
    plm()
        .args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--scope"))
        .stdout(predicate::str::contains("--dry-run"));
}
