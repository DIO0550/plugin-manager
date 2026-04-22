use super::*;
use crate::target::PluginOrigin;

#[test]
fn identity_new_sets_kind_and_name_with_none_scope() {
    let id = ComponentIdentity::new(ComponentKind::Skill, "my-skill");
    assert_eq!(id.kind, ComponentKind::Skill);
    assert_eq!(id.name, "my-skill");
    assert_eq!(id.scope, None);
}

#[test]
fn identity_with_scope_sets_scope() {
    let id = ComponentIdentity::new(ComponentKind::Agent, "agent").with_scope(Scope::Project);
    assert_eq!(id.scope, Some(Scope::Project));
}

#[test]
fn qualified_name_formats_marketplace_plugin_name() {
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");
    let id = ComponentIdentity::new(ComponentKind::Command, "commit");
    assert_eq!(id.qualified_name(&origin), "official/my-plugin/commit");
}
