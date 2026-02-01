use crate::application::PluginSummary;
use crate::tui::manager::core::filter::filter_plugins;

fn make_plugin(name: &str, marketplace: Option<&str>) -> PluginSummary {
    PluginSummary {
        name: name.to_string(),
        marketplace: marketplace.map(|m| m.to_string()),
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
fn empty_filter_returns_all() {
    let plugins = vec![
        make_plugin("foo", None),
        make_plugin("bar", Some("mymarket")),
    ];
    let result = filter_plugins(&plugins, "");
    assert_eq!(result.len(), 2);
}

#[test]
fn filter_by_name_partial_match() {
    let plugins = vec![make_plugin("my-plugin", None), make_plugin("other", None)];
    let result = filter_plugins(&plugins, "plug");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "my-plugin");
}

#[test]
fn filter_by_marketplace_partial_match() {
    let plugins = vec![
        make_plugin("foo", Some("awesome-market")),
        make_plugin("bar", None),
    ];
    let result = filter_plugins(&plugins, "awesome");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "foo");
}

#[test]
fn filter_case_insensitive() {
    let plugins = vec![make_plugin("MyPlugin", None), make_plugin("other", None)];
    let result = filter_plugins(&plugins, "myplugin");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "MyPlugin");
}

#[test]
fn filter_no_match_returns_empty() {
    let plugins = vec![make_plugin("foo", None), make_plugin("bar", None)];
    let result = filter_plugins(&plugins, "xyz");
    assert!(result.is_empty());
}

#[test]
fn filter_with_none_marketplace() {
    let plugins = vec![make_plugin("test", None)];
    let result = filter_plugins(&plugins, "market");
    assert!(result.is_empty());
}
