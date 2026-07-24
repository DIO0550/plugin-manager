use super::*;

#[test]
fn test_parse_target_codex() {
    let target = parse_target("codex").unwrap();
    assert_eq!(target.name(), "codex");
}

#[test]
fn test_parse_target_copilot() {
    let target = parse_target("copilot").unwrap();
    assert_eq!(target.name(), "copilot");
}

#[test]
fn test_parse_target_unknown() {
    let result = parse_target("unknown");
    assert!(result.is_err());
}

#[test]
fn test_parse_target_antigravity() {
    let target = parse_target("antigravity").unwrap();
    assert_eq!(target.name(), "antigravity");
}

#[test]
fn test_all_targets() {
    let targets = all_targets();
    assert_eq!(targets.len(), 5);
    assert!(targets.iter().any(|t| t.name() == "antigravity"));
    assert!(targets.iter().any(|t| t.name() == "codex"));
    assert!(targets.iter().any(|t| t.name() == "copilot"));
    assert!(targets.iter().any(|t| t.name() == "cursor"));
    assert!(targets.iter().any(|t| t.name() == "gemini"));
}

#[test]
fn test_parse_target_gemini() {
    let target = parse_target("gemini").unwrap();
    assert_eq!(target.name(), "gemini");
}

#[test]
fn test_parse_target_cursor() {
    let target = parse_target("cursor").unwrap();
    assert_eq!(target.name(), "cursor");
    assert_eq!(target.display_name(), "Cursor");
    assert_eq!(target.kind(), TargetKind::Cursor);
}

#[test]
fn test_target_kind_cursor_as_str_and_formats() {
    assert_eq!(TargetKind::Cursor.as_str(), "cursor");
    assert_eq!(
        TargetKind::Cursor.command_format(),
        crate::component::CommandFormat::ClaudeCode
    );
    assert_eq!(
        TargetKind::Cursor.agent_format(),
        crate::component::AgentFormat::ClaudeCode
    );
}

#[test]
fn test_name_is_derived_from_kind() {
    // `Target::name()` は `kind().as_str()` から導出される単一の真実源。
    // 各ターゲットで両者が一致することを保証し、識別子表現のドリフトを防ぐ。
    for target in all_targets() {
        assert_eq!(target.name(), target.kind().as_str());
    }
}

#[test]
fn test_can_place_scope_matches_supports_and_supported_components() {
    // can_place_scope がサポート判定の単一真実源。
    // supports(kind) ⇔ supported_components 包含
    // supports_scope ⇔ can_place_scope
    // いずれかの scope で can_place_scope が true ⇔ supported_components に含まれる
    for target in all_targets() {
        for &kind in &[
            ComponentKind::Skill,
            ComponentKind::Agent,
            ComponentKind::Command,
            ComponentKind::Instruction,
            ComponentKind::Hook,
        ] {
            assert_eq!(
                target.supports(kind),
                target.supported_components().contains(&kind),
                "{} supports({}) vs supported_components",
                target.name(),
                kind
            );
            for &scope in &[Scope::Personal, Scope::Project] {
                assert_eq!(
                    target.supports_scope(kind, scope),
                    target.can_place_scope(kind, scope),
                    "{} supports_scope({:?}, {:?})",
                    target.name(),
                    kind,
                    scope
                );
            }
            let any_scope = target.can_place_scope(kind, Scope::Personal)
                || target.can_place_scope(kind, Scope::Project);
            assert_eq!(
                any_scope,
                target.supported_components().contains(&kind),
                "{} kind {:?} any-scope vs supported_components",
                target.name(),
                kind
            );
        }
    }
}

// ========================================
// PluginOrigin tests
// ========================================

#[test]
fn test_plugin_origin_from_marketplace_named() {
    let origin = PluginOrigin::from_marketplace("official", "my-plugin");
    assert_eq!(origin.dir_names(), Some(("official", "my-plugin")));
}

#[test]
fn test_plugin_origin_from_marketplace_github_is_normalized() {
    // marketplace = "github" は Github variant に正規化される
    let origin = PluginOrigin::from_marketplace("github", "owner--repo");
    assert!(matches!(origin, PluginOrigin::Github { .. }));
    assert_eq!(origin.dir_names(), Some(("github", "owner--repo")));
}

#[test]
fn test_plugin_origin_from_github() {
    let origin = PluginOrigin::from_github("owner", "repo");
    assert!(matches!(origin, PluginOrigin::Github { .. }));
    // owner--repo エンコードは GithubCacheId に集約されている
    assert_eq!(origin.dir_names(), Some(("github", "owner--repo")));
}

#[test]
fn test_plugin_origin_from_cached_plugin_none_is_github() {
    let origin = PluginOrigin::from_cached_plugin(None, "owner--repo");
    assert!(matches!(origin, PluginOrigin::Github { .. }));
    assert_eq!(origin.dir_names(), Some(("github", "owner--repo")));
}

#[test]
fn test_plugin_origin_from_cached_plugin_some_github_is_github() {
    // Some("github") と None の意味揺れを 1 つの variant に畳み込む
    let origin = PluginOrigin::from_cached_plugin(Some("github"), "owner--repo");
    assert!(matches!(origin, PluginOrigin::Github { .. }));
}

#[test]
fn test_plugin_origin_from_cached_plugin_named_marketplace() {
    let origin = PluginOrigin::from_cached_plugin(Some("official"), "my-plugin");
    assert_eq!(origin.dir_names(), Some(("official", "my-plugin")));
}

#[test]
fn test_plugin_origin_unknown_has_no_dir_names() {
    assert_eq!(PluginOrigin::Unknown.dir_names(), None);
}
