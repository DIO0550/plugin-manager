use super::*;
use crate::host::HostKind;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

/// モック用のファイル取得結果
enum MockFileResult {
    Ok(String),
    Err { status: u16, message: String },
}

/// テスト用モッククライアント
///
/// `fetch_file` の path を `Mutex<Option<String>>` で記録し、
/// 戻り値（Ok/Err）を切り替えられる。
struct MockHostClient {
    fetch_file_result: MockFileResult,
    last_fetch_path: Mutex<Option<String>>,
}

impl MockHostClient {
    fn with_body(body: &str) -> Self {
        Self {
            fetch_file_result: MockFileResult::Ok(body.to_string()),
            last_fetch_path: Mutex::new(None),
        }
    }

    fn with_error(status: u16, message: &str) -> Self {
        Self {
            fetch_file_result: MockFileResult::Err {
                status,
                message: message.to_string(),
            },
            last_fetch_path: Mutex::new(None),
        }
    }

    fn last_path(&self) -> Option<String> {
        self.last_fetch_path.lock().unwrap().clone()
    }
}

impl HostClient for MockHostClient {
    fn get_default_branch<'a>(
        &'a self,
        _repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async { Ok("main".to_string()) })
    }

    fn get_commit_sha<'a>(
        &'a self,
        _repo: &'a Repo,
        _git_ref: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async { Ok("abc123".to_string()) })
    }

    fn download_archive<'a>(
        &'a self,
        _repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn download_archive_with_sha<'a>(
        &'a self,
        _repo: &'a Repo,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>> {
        Box::pin(async { Ok((Vec::new(), "main".to_string(), "abc123".to_string())) })
    }

    fn fetch_file<'a>(
        &'a self,
        _repo: &'a Repo,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        *self.last_fetch_path.lock().unwrap() = Some(path.to_string());
        let result = match &self.fetch_file_result {
            MockFileResult::Ok(body) => Ok(body.clone()),
            MockFileResult::Err { status, message } => Err(PlmError::RepoApi {
                url: "https://api.github.com/test".to_string(),
                status: *status,
                message: message.clone(),
            }),
        };
        Box::pin(async move { result })
    }
}

fn sample_repo() -> Repo {
    Repo::new(HostKind::GitHub, "acme", "catalog", None)
}

fn sample_manifest() -> MarketplaceManifest {
    MarketplaceManifest {
        name: "catalog".to_string(),
        owner: Some(MarketplaceOwner {
            name: "Acme".to_string(),
            email: Some("hello@acme.test".to_string()),
        }),
        plugins: vec![MarketplacePlugin {
            name: "plugin-a".to_string(),
            source: PluginSource::Local("./plugins/plugin-a".to_string()),
            description: Some("A test plugin".to_string()),
            version: Some("0.1.0".to_string()),
        }],
    }
}

fn sample_manifest_json() -> String {
    r#"{
      "name": "catalog",
      "owner": {"name": "Acme", "email": "hello@acme.test"},
      "plugins": [
        {
          "name": "plugin-a",
          "source": "./plugins/plugin-a",
          "description": "A test plugin",
          "version": "0.1.0"
        }
      ]
    }"#
    .to_string()
}

// ---- MarketplaceCache::from_manifest ----

#[test]
fn from_manifest_sets_source_as_github_owner_name() {
    let cache = MarketplaceCache::from_manifest(sample_manifest(), "catalog", &sample_repo());
    assert_eq!(cache.source, "github:acme/catalog");
}

#[test]
fn from_manifest_propagates_owner_and_plugins() {
    let manifest = sample_manifest();
    let expected_owner_name = manifest.owner.as_ref().map(|o| o.name.clone());
    let expected_plugin_name = manifest.plugins[0].name.clone();

    let cache = MarketplaceCache::from_manifest(manifest, "catalog", &sample_repo());

    assert_eq!(
        cache.owner.as_ref().map(|o| o.name.clone()),
        expected_owner_name
    );
    assert_eq!(cache.plugins.len(), 1);
    assert_eq!(cache.plugins[0].name, expected_plugin_name);
}

#[test]
fn from_manifest_sets_fetched_at_near_now() {
    let before = Utc::now();
    let cache = MarketplaceCache::from_manifest(sample_manifest(), "catalog", &sample_repo());
    let after = Utc::now();

    assert!(
        cache.fetched_at >= before && cache.fetched_at <= after,
        "fetched_at {} should be between {} and {}",
        cache.fetched_at,
        before,
        after
    );
}

