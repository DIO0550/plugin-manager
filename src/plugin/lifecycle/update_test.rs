use super::*;

#[test]
fn test_restore_repo_from_source_repo() {
    let mut meta = PluginMeta::default();
    meta.set_source_repo("owner", "repo");

    let repo = restore_repo(&meta, "any-name", "main").unwrap();
    assert_eq!(repo.owner(), "owner");
    assert_eq!(repo.name(), "repo");
    assert_eq!(repo.git_ref(), Some("main"));
}

#[test]
fn test_restore_repo_from_plugin_name() {
    let meta = PluginMeta::default();

    let repo = restore_repo(&meta, "owner--repo", "main").unwrap();
    assert_eq!(repo.owner(), "owner");
    assert_eq!(repo.name(), "repo");
    assert_eq!(repo.git_ref(), Some("main"));
}

#[test]
fn test_restore_repo_invalid_name() {
    let meta = PluginMeta::default();

    let result = restore_repo(&meta, "invalid-name", "main");
    assert!(result.is_err());
}

#[test]
fn test_update_outcome_factories() {
    let updated = UpdateOutcome::updated(
        "test",
        Some("abc123".to_string()),
        "def456".to_string(),
        vec!["codex".to_string()],
        vec![],
    );
    assert!(matches!(updated.status, UpdateStatus::Updated { .. }));

    let up_to_date = UpdateOutcome::up_to_date("test");
    assert!(matches!(up_to_date.status, UpdateStatus::AlreadyUpToDate));

    let failed = UpdateOutcome::failed("test", "error".to_string());
    assert!(matches!(failed.status, UpdateStatus::Failed));

    let skipped = UpdateOutcome::skipped("test", "reason".to_string());
    assert!(matches!(skipped.status, UpdateStatus::Skipped { .. }));
}

#[test]
fn test_rolled_back_factory_without_note() {
    let outcome = UpdateOutcome::rolled_back("test", None);
    assert!(matches!(outcome.status, UpdateStatus::RolledBack));
    assert_eq!(outcome.plugin_name, "test");
    assert_eq!(outcome.error, None);
    assert!(outcome.deployed_targets.is_empty());
    assert!(outcome.failed_targets.is_empty());
}

#[test]
fn test_rolled_back_factory_with_note() {
    let outcome = UpdateOutcome::rolled_back("test", Some("WARNING: restore failed".to_string()));
    assert!(matches!(outcome.status, UpdateStatus::RolledBack));
    assert_eq!(outcome.error.as_deref(), Some("WARNING: restore failed"));
}

// =============================================================================
// バッチアトミック (update_all_plugins_with_deps) テスト
// =============================================================================

mod batch {
    use super::super::*;
    use crate::plugin::PackageCache;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::future::Future;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    const OLD_SHA: &str = "oldsha";

