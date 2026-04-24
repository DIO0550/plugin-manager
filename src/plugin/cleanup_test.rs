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

/// `home` が None の場合でも project scope の cleanup は通常通り走ることを
/// 確認する（integration レベルの回帰防止）。
#[test]
fn cleanup_runs_project_scope_when_home_is_none() {
    let tmp = TempDir::new().unwrap();
    let project = tmp.path().join("proj");
    let project_plugin_dir = make_empty_plugin_dir(&project.join(".codex"), "skills");

    cleanup_plugin_directories_impl(&RealFs, TargetKind::Codex, None, &origin(), &project);

    assert!(
        !project_plugin_dir.exists(),
        "project scope cleanup must still run when HOME is unset"
    );
}

/// `home` が None の場合、`cleanup_specs` は personal scope 側のエントリを
/// まるごと省略して project_root 配下のみを返すことを直接検証する。
/// これにより literal `~` を含む誤パス（`./~/.codex` など）が絶対に
/// cleanup 対象に含まれないことを構造的に保証する。
#[test]
fn cleanup_specs_with_home_none_returns_only_project_entries() {
    let project = std::path::Path::new("/proj");

    for kind in [
        TargetKind::Codex,
        TargetKind::Copilot,
        TargetKind::Antigravity,
        TargetKind::GeminiCli,
    ] {
        let specs = cleanup_specs(kind, None, project);
        assert!(
            !specs.is_empty(),
            "{kind:?}: project scope specs must exist"
        );
        for (base, _) in &specs {
            assert!(
                base.starts_with(project),
                "{kind:?}: unexpected non-project entry {base:?} when home=None"
            );
        }
    }
}
