use super::*;
use std::path::PathBuf;

fn make_manifest(name: &str) -> PluginManifest {
    PluginManifest {
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
    }
}

#[test]
fn id_returns_some_value_when_set() {
    let pkg = CachedPackage {
        name: "my-plugin".to_string(),
        id: Some("owner--repo".to_string()),
        marketplace: None,
        path: PathBuf::from("/tmp/test"),
        manifest: make_manifest("my-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    assert_eq!(pkg.id(), "owner--repo");
}

#[test]
fn id_falls_back_to_name_when_none() {
    let pkg = CachedPackage {
        name: "my-plugin".to_string(),
        id: None,
        marketplace: None,
        path: PathBuf::from("/tmp/test"),
        manifest: make_manifest("my-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    assert_eq!(pkg.id(), "my-plugin");
}