    /// zip アーカイブを生成（GitHub 形式の prefix 付き）
    fn make_zip(entries: &[(&str, &str)]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default();
            for (path, content) in entries {
                zip.start_file(*path, options).unwrap();
                zip.write_all(content.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        buf
    }

    /// 有効な更新後アーカイブ（plugin.json + data.txt="v2"）
    fn valid_archive(name: &str) -> Vec<u8> {
        let manifest = format!(r#"{{"name":"{}","version":"2.0.0"}}"#, name);
        make_zip(&[
            ("pkg-main/plugin.json", manifest.as_str()),
            ("pkg-main/data.txt", "v2"),
        ])
    }

    /// plugin.json を含まない無効アーカイブ
    fn invalid_archive() -> Vec<u8> {
        make_zip(&[("pkg-main/readme.md", "no manifest here")])
    }

    /// テスト用プラグインをキャッシュにセットアップする。
    ///
    /// `cache_id` = `name` とし、`sourceRepo = "owner/{name}"`（直接 GitHub 扱い）。
    /// data.txt は "v1"、commitSha は `installed_sha`。
    fn setup_plugin(cache_dir: &Path, cache_id: &str, installed_sha: &str, enabled: &[&str]) {
        let dir = cache_dir.join("github").join(cache_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"name":"{}","version":"1.0.0"}}"#, cache_id),
        )
        .unwrap();
        fs::write(dir.join("data.txt"), "v1").unwrap();

        let mut status = HashMap::new();
        for t in enabled {
            status.insert(t.to_string(), "enabled");
        }
        let status_json = if status.is_empty() {
            String::new()
        } else {
            let entries: Vec<String> = status
                .iter()
                .map(|(k, v)| format!(r#""{}":"{}""#, k, v))
                .collect();
            format!(r#","statusByTarget":{{{}}}"#, entries.join(","))
        };
        let meta = format!(
            r#"{{"gitRef":"main","commitSha":"{}","sourceRepo":"owner/{}"{}}}"#,
            installed_sha, cache_id, status_json
        );
        fs::write(dir.join(".plm-meta.json"), meta).unwrap();
    }

    fn read_data_in(cache_dir: &Path, mp_dir: &str, cache_id: &str) -> String {
        fs::read_to_string(cache_dir.join(mp_dir).join(cache_id).join("data.txt"))
            .unwrap_or_default()
    }

    fn read_data(cache_dir: &Path, cache_id: &str) -> String {
        read_data_in(cache_dir, "github", cache_id)
    }

    fn read_commit_sha(cache_dir: &Path, cache_id: &str) -> Option<String> {
        let path = cache_dir
            .join("github")
            .join(cache_id)
            .join(".plm-meta.json");
        let content = fs::read_to_string(path).ok()?;
        let v: serde_json::Value = serde_json::from_str(&content).ok()?;
        v.get("commitSha")
            .and_then(|s| s.as_str())
            .map(String::from)
    }

    fn backup_exists(cache_dir: &Path, cache_id: &str) -> bool {
        cache_dir
            .join(".backup")
            .join("github")
            .join(cache_id)
            .exists()
    }

    fn temp_exists(cache_dir: &Path, cache_id: &str) -> bool {
        temp_exists_in(cache_dir, "github", cache_id)
    }

    fn temp_exists_in(cache_dir: &Path, mp_dir: &str, cache_id: &str) -> bool {
        cache_dir.join(".temp").join(mp_dir).join(cache_id).exists()
    }

    /// 直接 GitHub プラグインを、manifest 名・sourceRepo を明示指定してセットアップする。
    ///
    /// `source_repo`（"owner/name" 形式）から `restore_repo` が `repo.name()` を導出するため、
    /// MockBatchClient のキー（repo.name()）を制御したい場合に使う。
    fn setup_plugin_named(
        cache_dir: &Path,
        cache_id: &str,
        display_name: &str,
        source_repo: &str,
        installed_sha: &str,
    ) {
        let dir = cache_dir.join("github").join(cache_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"name":"{}","version":"1.0.0"}}"#, display_name),
        )
        .unwrap();
        fs::write(dir.join("data.txt"), "v1").unwrap();
        fs::write(
            dir.join(".plm-meta.json"),
            format!(
                r#"{{"gitRef":"main","commitSha":"{}","sourceRepo":"{}"}}"#,
                installed_sha, source_repo
            ),
        )
        .unwrap();
    }

    fn find<'a>(results: &'a [UpdateOutcome], name: &str) -> &'a UpdateOutcome {
        results
            .iter()
            .find(|r| r.plugin_name == name)
            .unwrap_or_else(|| panic!("outcome for '{}' not found", name))
    }

