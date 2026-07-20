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

#[test]
fn home_dir_ignores_plm_home() {
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    let _lock = env_lock().lock().unwrap();
    let plm_home = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let prev_plm = std::env::var_os("PLM_HOME");
    let prev_home = std::env::var_os("HOME");
    std::env::set_var("PLM_HOME", plm_home.path());
    std::env::set_var("HOME", home.path());

    let got = home_dir();
    assert_eq!(got, home.path());
    assert_ne!(got, plm_home.path());

    match prev_plm {
        Some(v) => std::env::set_var("PLM_HOME", v),
        None => std::env::remove_var("PLM_HOME"),
    }
    match prev_home {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
}
