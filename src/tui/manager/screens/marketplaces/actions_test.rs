use crate::application::InstalledPlugin;
use crate::marketplace::{MarketplaceCache, MarketplacePlugin, MarketplaceRegistry, PluginSource};
use crate::tui::manager::screens::marketplaces::model::PluginInstallResult;

// ============================================================================
// install_plugins テスト用ヘルパー
// ============================================================================

fn make_success_result(name: &str) -> PluginInstallResult {
    PluginInstallResult {
        plugin_name: name.to_string(),
        success: true,
        error: None,
    }
}

fn make_failure_result(name: &str, error: &str) -> PluginInstallResult {
    PluginInstallResult {
        plugin_name: name.to_string(),
        success: false,
        error: Some(error.to_string()),
    }
}

fn make_plugin(name: &str) -> InstalledPlugin {
    InstalledPlugin::new_for_test(name, "1.0.0", Vec::new(), None, None, true)
}

fn make_marketplace_plugin(name: &str) -> MarketplacePlugin {
    MarketplacePlugin {
        name: name.to_string(),
        source: PluginSource::Local(format!("./plugins/{}", name)),
        description: Some(format!("{} description", name)),
        version: Some("1.0.0".to_string()),
    }
}

fn make_cache(name: &str, plugins: Vec<MarketplacePlugin>) -> MarketplaceCache {
    MarketplaceCache {
        name: name.to_string(),
        fetched_at: chrono::Utc::now(),
        source: "owner/repo".to_string(),
        owner: None,
        plugins,
    }
}

#[test]
fn returns_plugins_with_installed_flag() {
    let cache = make_cache(
        "test-mp",
        vec![
            make_marketplace_plugin("plugin-a"),
            make_marketplace_plugin("plugin-b"),
            make_marketplace_plugin("plugin-c"),
        ],
    );
    let installed = vec![make_plugin("plugin-b")];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "plugin-a");
    assert!(!result[0].installed);
    assert_eq!(result[1].name, "plugin-b");
    assert!(result[1].installed);
    assert_eq!(result[2].name, "plugin-c");
    assert!(!result[2].installed);
}

#[test]
fn returns_empty_when_cache_not_found() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let registry = MarketplaceRegistry::with_cache_dir(tmp_dir.path().to_path_buf()).unwrap();
    let installed: Vec<InstalledPlugin> = vec![];

    let result = super::get_browse_plugins_with_registry(&registry, "nonexistent", &installed);

    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_cache_has_no_plugins() {
    let cache = make_cache("test-mp", vec![]);
    let installed: Vec<InstalledPlugin> = vec![];

    let result = super::build_browse_plugins(&cache, &installed);

    assert!(result.is_empty());
}

#[test]
fn all_plugins_installed() {
    let cache = make_cache(
        "test-mp",
        vec![
            make_marketplace_plugin("plugin-a"),
            make_marketplace_plugin("plugin-b"),
        ],
    );
    let installed = vec![make_plugin("plugin-a"), make_plugin("plugin-b")];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 2);
    assert!(result[0].installed);
    assert!(result[1].installed);
}

#[test]
fn no_plugins_installed() {
    let cache = make_cache(
        "test-mp",
        vec![
            make_marketplace_plugin("plugin-a"),
            make_marketplace_plugin("plugin-b"),
        ],
    );
    let installed: Vec<InstalledPlugin> = vec![];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 2);
    assert!(!result[0].installed);
    assert!(!result[1].installed);
}

#[test]
fn installed_check_is_exact_match() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("my-plugin")]);
    let installed = vec![make_plugin("my-plugin-full")];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(!result[0].installed);
}

#[test]
fn installed_check_is_case_sensitive() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("my-plugin")]);
    let installed = vec![make_plugin("My-Plugin")];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(!result[0].installed);
}

#[test]
fn maps_all_fields_correctly() {
    let mp = MarketplacePlugin {
        name: "test-plugin".to_string(),
        source: PluginSource::Local("./plugins/test".to_string()),
        description: Some("A test plugin".to_string()),
        version: Some("2.0.0".to_string()),
    };
    let cache = make_cache("test-mp", vec![mp]);
    let installed: Vec<InstalledPlugin> = vec![];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "test-plugin");
    assert_eq!(result[0].description, Some("A test plugin".to_string()));
    assert_eq!(result[0].version, Some("2.0.0".to_string()));
    assert!(matches!(result[0].source, PluginSource::Local(ref s) if s == "./plugins/test"));
    assert!(!result[0].installed);
}

#[test]
fn disabled_plugin_still_counts_as_installed() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("plugin-a")]);
    let plugin = InstalledPlugin::new_for_test("plugin-a", "1.0.0", Vec::new(), None, None, false);
    let installed = vec![plugin];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(result[0].installed);
}

#[test]
fn installed_detected_by_id_when_name_differs() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("owner--repo")]);
    let plugin = InstalledPlugin::new_for_test(
        "Display Name",
        "1.0.0",
        Vec::new(),
        Some("owner--repo".to_string()),
        None,
        true,
    );
    let installed = vec![plugin];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(result[0].installed);
}

#[test]
fn same_name_different_marketplace_counts_as_installed() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("plugin-a")]);
    let plugin = InstalledPlugin::new_for_test(
        "plugin-a",
        "1.0.0",
        Vec::new(),
        None,
        Some("other-mp".to_string()),
        true,
    );
    let installed = vec![plugin];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(result[0].installed);
}