    /// marketplace 配下に外部 GitHub プラグインをセットアップする。
    ///
    /// `mp_dir/cache_id` に plugin.json（manifest 名 = `display_name`）+ data.txt="v1" + meta。
    /// meta は `commitSha`/`gitRef` のみ（marketplace 経路は `sourceRepo` を使わない）。
    fn setup_mp_plugin(
        cache_dir: &Path,
        mp_dir: &str,
        cache_id: &str,
        display_name: &str,
        installed_sha: &str,
    ) {
        let dir = cache_dir.join(mp_dir).join(cache_id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"name":"{}","version":"1.0.0"}}"#, display_name),
        )
        .unwrap();
        fs::write(dir.join("data.txt"), "v1").unwrap();
        fs::write(
            dir.join(".plm-meta.json"),
            format!(
                r#"{{"gitRef":"main","commitSha":"{}","marketplace":"{}"}}"#,
                installed_sha, mp_dir
            ),
        )
        .unwrap();
    }

    /// 更新対象がない marketplace stub（直接 GitHub 経路のみ使うテスト用）
    struct NoMarketplaces;
    impl MarketplaceResolver for NoMarketplaces {
        fn resolve(&self, _marketplace: &str) -> Result<Option<MarketplaceCache>> {
            Ok(None)
        }
    }

    /// 固定の MarketplaceCache を返す resolver stub。
    ///
    /// 各 entry は External GitHub（`repo` = `owner/{repo_name}`）として登録する。
    /// `repo_name` は MockBatchClient のキー（repo.name()）と一致させる。
    struct WithMarketplace {
        market: String,
        entries: Vec<(String, String)>, // (cache_id, repo_name)
    }
    impl MarketplaceResolver for WithMarketplace {
        fn resolve(&self, marketplace: &str) -> Result<Option<MarketplaceCache>> {
            if marketplace != self.market {
                return Ok(None);
            }
            let plugins = self
                .entries
                .iter()
                .map(
                    |(cache_id, repo_name)| crate::marketplace::MarketplacePlugin {
                        name: cache_id.clone(),
                        source: crate::marketplace::PluginSource::External {
                            source: "github".to_string(),
                            repo: format!("owner/{}", repo_name),
                        },
                        description: None,
                        version: None,
                    },
                )
                .collect();
            Ok(Some(MarketplaceCache {
                name: self.market.clone(),
                fetched_at: chrono::Utc::now(),
                source: format!("github:owner/{}", self.market).parse().unwrap(),
                owner: None,
                plugins,
            }))
        }
    }

    #[derive(Clone)]
    enum Download {
        Valid,
        Invalid,
        Fail,
    }

    /// repo.name() をキーに挙動を切り替えるモッククライアント
    struct MockBatchClient {
        /// name -> get_commit_sha が返す SHA（未指定は "REMOTE_SHA"）
        latest: HashMap<String, String>,
        /// get_commit_sha が Err を返す name 集合
        sha_fail: HashSet<String>,
        /// name -> download 挙動（未指定は Valid）
        download: HashMap<String, Download>,
    }

    impl MockBatchClient {
        fn new() -> Self {
            Self {
                latest: HashMap::new(),
                sha_fail: HashSet::new(),
                download: HashMap::new(),
            }
        }
    }

    impl HostClient for MockBatchClient {
        fn get_default_branch<'a>(
            &'a self,
            _repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            Box::pin(async { Ok("main".to_string()) })
        }

        fn get_commit_sha<'a>(
            &'a self,
            repo: &'a Repo,
            _git_ref: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            let name = repo.name().to_string();
            let result = if self.sha_fail.contains(&name) {
                Err(PlmError::RepoApi {
                    url: "https://api.github.com/test".to_string(),
                    status: 500,
                    message: "injected sha failure".to_string(),
                })
            } else {
                Ok(self
                    .latest
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| "REMOTE_SHA".to_string()))
            };
            Box::pin(async move { result })
        }

        fn download_archive<'a>(
            &'a self,
            _repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn download_archive_with_sha<'a>(
            &'a self,
            repo: &'a Repo,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>> {
            let name = repo.name().to_string();
            let behavior = self.download.get(&name).cloned().unwrap_or(Download::Valid);
            let result = match behavior {
                Download::Valid => Ok((
                    valid_archive(&name),
                    "main".to_string(),
                    format!("commit-{}", name),
                )),
                Download::Invalid => Ok((invalid_archive(), "main".to_string(), "x".to_string())),
                Download::Fail => Err(PlmError::RepoApi {
                    url: "https://api.github.com/test".to_string(),
                    status: 500,
                    message: "injected download failure".to_string(),
                }),
            };
            Box::pin(async move { result })
        }

        fn fetch_file<'a>(
            &'a self,
            _repo: &'a Repo,
            _path: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            Box::pin(async { Ok(String::new()) })
        }
    }

    /// commit_staged / restore に障害を注入できるキャッシュデコレータ
    ///
    /// `fail_commit` / `fail_restore` は cache_id（name）一致で発火（github 単一 marketplace 用）。
    /// `fail_commit_keyed` / `fail_restore_keyed` は `"{mp}/{cache_id}"` キー一致で発火し、
    /// 別 marketplace 間で同名 cache_id を区別する必要があるテスト（W-002）で使う。
    struct FaultyCache {
        inner: PackageCache,
        fail_commit: HashSet<String>,
        fail_restore: HashSet<String>,
        fail_commit_keyed: HashSet<String>,
        fail_restore_keyed: HashSet<String>,
        /// N 番目（1 始まり）の commit_staged 呼び出しを失敗させる（0 = 無効）。
        /// cache.list() の順序が非決定的なテストで「先に swap 成功した 1 件＋次で失敗」を
        /// プラグイン名に依存せず再現するために使う。
        fail_commit_nth: usize,
        commit_calls: AtomicUsize,
    }

    impl FaultyCache {
        fn new(cache_dir: PathBuf) -> Self {
            Self {
                inner: PackageCache::with_cache_dir(cache_dir).unwrap(),
                fail_commit: HashSet::new(),
                fail_restore: HashSet::new(),
                fail_commit_keyed: HashSet::new(),
                fail_restore_keyed: HashSet::new(),
                fail_commit_nth: 0,
                commit_calls: AtomicUsize::new(0),
            }
        }

        fn key(marketplace: Option<&str>, name: &str) -> String {
            format!("{}/{}", marketplace.unwrap_or("github"), name)
        }
    }

    impl PackageCacheAccess for FaultyCache {
        fn plugin_path(&self, marketplace: Option<&str>, name: &str) -> PathBuf {
            self.inner.plugin_path(marketplace, name)
        }
        fn is_cached(&self, marketplace: Option<&str>, name: &str) -> bool {
            self.inner.is_cached(marketplace, name)
        }
        fn store_from_archive(
            &self,
            marketplace: Option<&str>,
            name: &str,
            archive: &[u8],
            source_path: Option<&str>,
        ) -> Result<PathBuf> {
            self.inner
                .store_from_archive(marketplace, name, archive, source_path)
        }
        fn load_manifest(
            &self,
            marketplace: Option<&str>,
            name: &str,
        ) -> Result<crate::plugin::PluginManifest> {
            self.inner.load_manifest(marketplace, name)
        }
        fn remove(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
            self.inner.remove(marketplace, name)
        }
        fn list(&self) -> Result<Vec<(Option<String>, String)>> {
            self.inner.list()
        }
        fn backup(&self, marketplace: Option<&str>, name: &str) -> Result<PathBuf> {
            self.inner.backup(marketplace, name)
        }
        fn restore(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
            if self.fail_restore.contains(name)
                || self
                    .fail_restore_keyed
                    .contains(&Self::key(marketplace, name))
            {
                return Err(PlmError::Cache("injected restore failure".to_string()));
            }
            self.inner.restore(marketplace, name)
        }
        fn remove_backup(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
            self.inner.remove_backup(marketplace, name)
        }
        fn atomic_update(
            &self,
            marketplace: Option<&str>,
            name: &str,
            archive: &[u8],
        ) -> Result<PathBuf> {
            self.inner.atomic_update(marketplace, name, archive)
        }
        fn atomic_update_with_source_path(
            &self,
            marketplace: Option<&str>,
            name: &str,
            archive: &[u8],
            source_path: Option<&str>,
        ) -> Result<PathBuf> {
            self.inner
                .atomic_update_with_source_path(marketplace, name, archive, source_path)
        }
        fn stage_from_archive(
            &self,
            marketplace: Option<&str>,
            name: &str,
            archive: &[u8],
            source_path: Option<&str>,
        ) -> Result<PathBuf> {
            self.inner
                .stage_from_archive(marketplace, name, archive, source_path)
        }
        fn commit_staged(&self, marketplace: Option<&str>, name: &str) -> Result<PathBuf> {
            let call = self.commit_calls.fetch_add(1, Ordering::SeqCst) + 1;
            if self.fail_commit.contains(name)
                || self
                    .fail_commit_keyed
                    .contains(&Self::key(marketplace, name))
                || (self.fail_commit_nth != 0 && self.fail_commit_nth == call)
            {
                return Err(PlmError::Cache("injected commit failure".to_string()));
            }
            self.inner.commit_staged(marketplace, name)
        }
        fn discard_staged(&self, marketplace: Option<&str>, name: &str) -> Result<()> {
            self.inner.discard_staged(marketplace, name)
        }
        fn has_marketplace_entry(&self, marketplace: &str, entry: &str) -> Result<bool> {
            self.inner.has_marketplace_entry(marketplace, entry)
        }
        fn remove_marketplace_entry(&self, marketplace: &str, entry: &str) -> Result<()> {
            self.inner.remove_marketplace_entry(marketplace, entry)
        }
        fn list_marketplace_entries(&self, marketplace: &str) -> Result<Vec<String>> {
            self.inner.list_marketplace_entries(marketplace)
        }
    }

    fn count_status(results: &[UpdateOutcome], pred: impl Fn(&UpdateStatus) -> bool) -> usize {
        results.iter().filter(|r| pred(&r.status)).count()
    }

    // ---- 正常系 ----

    #[tokio::test]
    async fn test_single_update_commits() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        setup_plugin(&cache_dir, "repoA", OLD_SHA, &[]);
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 1);
        let a = find(&results, "repoA");
        assert!(matches!(a.status, UpdateStatus::Updated { .. }));
        assert_eq!(read_data(&cache_dir, "repoA"), "v2");
        assert_eq!(
            read_commit_sha(&cache_dir, "repoA").as_deref(),
            Some("commit-repoA")
        );
        assert!(!backup_exists(&cache_dir, "repoA"));
        // staging temp が commit_staged で消費されている（rename→copy 退行検出）
        assert!(!temp_exists(&cache_dir, "repoA"));
    }

    #[tokio::test]
    async fn test_multiple_updates_all_commit() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB", "repoC"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 3);
        assert_eq!(
            count_status(&results, |s| matches!(s, UpdateStatus::Updated { .. })),
            3
        );
        for id in ["repoA", "repoB", "repoC"] {
            assert_eq!(read_data(&cache_dir, id), "v2");
            assert!(
                !backup_exists(&cache_dir, id),
                "backup for {} should be gone",
                id
            );
            // staging temp が全件 commit_staged で消費されている
            assert!(!temp_exists(&cache_dir, id), "temp for {} remained", id);
        }
    }

    #[tokio::test]
    async fn test_zero_updates_keeps_production() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        // installed sha == remote sha → up to date
        setup_plugin(&cache_dir, "repoA", "REMOTE_SHA", &[]);
        setup_plugin(&cache_dir, "repoB", "REMOTE_SHA", &[]);
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 2);
        assert_eq!(
            count_status(&results, |s| matches!(s, UpdateStatus::AlreadyUpToDate)),
            2
        );
        // 本番不変・backup なし
        for id in ["repoA", "repoB"] {
            assert_eq!(read_data(&cache_dir, id), "v1");
            assert!(!backup_exists(&cache_dir, id));
        }
    }

    // ---- prepare 失敗の全件ロールバック ----

    #[tokio::test]
    async fn test_download_failure_rolls_back_all() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB", "repoC"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let mut client = MockBatchClient::new();
        client.download.insert("repoB".to_string(), Download::Fail);
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        // 全件が結果に出る（review B: 未到達も RolledBack）
        assert_eq!(results.len(), 3);
        assert!(matches!(
            find(&results, "repoB").status,
            UpdateStatus::Failed
        ));
        assert!(matches!(
            find(&results, "repoA").status,
            UpdateStatus::RolledBack
        ));
        assert!(matches!(
            find(&results, "repoC").status,
            UpdateStatus::RolledBack
        ));
        // 本番は全件更新前のまま、staging/backup 全削除
        for id in ["repoA", "repoB", "repoC"] {
            assert_eq!(read_data(&cache_dir, id), "v1");
            assert_eq!(read_commit_sha(&cache_dir, id).as_deref(), Some(OLD_SHA));
            assert!(!backup_exists(&cache_dir, id), "backup {} remained", id);
            assert!(!temp_exists(&cache_dir, id), "temp {} remained", id);
        }
        // 整合性: 件数保存（check失敗なし → up_to_date(0)+skipped(0)+update_count(3)）
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_staging_validation_failure_rolls_back_all() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB", "repoC"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let mut client = MockBatchClient::new();
        client
            .download
            .insert("repoB".to_string(), Download::Invalid);
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 3);
        assert!(matches!(
            find(&results, "repoB").status,
            UpdateStatus::Failed
        ));
        for id in ["repoA", "repoC"] {
            assert!(matches!(
                find(&results, id).status,
                UpdateStatus::RolledBack
            ));
        }
        for id in ["repoA", "repoB", "repoC"] {
            assert_eq!(read_data(&cache_dir, id), "v1");
            assert!(!backup_exists(&cache_dir, id));
            assert!(!temp_exists(&cache_dir, id));
        }
    }

    // ---- commit(swap) 失敗の全件 restore ----

    #[tokio::test]
    async fn test_commit_failure_restores_all() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB", "repoC"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let mut cache = FaultyCache::new(cache_dir.clone());
        cache.fail_commit.insert("repoB".to_string());
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 3);
        // 当該 repoB は Failed ちょうど 1 件、RolledBack で二重計上されない（review A）
        let repo_b_outcomes: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| r.plugin_name == "repoB")
            .collect();
        assert_eq!(repo_b_outcomes.len(), 1);
        assert!(matches!(repo_b_outcomes[0].status, UpdateStatus::Failed));
        // 他は RolledBack
        assert!(matches!(
            find(&results, "repoA").status,
            UpdateStatus::RolledBack
        ));
        assert!(matches!(
            find(&results, "repoC").status,
            UpdateStatus::RolledBack
        ));
        // 全件が更新前状態に復元
        for id in ["repoA", "repoB", "repoC"] {
            assert_eq!(read_data(&cache_dir, id), "v1", "{} not restored", id);
            assert_eq!(read_commit_sha(&cache_dir, id).as_deref(), Some(OLD_SHA));
        }
    }

    #[tokio::test]
    async fn test_single_commit_failure_restores() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        setup_plugin(&cache_dir, "repoA", OLD_SHA, &[]);
        let mut cache = FaultyCache::new(cache_dir.clone());
        cache.fail_commit.insert("repoA".to_string());
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 1);
        assert!(matches!(
            find(&results, "repoA").status,
            UpdateStatus::Failed
        ));
        assert_eq!(read_data(&cache_dir, "repoA"), "v1");
        assert_eq!(
            read_commit_sha(&cache_dir, "repoA").as_deref(),
            Some(OLD_SHA)
        );
    }

    // ---- エッジケース: restore 自体の失敗 ----

    #[tokio::test]
    async fn test_rollback_restore_failure_warns() {
        // swap 済み（RolledBack 予定）のプラグインの restore が失敗したら警告が outcome に載る。
        // cache.list() の順序は非決定的なため、「2 番目の swap を失敗」させることで
        // 「1 件目=swap 成功（後で restore 失敗）/ 2 件目=swap 失敗」をプラグイン名に依存せず再現する。
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let mut cache = FaultyCache::new(cache_dir.clone());
        cache.fail_commit_nth = 2; // 2 番目の swap を失敗 → 1 番目は swap 済み
        cache.fail_restore.insert("repoA".to_string());
        cache.fail_restore.insert("repoB".to_string()); // どちらが swap 済みでも restore 失敗
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 2);
        // 1 件目（swap 済み）→ RolledBack + restore 失敗の警告
        let rolled: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| matches!(r.status, UpdateStatus::RolledBack))
            .collect();
        assert_eq!(rolled.len(), 1);
        assert!(
            rolled[0]
                .error
                .as_deref()
                .unwrap_or("")
                .contains("restore failed"),
            "expected restore warning on rolled-back plugin, got {:?}",
            rolled[0].error
        );
        // 2 件目（swap 失敗）→ Failed（restore も失敗するため併記される）
        let failed: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| matches!(r.status, UpdateStatus::Failed))
            .collect();
        assert_eq!(failed.len(), 1);
        assert!(
            failed[0]
                .error
                .as_deref()
                .unwrap_or("")
                .contains("restore also failed"),
            "expected combined warning on failed plugin, got {:?}",
            failed[0].error
        );
    }

    #[tokio::test]
    async fn test_rollback_skips_restore_for_unswapped_plugins() {
        // review 指摘: swap 未実施（idx > failed_idx）のプラグインには restore を呼ばない。
        // 2 番目の swap を失敗させ、3 番目（未 swap）の restore を失敗注入しても、
        // restore は呼ばれないため警告は出ず、本番は無改変（v1）のまま RolledBack になる。
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        for id in ["repoA", "repoB", "repoC"] {
            setup_plugin(&cache_dir, id, OLD_SHA, &[]);
        }
        let mut cache = FaultyCache::new(cache_dir.clone());
        cache.fail_commit_nth = 2; // 1 件目 swap 成功 / 2 件目 swap 失敗 / 3 件目 未 swap
                                   // 全件の restore を失敗注入する。未 swap の 3 件目で restore が呼ばれていれば
                                   // 警告が載るが、呼ばれない設計なので 1 件は警告なし RolledBack になるはず。
        for id in ["repoA", "repoB", "repoC"] {
            cache.fail_restore.insert(id.to_string());
        }
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 3);
        // Failed は 1 件（2 件目）
        assert_eq!(
            count_status(&results, |s| matches!(s, UpdateStatus::Failed)),
            1
        );
        // RolledBack は 2 件。うち未 swap の 1 件は restore 未実行なので警告なし。
        let rolled: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| matches!(r.status, UpdateStatus::RolledBack))
            .collect();
        assert_eq!(rolled.len(), 2);
        let no_warning = rolled.iter().filter(|r| r.error.is_none()).count();
        assert_eq!(
            no_warning,
            1,
            "unswapped plugin must be RolledBack without a restore attempt/warning, got {:?}",
            rolled.iter().map(|r| &r.error).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_rollback_restore_failure_on_failed_target() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        setup_plugin(&cache_dir, "repoA", OLD_SHA, &[]);
        let mut cache = FaultyCache::new(cache_dir.clone());
        cache.fail_commit.insert("repoA".to_string());
        cache.fail_restore.insert("repoA".to_string());
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        let a = find(&results, "repoA");
        assert!(matches!(a.status, UpdateStatus::Failed));
        assert!(
            a.error
                .as_deref()
                .unwrap_or("")
                .contains("restore also failed"),
            "expected combined warning, got {:?}",
            a.error
        );
    }

    // ---- 回帰 ----

    #[tokio::test]
    async fn test_redeploy_failure_does_not_trigger_rollback() {
        // redeploy（enable_plugin）失敗は非アトミック: commit 済みは Updated を維持
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        setup_plugin(&cache_dir, "repoA", OLD_SHA, &["codex"]);
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let client = MockBatchClient::new();
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 1);
        // redeploy の成否に関わらず swap は確定し Updated（ロールバックしない）
        assert!(matches!(
            find(&results, "repoA").status,
            UpdateStatus::Updated { .. }
        ));
        assert_eq!(read_data(&cache_dir, "repoA"), "v2");
    }

    #[tokio::test]
    async fn test_sha_check_failure_is_skipped_others_proceed() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        setup_plugin(&cache_dir, "repoA", OLD_SHA, &[]);
        setup_plugin(&cache_dir, "repoB", OLD_SHA, &[]);
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let mut client = MockBatchClient::new();
        client.sha_fail.insert("repoB".to_string()); // CHECK で get_commit_sha 失敗
        let resolver = NoMarketplaces;

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        // repoB は error_count でスキップ → results に出ない。repoA のみ Updated。
        assert_eq!(results.len(), 1);
        assert!(matches!(
            find(&results, "repoA").status,
            UpdateStatus::Updated { .. }
        ));
        assert_eq!(read_data(&cache_dir, "repoA"), "v2");
        // repoB は本番未改変
        assert_eq!(read_data(&cache_dir, "repoB"), "v1");
        assert_eq!(
            read_commit_sha(&cache_dir, "repoB").as_deref(),
            Some(OLD_SHA)
        );
    }

    // ---- marketplace 経由経路 ----

    #[tokio::test]
    async fn test_marketplace_plugin_commits() {
        // marketplace 経由（market != "github"）の更新がコミットされる
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        // cache_id "plugM" を marketplace "mymarket" 配下に配置
        setup_mp_plugin(&cache_dir, "mymarket", "plugM", "plugM-disp", OLD_SHA);
        let cache = PackageCache::with_cache_dir(cache_dir.clone()).unwrap();
        let client = MockBatchClient::new();
        // entry: cache_id "plugM" → repo "owner/repoM"（repo.name() = "repoM" が mock キー）
        let resolver = WithMarketplace {
            market: "mymarket".to_string(),
            entries: vec![("plugM".to_string(), "repoM".to_string())],
        };

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 1);
        assert!(matches!(
            find(&results, "plugM-disp").status,
            UpdateStatus::Updated { .. }
        ));
        // 本番（mymarket 配下）が v2 に更新
        assert_eq!(read_data_in(&cache_dir, "mymarket", "plugM"), "v2");
        assert!(!temp_exists_in(&cache_dir, "mymarket", "plugM"));
    }

    // ---- 複数 marketplace 間の同名 cache_id 衝突（review A リグレッションガード）----

    #[tokio::test]
    async fn test_same_cache_id_across_marketplaces_commit_failure() {
        // github/foo と mymarket/foo（同名 cache_id）が併存し、mymarket/foo のみ commit 失敗。
        // 当該はちょうど 1 件 Failed、github/foo は RolledBack（cache_id 文字列衝突で
        // 二重計上されないこと＝index 識別の検証）。
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().to_path_buf();
        // 直接 GitHub: github/foo（display "foo-gh", sourceRepo owner/foo-gh → repo.name "foo-gh"）
        setup_plugin_named(&cache_dir, "foo", "foo-gh", "owner/foo-gh", OLD_SHA);
        // marketplace: mymarket/foo（display "foo-mp", entry repo "owner/foo-mp"）
        setup_mp_plugin(&cache_dir, "mymarket", "foo", "foo-mp", OLD_SHA);

        let mut cache = FaultyCache::new(cache_dir.clone());
        // mymarket/foo の commit のみ失敗させる（github/foo は成功させる）
        cache.fail_commit_keyed.insert("mymarket/foo".to_string());

        let client = MockBatchClient::new();
        let resolver = WithMarketplace {
            market: "mymarket".to_string(),
            entries: vec![("foo".to_string(), "foo-mp".to_string())],
        };

        let results =
            update_all_plugins_with_deps(&cache, &client, &resolver, tmp.path(), None).await;

        assert_eq!(results.len(), 2);
        // Failed はちょうど 1 件（mymarket/foo = "foo-mp"）。二重計上されない。
        let failed: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| matches!(r.status, UpdateStatus::Failed))
            .collect();
        assert_eq!(failed.len(), 1, "exactly one Failed expected");
        assert_eq!(failed[0].plugin_name, "foo-mp");
        // github/foo は RolledBack（更新前へ復元）
        let rolled: Vec<&UpdateOutcome> = results
            .iter()
            .filter(|r| matches!(r.status, UpdateStatus::RolledBack))
            .collect();
        assert_eq!(rolled.len(), 1);
        assert_eq!(rolled[0].plugin_name, "foo-gh");
        // 本番は両方更新前（v1）に復元
        assert_eq!(read_data_in(&cache_dir, "github", "foo"), "v1");
        assert_eq!(read_data_in(&cache_dir, "mymarket", "foo"), "v1");
    }
}
