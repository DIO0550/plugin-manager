use super::*;

#[test]
fn test_plugin_action_kind() {
    let install = PluginAction::Install {
        plugin_name: "test".to_string(),
        marketplace: None,
    };
    assert_eq!(install.kind(), "install");
    assert!(install.is_deploy());
    assert!(!install.is_remove());

    let disable = PluginAction::Disable {
        plugin_name: "test".to_string(),
        marketplace: Some("official".to_string()),
    };
    assert_eq!(disable.kind(), "disable");
    assert!(!disable.is_deploy());
    assert!(disable.is_remove());
}