#[test]
fn returns_empty_when_cache_json_corrupted() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let registry = MarketplaceRegistry::with_cache_dir(tmp_dir.path().to_path_buf()).unwrap();

    // 不正なJSONファイルを直接書き込み
    std::fs::write(
        tmp_dir.path().join("corrupted-mp.json"),
        "not valid json{{{",
    )
    .unwrap();

    let installed: Vec<InstalledPlugin> = vec![];
    let result = super::get_browse_plugins_with_registry(&registry, "corrupted-mp", &installed);

    assert!(result.is_empty());
}

#[test]
fn returns_plugins_via_registry() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let registry = MarketplaceRegistry::with_cache_dir(tmp_dir.path().to_path_buf()).unwrap();

    let cache = make_cache(
        "test-mp",
        vec![
            make_marketplace_plugin("plugin-a"),
            make_marketplace_plugin("plugin-b"),
        ],
    );
    registry.store(&cache).unwrap();

    let installed = vec![make_plugin("plugin-a")];
    let result = super::get_browse_plugins_with_registry(&registry, "test-mp", &installed);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "plugin-a");
    assert!(result[0].installed);
    assert_eq!(result[1].name, "plugin-b");
    assert!(!result[1].installed);
}

// ============================================================================
// build_install_summary テスト (T-01 -- T-07)
// ============================================================================

#[test]
fn build_summary_all_success() {
    let results = vec![make_success_result("a"), make_success_result("b")];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 2);
    assert_eq!(summary.failed, 0);
}

#[test]
fn build_summary_all_failure() {
    let results = vec![
        make_failure_result("a", "err1"),
        make_failure_result("b", "err2"),
    ];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 2);
}

#[test]
fn build_summary_mixed() {
    let results = vec![make_success_result("a"), make_failure_result("b", "err")];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 1);
    assert_eq!(summary.failed, 1);
}

#[test]
fn build_summary_empty() {
    let results: Vec<PluginInstallResult> = vec![];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 0);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn build_summary_single_success() {
    let results = vec![make_success_result("a")];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.succeeded, 1);
    assert_eq!(summary.failed, 0);
}

#[test]
fn build_summary_single_failure() {
    let results = vec![make_failure_result("a", "err")];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 1);
}

#[test]
fn build_summary_preserves_results() {
    let results = vec![
        make_success_result("x"),
        make_failure_result("y", "fail"),
        make_success_result("z"),
    ];
    let summary = super::build_install_summary(results);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.succeeded, 2);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.results[0].plugin_name, "x");
    assert!(summary.results[0].success);
    assert_eq!(summary.results[1].plugin_name, "y");
    assert!(!summary.results[1].success);
    assert_eq!(summary.results[1].error, Some("fail".to_string()));
    assert_eq!(summary.results[2].plugin_name, "z");
    assert!(summary.results[2].success);
}

// ============================================================================
// make_all_failed_summary テスト (T-08 -- T-10)
// ============================================================================

#[test]
fn all_failed_summary_records_error() {
    let names = vec!["a".to_string(), "b".to_string()];
    let summary = super::make_all_failed_summary(&names, "runtime error");
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 2);
    for r in &summary.results {
        assert!(!r.success);
        assert_eq!(r.error, Some("runtime error".to_string()));
    }
}

#[test]
fn all_failed_summary_empty_names() {
    let names: Vec<String> = vec![];
    let summary = super::make_all_failed_summary(&names, "err");
    assert_eq!(summary.total, 0);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn all_failed_summary_preserves_names() {
    let names = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let summary = super::make_all_failed_summary(&names, "err");
    assert_eq!(summary.results.len(), 3);
    assert_eq!(summary.results[0].plugin_name, "x");
    assert_eq!(summary.results[1].plugin_name, "y");
    assert_eq!(summary.results[2].plugin_name, "z");
}

// ============================================================================
// install_plugins precondition テスト
// ============================================================================

#[test]
fn install_plugins_empty_plugin_names_returns_empty_summary() {
    let plugins: Vec<String> = vec![];
    let targets = vec!["codex".to_string()];
    let summary = super::install_plugins(
        "test-mp",
        &plugins,
        &targets,
        crate::component::Scope::Personal,
    );
    assert_eq!(summary.total, 0);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 0);
}

#[test]
fn install_plugins_empty_target_names_returns_all_failed() {
    let plugins = vec!["plugin-a".to_string(), "plugin-b".to_string()];
    let targets: Vec<String> = vec![];
    let summary = super::install_plugins(
        "test-mp",
        &plugins,
        &targets,
        crate::component::Scope::Personal,
    );
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 2);
    for r in &summary.results {
        assert!(!r.success);
        assert_eq!(r.error, Some("No targets specified".to_string()));
    }
}

#[test]
fn install_plugins_invalid_target_returns_all_failed() {
    let plugins = vec!["plugin-a".to_string()];
    let targets = vec!["nonexistent-target".to_string()];
    let summary = super::install_plugins(
        "test-mp",
        &plugins,
        &targets,
        crate::component::Scope::Personal,
    );
    assert_eq!(summary.total, 1);
    assert_eq!(summary.succeeded, 0);
    assert_eq!(summary.failed, 1);
    assert!(!summary.results[0].success);
    assert!(summary.results[0].error.is_some());
}

#[test]
fn install_plugins_no_runtime_returns_all_failed() {
    // Tokio ランタイムのない別スレッドから呼び出して検証
    let result = std::thread::spawn(|| {
        let plugins = vec!["plugin-a".to_string(), "plugin-b".to_string()];
        let targets = vec!["codex".to_string()];
        super::install_plugins(
            "test-mp",
            &plugins,
            &targets,
            crate::component::Scope::Personal,
        )
    })
    .join()
    .unwrap();

    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 0);
    assert_eq!(result.failed, 2);
    for r in &result.results {
        assert!(!r.success);
        assert_eq!(r.error, Some("No Tokio runtime available".to_string()));
    }
}
