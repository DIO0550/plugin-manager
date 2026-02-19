use super::*;

#[test]
fn test_plugin_plan_expand_empty() {
    let plan = PluginIntent::new(
        PluginAction::Enable {
            plugin_name: "test-plugin".to_string(),
            marketplace: None,
        },
        vec![],
        std::env::temp_dir(),
    );

    let ops = plan.expand();
    assert!(ops.is_empty());
}
