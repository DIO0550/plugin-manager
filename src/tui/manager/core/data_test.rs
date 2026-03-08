use crate::application::PluginSummary;
use crate::tui::manager::core::DataStore;

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
