use super::*;
use crate::marketplace::MarketplaceManifest;
use crate::plugin::cached_package::CachedPackage;
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
        installed_at: None,
    }
}

fn create_test_marketplace_content() -> MarketplaceContent {
    let cached = CachedPackage {
        name: "test-plugin".to_string(),
        cache_key: None,
        marketplace: Some("test-marketplace".to_string()),
        path: PathBuf::from("/tmp/test-plugin"),
        manifest: make_manifest("test-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    MarketplaceContent::from(cached)
}

fn create_test_marketplace_content_with_cache_key(key: &str) -> MarketplaceContent {
    let cached = CachedPackage {
        name: "test-plugin".to_string(),
        cache_key: Some(key.to_string()),
        marketplace: Some("test-marketplace".to_string()),
        path: PathBuf::from("/tmp/test-plugin"),
        manifest: make_manifest("test-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    MarketplaceContent::from(cached)
}

fn create_test_marketplace_content_no_marketplace() -> MarketplaceContent {
    let cached = CachedPackage {
        name: "test-plugin".to_string(),
        cache_key: None,
        marketplace: None,
        path: PathBuf::from("/tmp/test-plugin"),
        manifest: make_manifest("test-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: None,
    };
    MarketplaceContent::from(cached)
}

#[test]
fn test_name_returns_package_name() {
    let pkg = create_test_marketplace_content();
    let name: &str = pkg.name();
    assert_eq!(name, "test-plugin");
}

#[test]
fn test_cache_key_returns_some_when_present() {
    let pkg = create_test_marketplace_content_with_cache_key("owner--repo");
    let key: Option<&str> = pkg.cache_key();
    assert_eq!(key, Some("owner--repo"));
}

#[test]
fn test_cache_key_returns_none_when_absent() {
    let pkg = create_test_marketplace_content();
    let key: Option<&str> = pkg.cache_key();
    assert_eq!(key, None);
}

#[test]
fn test_marketplace_returns_some_when_present() {
    let pkg = create_test_marketplace_content();
    let mp: Option<&str> = pkg.marketplace();
    assert_eq!(mp, Some("test-marketplace"));
}

#[test]
fn test_marketplace_returns_none_when_absent() {
    let pkg = create_test_marketplace_content_no_marketplace();
    let mp: Option<&str> = pkg.marketplace();
    assert_eq!(mp, None);
}

#[test]
fn test_path_returns_package_path() {
    let pkg = create_test_marketplace_content();
    let path: &Path = pkg.path();
    assert_eq!(path, Path::new("/tmp/test-plugin"));
}

#[test]
fn test_manifest_returns_plugin_manifest() {
    let pkg = create_test_marketplace_content();
    let manifest: &PluginManifest = pkg.manifest();
    assert_eq!(manifest.name, "test-plugin");
    assert_eq!(manifest.version, "1.0.0");
}

#[test]
fn test_marketplace_manifest_returns_some_when_present() {
    let mp_manifest = MarketplaceManifest {
        name: "test-mp".to_string(),
        owner: None,
        plugins: vec![],
    };
    let cached = CachedPackage {
        name: "test-plugin".to_string(),
        cache_key: None,
        marketplace: Some("test-marketplace".to_string()),
        path: PathBuf::from("/tmp/test-plugin"),
        manifest: make_manifest("test-plugin"),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
        marketplace_manifest: Some(mp_manifest),
    };
    let pkg = MarketplaceContent::from(cached);
    let result = pkg.marketplace_manifest();
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "test-mp");
}

#[test]
fn test_marketplace_manifest_returns_none_when_absent() {
    let pkg = create_test_marketplace_content();
    assert!(pkg.marketplace_manifest().is_none());
}
