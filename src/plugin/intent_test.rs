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

    let result = intent.expand();
    assert!(result.operations.is_empty());
    assert!(result.validation_errors.is_empty());
}
