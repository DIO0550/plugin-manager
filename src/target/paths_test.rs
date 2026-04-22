use super::*;
use std::path::Path;

#[test]
fn base_dir_personal_uses_home_personal_subdir() {
    let dir = base_dir(
        Scope::Personal,
        Path::new("/proj"),
        ".codex",
        ".codex_project",
    );
    // HOME が設定されていれば home_dir().join(".codex") になる
    assert!(dir.ends_with(".codex"));
}

#[test]
fn base_dir_project_uses_project_root_with_project_subdir() {
    let dir = base_dir(Scope::Project, Path::new("/proj"), ".codex", ".github");
    assert_eq!(dir, Path::new("/proj/.github"));
}
