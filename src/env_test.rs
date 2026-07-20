use super::*;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// 環境変数を触るテストの直列化用ロック
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn clear(keys: &[&'static str]) -> Self {
        let saved = keys
            .iter()
            .map(|&key| {
                let prev = std::env::var_os(key);
                std::env::remove_var(key);
                (key, prev)
            })
            .collect();
        Self { saved }
    }

    fn set(&self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        std::env::set_var(key, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, prev) in self.saved.drain(..) {
            match prev {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[test]
fn test_get_existing_var() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["TEST_ENV_VAR"]);
    guard.set("TEST_ENV_VAR", "test_value");
    assert_eq!(EnvVar::get("TEST_ENV_VAR"), Some("test_value".to_string()));
}

#[test]
fn test_get_empty_var() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["TEST_EMPTY_VAR"]);
    guard.set("TEST_EMPTY_VAR", "");
    assert_eq!(EnvVar::get("TEST_EMPTY_VAR"), None);
}

#[test]
fn test_get_nonexistent_var() {
    assert_eq!(EnvVar::get("NONEXISTENT_VAR_12345"), None);
}

#[test]
fn plm_paths_with_root_plm_dir() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(paths.plm_dir(), PathBuf::from("/tmp/foo/.plm"));
}

#[test]
fn plm_paths_with_root_targets_json() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(
        paths.targets_json(),
        PathBuf::from("/tmp/foo/.plm/targets.json")
    );
}

#[test]
fn plm_paths_with_root_marketplaces_json() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(
        paths.marketplaces_json(),
        PathBuf::from("/tmp/foo/.plm/marketplaces.json")
    );
}

#[test]
fn plm_paths_with_root_imports_json() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(
        paths.imports_json(),
        PathBuf::from("/tmp/foo/.plm/imports.json")
    );
}

#[test]
fn plm_paths_with_root_plugins_cache_dir() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(
        paths.plugins_cache_dir(),
        PathBuf::from("/tmp/foo/.plm/cache/plugins")
    );
}

#[test]
fn plm_paths_with_root_marketplaces_cache_dir() {
    let paths = PlmPaths::with_root(PathBuf::from("/tmp/foo"));
    assert_eq!(
        paths.marketplaces_cache_dir(),
        PathBuf::from("/tmp/foo/.plm/cache/marketplaces")
    );
}

#[test]
fn plm_paths_five_accessors_share_plm_prefix() {
    let root = PathBuf::from("/tmp/unify");
    let paths = PlmPaths::with_root(root.clone());
    let prefix = root.join(".plm");
    for path in [
        paths.plm_dir(),
        paths.targets_json(),
        paths.marketplaces_json(),
        paths.imports_json(),
        paths.plugins_cache_dir(),
        paths.marketplaces_cache_dir(),
    ] {
        assert!(
            path.starts_with(&prefix),
            "{} does not start with {}",
            path.display(),
            prefix.display()
        );
    }
}

#[test]
fn plm_root_uses_plm_home_when_set() {
    let _lock = env_lock().lock().unwrap();
    let plm_home = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", plm_home.path());
    guard.set("HOME", home.path());

    let root = plm_root().unwrap();
    assert_eq!(root, plm_home.path());
}

#[test]
fn plm_root_falls_back_to_home_when_plm_home_unset() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("HOME", home.path());

    let root = plm_root().unwrap();
    assert_eq!(root, home.path());
}

#[test]
fn plm_root_falls_back_when_plm_home_empty() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", "");
    guard.set("HOME", home.path());

    let root = plm_root().unwrap();
    assert_eq!(root, home.path());
}

#[test]
fn plm_root_falls_back_when_plm_home_whitespace() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", "   ");
    guard.set("HOME", home.path());

    let root = plm_root().unwrap();
    assert_eq!(root, home.path());
}

#[test]
fn plm_root_prefers_plm_home_over_home() {
    let _lock = env_lock().lock().unwrap();
    let plm_home = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", plm_home.path());
    guard.set("HOME", home.path());

    assert_eq!(plm_root().unwrap(), plm_home.path());
    assert_ne!(plm_root().unwrap(), home.path());
}

#[test]
fn plm_root_rejects_relative_plm_home() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", "relative/path");
    guard.set("HOME", home.path());

    let err = plm_root().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("absolute path"), "unexpected error: {}", msg);
}

#[test]
fn plm_root_rejects_relative_home() {
    let _lock = env_lock().lock().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("HOME", "relative");

    let err = plm_root().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("absolute path"), "unexpected error: {}", msg);
}

#[test]
fn plm_root_errors_when_both_unset() {
    let _lock = env_lock().lock().unwrap();
    let _guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);

    let err = plm_root().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("not set or empty"),
        "unexpected error: {}",
        msg
    );
}

#[test]
fn plm_root_is_idempotent() {
    let _lock = env_lock().lock().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("HOME", home.path());

    let a = plm_root().unwrap();
    let b = plm_root().unwrap();
    assert_eq!(a, b);
}

#[test]
fn plm_paths_new_uses_plm_root() {
    let _lock = env_lock().lock().unwrap();
    let plm_home = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let guard = EnvGuard::clear(&["PLM_HOME", "HOME"]);
    guard.set("PLM_HOME", plm_home.path());
    guard.set("HOME", home.path());

    let paths = PlmPaths::new().unwrap();
    assert_eq!(
        paths.targets_json(),
        plm_home.path().join(".plm").join("targets.json")
    );
}
