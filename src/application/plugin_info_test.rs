use super::*;

// ========================================
// parse_plugin_name tests
// ========================================

#[test]
fn test_parse_plugin_name_simple() {
    let result = parse_plugin_name("my-plugin").unwrap();
    assert_eq!(result, (None, "my-plugin".to_string()));
}

#[test]
fn test_parse_plugin_name_with_marketplace() {
    let result = parse_plugin_name("marketplace/plugin").unwrap();
    assert_eq!(
        result,
        (Some("marketplace".to_string()), "plugin".to_string())
    );
}

#[test]
fn test_parse_plugin_name_empty() {
    let result = parse_plugin_name("");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidArgument(msg) => {
            assert!(msg.contains("empty"));
        }
        e => panic!("Expected InvalidArgument, got: {:?}", e),
    }
}

#[test]
fn test_parse_plugin_name_leading_slash() {
    let result = parse_plugin_name("/plugin");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidArgument(msg) => {
            assert!(msg.contains("start with"));
        }
        e => panic!("Expected InvalidArgument, got: {:?}", e),
    }
}

#[test]
fn test_parse_plugin_name_trailing_slash() {
    let result = parse_plugin_name("plugin/");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidArgument(msg) => {
            assert!(msg.contains("end with"));
        }
        e => panic!("Expected InvalidArgument, got: {:?}", e),
    }
}

#[test]
fn test_parse_plugin_name_too_many_slashes() {
    let result = parse_plugin_name("a/b/c");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::InvalidArgument(msg) => {
            assert!(msg.contains("too many"));
        }
        e => panic!("Expected InvalidArgument, got: {:?}", e),
    }
}

// ========================================
// restore_github_repo tests
// ========================================

#[test]
fn test_restore_github_repo_normal() {
    assert_eq!(restore_github_repo("owner--repo"), "owner/repo");
}

#[test]
fn test_restore_github_repo_no_separator() {
    assert_eq!(restore_github_repo("simple-name"), "simple-name");
}

#[test]
fn test_restore_github_repo_multiple_dashes() {
    assert_eq!(
        restore_github_repo("owner--repo--extra"),
        "owner/repo--extra"
    );
}

// ========================================
// resolve_single_plugin tests
// ========================================

fn create_candidate(marketplace: &str, name: &str) -> PluginCandidate {
    PluginCandidate {
        marketplace: marketplace.to_string(),
        dir_name: name.to_string(),
        cache_path: PathBuf::from(format!("/cache/{}/{}", marketplace, name)),
        manifest: PluginManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            commands: None,
            agents: None,
            skills: None,
            instructions: None,
            hooks: None,
            mcp_servers: None,
            lsp_servers: None,
            installed_at: None,
        },
    }
}

#[test]
fn test_resolve_single_plugin_not_found() {
    let result = resolve_single_plugin(vec![], None, "missing");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::PluginNotFound(name) => {
            assert_eq!(name, "missing");
        }
        e => panic!("Expected PluginNotFound, got: {:?}", e),
    }
}

#[test]
fn test_resolve_single_plugin_one_match() {
    let candidates = vec![create_candidate("github", "my-plugin")];
    let result = resolve_single_plugin(candidates, None, "my-plugin");
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved.marketplace, "github");
}

#[test]
fn test_resolve_single_plugin_multiple_ambiguous() {
    let candidates = vec![
        create_candidate("marketplace-a", "common"),
        create_candidate("marketplace-b", "common"),
    ];
    let result = resolve_single_plugin(candidates, None, "common");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::AmbiguousPlugin { name, candidates } => {
            assert_eq!(name, "common");
            assert_eq!(candidates.len(), 2);
        }
        e => panic!("Expected AmbiguousPlugin, got: {:?}", e),
    }
}

#[test]
fn test_resolve_single_plugin_filtered_by_marketplace() {
    let candidates = vec![
        create_candidate("marketplace-a", "common"),
        create_candidate("marketplace-b", "common"),
    ];
    let result = resolve_single_plugin(candidates, Some("marketplace-a"), "common");
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved.marketplace, "marketplace-a");
}

#[test]
fn test_resolve_single_plugin_filtered_not_found() {
    let candidates = vec![create_candidate("marketplace-a", "common")];
    let result = resolve_single_plugin(candidates, Some("marketplace-b"), "common");
    assert!(result.is_err());
    match result.unwrap_err() {
        PlmError::PluginNotFound(name) => {
            assert_eq!(name, "common");
        }
        e => panic!("Expected PluginNotFound, got: {:?}", e),
    }
}

// ========================================
// determine_source_from_path tests
// ========================================

#[test]
fn test_determine_source_github() {
    let path = PathBuf::from("/cache/github/owner--repo");
    let source = determine_source_from_path(&path, "github", "owner--repo");
    match source {
        PluginSource::GitHub { repository } => {
            assert_eq!(repository, "owner/repo");
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[test]
fn test_determine_source_marketplace() {
    let path = PathBuf::from("/cache/awesome-plugins/my-plugin");
    let source = determine_source_from_path(&path, "awesome-plugins", "my-plugin");
    match source {
        PluginSource::Marketplace { name } => {
            assert_eq!(name, "awesome-plugins");
        }
        _ => panic!("Expected Marketplace source"),
    }
}
