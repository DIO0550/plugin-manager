use super::*;

#[test]
fn test_plugin_intent_expand_empty() {
    let intent = PluginIntent::new(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![],
        std::env::temp_dir(),
    );

    let ops = intent.expand();
    assert!(ops.is_empty());
}
