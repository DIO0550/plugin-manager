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

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, Some(&home), &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_codex_removes_empty_plugin_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".codex"), "agents");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, Some(&home), &origin(), &project);

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_prompts_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".github"), "prompts");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Copilot,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_hooks_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&home.join(".copilot"), "hooks");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Copilot,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_copilot_removes_empty_hooks_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".github"), "hooks");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Copilot,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_antigravity_removes_empty_skills_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let base = home.join(".gemini").join("antigravity");
    let plugin_dir = make_empty_plugin_dir(&base, "skills");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Antigravity,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_antigravity_removes_empty_skills_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".agent"), "skills");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Antigravity,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_gemini_cli_removes_empty_skills_dir_personal() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&home.join(".gemini"), "skills");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::GeminiCli,
        Some(&home),
        &origin(),
        &project,
    );

    assert!(!plugin_dir.exists());
}

#[test]
fn cleanup_gemini_cli_removes_empty_skills_dir_project() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");
    let plugin_dir = make_empty_plugin_dir(&project.join(".gemini"), "skills");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::GeminiCli,
        Some(&home),
        &origin(),
        &project,
    );

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

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, Some(&home), &origin(), &project);

    assert!(plugin_dir.exists());
}

#[test]
fn cleanup_is_noop_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");

    // No directories created — should not panic or error
    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, Some(&home), &origin(), &project);
    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Copilot,
        Some(&home),
        &origin(),
        &project,
    );
    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Antigravity,
        Some(&home),
        &origin(),
        &project,
    );
    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::GeminiCli,
        Some(&home),
        &origin(),
        &project,
    );
}

/// 不正な `..` を含む origin が渡された場合、cleanup が base の外側を
/// 触らずスキップすることを確認する（path-escape 防御）。
#[test]
fn cleanup_rejects_origin_with_traversal_segment() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");

    // base の外にダミーディレクトリを作っておき、cleanup がこれを消さないことを確認する。
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();

    let bad_origin = PluginOrigin::from_marketplace("..", "..");

    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Codex,
        Some(&home),
        &bad_origin,
        &project,
    );

    assert!(
        outside.exists(),
        "cleanup must not escape base via `..` segments"
    );
}

/// パスセパレータを含む origin も同様にスキップされることを確認する。
#[test]
fn cleanup_rejects_origin_with_path_separator() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("proj");

    let bad_origin = PluginOrigin::from_marketplace("mp/inner", "plg");

    // スキップされる（panic しない）ことだけを確認する。
    cleanup_plugin_directories_impl(
        &RealFs,
        TargetKind::Codex,
        Some(&home),
        &bad_origin,
        &project,
    );
}

/// `home` が None の場合、personal scope の cleanup 対象はゼロ個となり
/// （`./~/.codex` のような誤パスを走査せず）、project scope 側は引き続き
/// 削除されることを確認する。
#[test]
fn cleanup_skips_personal_when_home_is_none() {
    let tmp = TempDir::new().unwrap();
    let project = tmp.path().join("proj");
    let project_plugin_dir = make_empty_plugin_dir(&project.join(".codex"), "skills");

    // CWD を一時ディレクトリに変更し、literal "~" 解決が意図しない場所を
    // 指すことに備える（この CWD に `~/.codex` を作っておき、cleanup が
    // それを触らないことを確認する）。
    let cwd_guard = tmp.path().join("cwd");
    std::fs::create_dir_all(&cwd_guard).unwrap();
    let tilde_dir = cwd_guard.join("~").join(".codex").join("skills");
    std::fs::create_dir_all(&tilde_dir).unwrap();

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, None, &origin(), &project);

    // project scope は正常に cleanup される
    assert!(
        !project_plugin_dir.exists(),
        "project scope cleanup must still run when HOME is unset"
    );
    // personal scope（literal `~` 解決先）は触られない
    assert!(
        tilde_dir.exists(),
        "personal cleanup must be skipped when HOME is unset"
    );
}
