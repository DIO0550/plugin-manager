use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;

fn make_plugin(name: &str) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        cache_key: None,
        marketplace: None,
        version: "1.0.0".to_string(),
        components: Vec::new(),
        enabled: true,
    }
}

#[test]
fn returns_true_when_plugin_exists() {
    let (_tmp, store) = DataStore::for_test(
        vec![make_plugin("owner--repo"), make_plugin("other--plugin")],
        vec![],
        None,
    );
    assert!(store.is_plugin_installed("owner--repo"));
}

#[test]
fn returns_false_when_plugin_not_exists() {
    let (_tmp, store) = DataStore::for_test(vec![make_plugin("owner--repo")], vec![], None);
    assert!(!store.is_plugin_installed("not-installed"));
}

#[test]
fn returns_false_when_plugins_empty() {
    let (_tmp, store) = DataStore::for_test(vec![], vec![], None);
    assert!(!store.is_plugin_installed("anything"));
}

#[test]
fn is_case_sensitive() {
    let (_tmp, store) = DataStore::for_test(vec![make_plugin("My-Plugin")], vec![], None);
    assert!(!store.is_plugin_installed("my-plugin"));
}

#[test]
fn rejects_partial_match() {
    let (_tmp, store) = DataStore::for_test(vec![make_plugin("my-plugin-full")], vec![], None);
    assert!(!store.is_plugin_installed("my-plugin"));
}

#[test]
fn ignores_marketplace_difference() {
    let plugin_a = PluginSummary {
        marketplace: None,
        ..make_plugin("shared-name")
    };
    let plugin_b = PluginSummary {
        marketplace: Some("market-a".to_string()),
        ..make_plugin("shared-name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin_a, plugin_b], vec![], None);
    assert!(store.is_plugin_installed("shared-name"));
}

#[test]
fn returns_true_when_plugin_disabled() {
    let plugin = PluginSummary {
        enabled: false,
        ..make_plugin("disabled-plugin")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    assert!(store.is_plugin_installed("disabled-plugin"));
}

// ============================================================================
// cache_key が設定されている場合のテスト
// ============================================================================

#[test]
fn is_plugin_installed_matches_by_cache_key() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    assert!(store.is_plugin_installed("owner--repo"));
}

#[test]
fn is_plugin_installed_does_not_match_by_display_name_when_cache_key_set() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    assert!(!store.is_plugin_installed("Display Name"));
}

#[test]
fn find_plugin_matches_by_cache_key() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    let found = store.find_plugin(&"owner--repo".to_string());
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Display Name");
}

#[test]
fn find_plugin_does_not_match_by_display_name_when_cache_key_set() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    assert!(store.find_plugin(&"Display Name".to_string()).is_none());
}

#[test]
fn plugin_index_matches_by_cache_key() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, store) = DataStore::for_test(vec![plugin], vec![], None);
    assert_eq!(store.plugin_index(&"owner--repo".to_string()), Some(0));
    assert_eq!(store.plugin_index(&"Display Name".to_string()), None);
}

#[test]
fn remove_plugin_works_by_cache_key() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, mut store) = DataStore::for_test(vec![plugin], vec![], None);
    store.remove_plugin(&"owner--repo".to_string());
    assert!(!store.is_plugin_installed("owner--repo"));
}

#[test]
fn set_plugin_enabled_works_by_cache_key() {
    let plugin = PluginSummary {
        cache_key: Some("owner--repo".to_string()),
        ..make_plugin("Display Name")
    };
    let (_tmp, mut store) = DataStore::for_test(vec![plugin], vec![], None);
    store.set_plugin_enabled(&"owner--repo".to_string(), false);
    let found = store.find_plugin(&"owner--repo".to_string()).unwrap();
    assert!(!found.enabled);
}
