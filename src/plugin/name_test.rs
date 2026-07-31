use super::PluginName;

#[test]
fn accepts_plain_name() {
    assert_eq!(
        PluginName::new("spec-plugin").unwrap().as_str(),
        "spec-plugin"
    );
}

#[test]
fn rejects_empty_dot_and_separators() {
    assert!(PluginName::new("").is_none());
    assert!(PluginName::new(".").is_none());
    assert!(PluginName::new("..").is_none());
    assert!(PluginName::new("a/b").is_none());
    assert!(PluginName::new("a\\b").is_none());
    assert!(PluginName::new("a\0b").is_none());
}