#[test]
fn from_manifest_sets_name_from_argument() {
    let cache =
        MarketplaceCache::from_manifest(sample_manifest(), "my-custom-name", &sample_repo());
    assert_eq!(cache.name, "my-custom-name");
}

// ---- MarketplaceRegistry::fetch_cache ----

fn temp_registry() -> (MarketplaceRegistry, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let registry =
        MarketplaceRegistry::with_cache_dir(tmp.path().to_path_buf()).expect("registry init");
    (registry, tmp)
}

#[tokio::test]
async fn fetch_cache_happy_path_with_none_source_path() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    let cache = registry
        .fetch_cache(&client, "catalog", &sample_repo(), None)
        .await
        .expect("fetch_cache should succeed");

    assert_eq!(cache.name, "catalog");
    assert_eq!(cache.source, "github:acme/catalog");
    assert_eq!(
        client.last_path().as_deref(),
        Some(".claude-plugin/marketplace.json")
    );
}

#[tokio::test]
async fn fetch_cache_uses_subdir_path() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    registry
        .fetch_cache(&client, "catalog", &sample_repo(), Some("subdir"))
        .await
        .expect("fetch_cache should succeed");

    assert_eq!(
        client.last_path().as_deref(),
        Some("subdir/.claude-plugin/marketplace.json")
    );
}

#[tokio::test]
async fn fetch_cache_propagates_owner_and_plugins_into_cache() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    let cache = registry
        .fetch_cache(&client, "catalog", &sample_repo(), None)
        .await
        .expect("fetch_cache should succeed");

    let owner = cache.owner.expect("owner should be propagated");
    assert_eq!(owner.name, "Acme");
    assert_eq!(owner.email.as_deref(), Some("hello@acme.test"));

    assert_eq!(cache.plugins.len(), 1);
    assert_eq!(cache.plugins[0].name, "plugin-a");
}

#[tokio::test]
async fn fetch_cache_returns_invalid_manifest_on_malformed_json() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body("{not json");

    let err = registry
        .fetch_cache(&client, "catalog", &sample_repo(), None)
        .await
        .expect_err("malformed JSON should fail");

    match err {
        PlmError::InvalidManifest(msg) => {
            assert!(
                msg.contains("Failed to parse marketplace.json"),
                "expected message to contain context, got: {msg}"
            );
        }
        other => panic!("expected InvalidManifest, got: {other:?}"),
    }
}

#[tokio::test]
async fn fetch_cache_propagates_host_client_error() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_error(404, "not found");

    let err = registry
        .fetch_cache(&client, "catalog", &sample_repo(), None)
        .await
        .expect_err("host client error should propagate");

    match err {
        PlmError::RepoApi { status, .. } => assert_eq!(status, 404),
        other => panic!("expected RepoApi, got: {other:?}"),
    }
}

#[tokio::test]
async fn fetch_cache_does_not_persist_anything() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    let _cache = registry
        .fetch_cache(&client, "catalog", &sample_repo(), None)
        .await
        .expect("fetch_cache should succeed");

    assert!(
        registry.get("catalog").unwrap().is_none(),
        "fetch_cache must not persist the cache"
    );
}

#[tokio::test]
async fn fetch_cache_with_empty_source_path_preserves_legacy_path() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    registry
        .fetch_cache(&client, "catalog", &sample_repo(), Some(""))
        .await
        .expect("fetch_cache should succeed");

    assert_eq!(
        client.last_path().as_deref(),
        Some("/.claude-plugin/marketplace.json")
    );
}

#[tokio::test]
async fn fetch_cache_with_trailing_slash_preserves_legacy_path() {
    let (registry, _tmp) = temp_registry();
    let client = MockHostClient::with_body(&sample_manifest_json());

    registry
        .fetch_cache(&client, "catalog", &sample_repo(), Some("subdir/"))
        .await
        .expect("fetch_cache should succeed");

    assert_eq!(
        client.last_path().as_deref(),
        Some("subdir//.claude-plugin/marketplace.json")
    );
}

#[test]
fn parse_legacy_cache_with_original_manifest_ignores_unknown_field() {
    // 旧キャッシュの "original_manifest" キーは unknown field として無視される
    let json = r#"{
        "name": "legacy",
        "fetched_at": "2025-01-15T10:30:00Z",
        "source": "github:o/n",
        "owner": null,
        "plugins": [],
        "original_manifest": { "name": "x", "owner": null, "plugins": [] }
    }"#;
    let cache: MarketplaceCache = serde_json::from_str(json).expect("parse must not fail");
    assert_eq!(cache.name, "legacy");
    assert_eq!(cache.source, "github:o/n");
    assert!(cache.plugins.is_empty());
}
