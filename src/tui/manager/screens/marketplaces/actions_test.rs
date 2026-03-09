use crate::application::PluginSummary;
use crate::marketplace::{MarketplaceCache, MarketplacePlugin, MarketplaceRegistry, PluginSource};

fn make_plugin(name: &str) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: None,
        version: "1.0.0".to_string(),
        skills: vec![],
        agents: vec![],
        commands: vec![],
        instructions: vec![],
        hooks: vec![],
        enabled: true,
    }
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
    let installed: Vec<PluginSummary> = vec![];

    let result = super::get_browse_plugins_with_registry(&registry, "nonexistent", &installed);

    assert!(result.is_empty());
}

#[test]
fn returns_empty_when_cache_has_no_plugins() {
    let cache = make_cache("test-mp", vec![]);
    let installed: Vec<PluginSummary> = vec![];

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
    let installed: Vec<PluginSummary> = vec![];

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
    let installed: Vec<PluginSummary> = vec![];

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
    let mut plugin = make_plugin("plugin-a");
    plugin.enabled = false;
    let installed = vec![plugin];

    let result = super::build_browse_plugins(&cache, &installed);

    assert_eq!(result.len(), 1);
    assert!(result[0].installed);
}

#[test]
fn same_name_different_marketplace_counts_as_installed() {
    let cache = make_cache("test-mp", vec![make_marketplace_plugin("plugin-a")]);
    let mut plugin = make_plugin("plugin-a");
    plugin.marketplace = Some("other-mp".to_string());
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

    let installed: Vec<PluginSummary> = vec![];
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
