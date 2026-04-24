use super::*;
use std::path::Path;

#[test]
fn base_dir_personal_uses_home_personal_subdir() {
    // HOME の値そのものに依存させず、同じ home_dir() の結果で期待値を組み立てる。
    // これにより Personal 分岐が personal_subdir 側を選ぶことと、
    // project_subdir 側は一切使われないことを full-path で検証できる。
    let expected = home_dir().join(".codex");
    let dir = base_dir(
        Scope::Personal,
        Path::new("/proj"),
        ".codex",
        ".codex_project",
    );
    assert_eq!(dir, expected);
}

#[test]
fn base_dir_project_uses_project_root_with_project_subdir() {
    let dir = base_dir(Scope::Project, Path::new("/proj"), ".codex", ".github");
    assert_eq!(dir, Path::new("/proj/.github"));
}
