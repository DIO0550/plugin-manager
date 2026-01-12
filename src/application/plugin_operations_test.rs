use super::*;

#[test]
fn test_get_uninstall_info_not_found() {
    // 存在しないプラグインの場合はエラーを返す
    let result = get_uninstall_info("nonexistent-plugin-12345", Some("github"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));
    assert!(err.contains("nonexistent-plugin-12345"));
}

#[test]
fn test_get_uninstall_info_default_marketplace() {
    // マーケットプレイス未指定時は "github" がデフォルト
    let result = get_uninstall_info("nonexistent-plugin-12345", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("github"));
}

#[test]
fn test_uninstall_plugin_not_found() {
    // 存在しないプラグインのアンインストールはエラー
    let temp_dir = std::env::temp_dir();
    let result = uninstall_plugin("nonexistent-plugin-12345", Some("github"), &temp_dir);
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn test_disable_plugin_not_found() {
    // 存在しないプラグインのdisableはエラー
    let temp_dir = std::env::temp_dir();
    let result = disable_plugin("nonexistent-plugin-12345", Some("github"), &temp_dir);
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn test_enable_plugin_not_found() {
    // 存在しないプラグインのenableはエラー
    let temp_dir = std::env::temp_dir();
    let result = enable_plugin("nonexistent-plugin-12345", Some("github"), &temp_dir);
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("not found"));
}
