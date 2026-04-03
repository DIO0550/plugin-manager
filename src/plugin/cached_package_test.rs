use super::*;
use std::path::PathBuf;

fn make_manifest() -> PluginManifest {
    PluginManifest {
        name: "test-plugin".to_string(),
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
    }
}

#[test]
fn cache_key_returns_some_value_when_set() {
    let pkg = CachedPackage {
        name: "my-plugin".to_string(),
        cache_key: Some("owner--repo".to_string()),
        marketplace: None,
        path: PathBuf::from("/tmp/test"),
        manifest: make_manifest(),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
    };
    assert_eq!(pkg.cache_key(), "owner--repo");
}

#[test]
fn cache_key_falls_back_to_name_when_none() {
    let pkg = CachedPackage {
        name: "my-plugin".to_string(),
        cache_key: None,
        marketplace: None,
        path: PathBuf::from("/tmp/test"),
        manifest: make_manifest(),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
    };
    assert_eq!(pkg.cache_key(), "my-plugin");
}
