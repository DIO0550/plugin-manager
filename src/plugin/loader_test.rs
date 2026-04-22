use super::*;
use crate::fs::RealFs;
use crate::target::{PluginOrigin, TargetKind};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn origin() -> PluginOrigin {
    PluginOrigin::from_marketplace("mp", "plg")
}

/// Create an empty plugin dir `base/<kind_subdir>/<marketplace>/<plugin>`.
fn make_empty_plugin_dir(base: &std::path::Path, kind_subdir: &str) -> PathBuf {
    let dir = base.join(kind_subdir).join("mp").join("plg");
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cleanup_codex_removes_empty_plugin_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&home.join(".codex"), "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_codex_removes_empty_plugin_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".codex"), "agents");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_prompts_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".github"), "prompts");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Copilot, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_hooks_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&home.join(".copilot"), "hooks");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Copilot, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_hooks_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".github"), "hooks");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Copilot, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_antigravity_removes_empty_skills_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let base = home.join(".gemini").join("antigravity");
    let plugin_dir = make_empty_plugin_dir(&base, "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Antigravity, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_antigravity_removes_empty_skills_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".agent"), "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Antigravity, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_gemini_cli_removes_empty_skills_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&home.join(".gemini"), "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::GeminiCli, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_gemini_cli_removes_empty_skills_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".gemini"), "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::GeminiCli, &home, &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_keeps_non_empty_dir() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".codex"), "skills");
    // Add a file to make it non-empty
    fs::write(plugin_dir.join("residual.md"), "keep me").unwrap();

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, &home, &origin(), &project);

    assert!(plugin_dir.exists());
}

#[test]
fn cleanup_is_noop_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");

    // No directories created — should not panic or error
    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, &home, &origin(), &project);
    cleanup_plugin_directories_impl(&RealFs, TargetKind::Copilot, &home, &origin(), &project);
    cleanup_plugin_directories_impl(&RealFs, TargetKind::Antigravity, &home, &origin(), &project);
    cleanup_plugin_directories_impl(&RealFs, TargetKind::GeminiCli, &home, &origin(), &project);
}
