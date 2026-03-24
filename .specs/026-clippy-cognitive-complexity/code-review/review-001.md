OpenAI Codex v0.115.0 (research preview)
--------
workdir: /workspace
model: gpt-5.4
provider: openai
approval: never
sandbox: danger-full-access
reasoning effort: high
reasoning summaries: none
session id: 019d1de6-f40c-7183-834b-1b8be6f3bd47
--------
user
以下のタスクの実装をレビューしてください。

【重要】ファイルの作成・編集は一切行わないでください。レビュー結果は標準出力のみで回答してください。

## 実装計画
clippy.toml を追加して Clippy の閾値をプロジェクト標準より厳しく設定し、既存の違反をリファクタリングで解消する。

## 対象タスク
1. clippy.toml 作成（4つの閾値設定）
2. too-many-arguments 違反の修正（構造体導入）
3. type-complexity 違反の修正（型エイリアス導入）
4. 既存 allow(clippy::...) アノテーションの削除

## 変更されたファイル
src/application/plugin_intent.rs
src/hooks/converter.rs
src/host.rs
src/plugin/update.rs
src/sync.rs
src/tui/manager/screens/installed/view.rs
src/tui/manager/screens/marketplaces/actions.rs
src/tui/manager/screens/marketplaces/update.rs
src/tui/manager/screens/marketplaces/update_test.rs
src/tui/manager/screens/marketplaces/view.rs

## 変更内容
diff --git a/src/application/plugin_intent.rs b/src/application/plugin_intent.rs
index a40cab5..edb5fa3 100644
--- a/src/application/plugin_intent.rs
+++ b/src/application/plugin_intent.rs
@@ -16,6 +16,10 @@ use crate::fs::{FileSystem, RealFs};
 use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
 use std::path::{Path, PathBuf};
 
+/// 単一コンポーネントの操作生成結果
+type CreateOperationResult =
+    std::result::Result<Option<(TargetId, FileOperation)>, (TargetId, String)>;
+
 /// `expand()` の結果
 #[derive(Debug)]
 pub struct ExpandResult {
@@ -135,7 +139,7 @@ impl PluginIntent {
         target: &dyn Target,
         component: &Component,
         origin: &PluginOrigin,
-    ) -> Result<Option<(TargetId, FileOperation)>, (TargetId, String)> {
+    ) -> CreateOperationResult {
         let context = PlacementContext {
             component: ComponentRef::new(component.kind, &component.name),
             origin,
diff --git a/src/hooks/converter.rs b/src/hooks/converter.rs
index f10d94b..f07676f 100644
--- a/src/hooks/converter.rs
+++ b/src/hooks/converter.rs
@@ -460,14 +460,12 @@ fn convert_hook_definition(
             Ok(Some(converted))
         }
         "prompt" | "agent" => {
-            let converted = convert_prompt_agent_hook(
-                hook,
-                hook_type,
-                event,
-                matcher,
+            let mut collector = ConversionCollector {
                 warnings,
                 wrapper_scripts,
-            );
+            };
+            let converted =
+                convert_prompt_agent_hook(hook, hook_type, event, matcher, &mut collector);
             Ok(Some(converted))
         }
         unknown => {
@@ -806,16 +804,26 @@ exit 0
     Ok(Value::Object(output))
 }
 
+/// Mutable collectors for warnings and wrapper scripts during conversion.
+struct ConversionCollector<'a> {
+    warnings: &'a mut Vec<ConversionWarning>,
+    wrapper_scripts: &'a mut Vec<WrapperScriptInfo>,
+}
+
 /// BL-006: Convert a prompt/agent hook to a stub script with warning.
 fn convert_prompt_agent_hook(
     hook: &Value,
     hook_type: &str,
     event: &str,
     matcher: Option<&str>,
-    warnings: &mut Vec<ConversionWarning>,
-    wrapper_scripts: &mut Vec<WrapperScriptInfo>,
+    collector: &mut ConversionCollector<'_>,
 ) -> Value {
-    let script_name = format!("{}-{}-{}.sh", hook_type, event, wrapper_scripts.len());
+    let script_name = format!(
+        "{}-{}-{}.sh",
+        hook_type,
+        event,
+        collector.wrapper_scripts.len()
+    );
 
     let original_json = serde_json::to_string_pretty(hook).unwrap_or_default();
 
@@ -831,17 +839,19 @@ fn convert_prompt_agent_hook(
         event
     );
 
-    wrapper_scripts.push(WrapperScriptInfo {
+    collector.wrapper_scripts.push(WrapperScriptInfo {
         path: format!("{}/{}", WRAPPERS_DIR, script_name),
         content: script_content,
         original_config: hook.clone(),
         matcher: matcher.map(|s| s.to_string()),
     });
 
-    warnings.push(ConversionWarning::PromptAgentHookStub {
-        event: event.to_string(),
-        hook_type: hook_type.to_string(),
-    });
+    collector
+        .warnings
+        .push(ConversionWarning::PromptAgentHookStub {
+            event: event.to_string(),
+            hook_type: hook_type.to_string(),
+        });
 
     let hook_obj = hook.as_object();
 
diff --git a/src/host.rs b/src/host.rs
index b3d3b9d..edf4c3d 100644
--- a/src/host.rs
+++ b/src/host.rs
@@ -12,6 +12,9 @@ use crate::repo::Repo;
 use std::future::Future;
 use std::pin::Pin;
 
+/// 非同期メソッドの戻り値型エイリアス
+type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
+
 /// ホスト種別
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
 pub enum HostKind {
@@ -38,39 +41,24 @@ impl std::fmt::Display for HostKind {
 }
 
 /// ホスト別クライアント trait
-#[allow(clippy::type_complexity)]
 pub trait HostClient: Send + Sync {
     /// デフォルトブランチを取得
-    fn get_default_branch<'a>(
-        &'a self,
-        repo: &'a Repo,
-    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
+    fn get_default_branch<'a>(&'a self, repo: &'a Repo) -> BoxFuture<'a, String>;
 
     /// コミットSHAを取得
-    fn get_commit_sha<'a>(
-        &'a self,
-        repo: &'a Repo,
-        git_ref: &'a str,
-    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
+    fn get_commit_sha<'a>(&'a self, repo: &'a Repo, git_ref: &'a str) -> BoxFuture<'a, String>;
 
     /// リポジトリをzipアーカイブとしてダウンロード
-    fn download_archive<'a>(
-        &'a self,
-        repo: &'a Repo,
-    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;
+    fn download_archive<'a>(&'a self, repo: &'a Repo) -> BoxFuture<'a, Vec<u8>>;
 
     /// リポジトリをダウンロードし、コミットSHAも一緒に返す
     fn download_archive_with_sha<'a>(
         &'a self,
         repo: &'a Repo,
-    ) -> Pin<Box<dyn Future<Output = Result<(Vec<u8>, String, String)>> + Send + 'a>>;
+    ) -> BoxFuture<'a, (Vec<u8>, String, String)>;
 
     /// リポジトリ内のファイルを取得
-    fn fetch_file<'a>(
-        &'a self,
-        repo: &'a Repo,
-        path: &'a str,
-    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
+    fn fetch_file<'a>(&'a self, repo: &'a Repo, path: &'a str) -> BoxFuture<'a, String>;
 }
 
 /// ホストクライアントファクトリー
diff --git a/src/plugin/update.rs b/src/plugin/update.rs
index 38c3cd1..709e3d0 100644
--- a/src/plugin/update.rs
+++ b/src/plugin/update.rs
@@ -188,74 +188,76 @@ pub async fn update_plugin(
     }
 
     // 更新処理実行
-    do_update(
-        plugin_name,
-        &latest_sha,
+    let ctx = UpdateCtx {
         cache,
-        &*client,
-        &repo,
-        &plugin_meta,
+        client: &*client,
+        repo: &repo,
+        plugin_meta: &plugin_meta,
         project_root,
         target_filter,
-    )
-    .await
+    };
+    do_update(plugin_name, &latest_sha, &ctx).await
+}
+
+/// 更新処理のコンテキスト
+struct UpdateCtx<'a> {
+    cache: &'a dyn PluginCacheAccess,
+    client: &'a dyn crate::host::HostClient,
+    repo: &'a Repo,
+    plugin_meta: &'a PluginMeta,
+    project_root: &'a Path,
+    target_filter: Option<&'a str>,
 }
 
 /// 更新処理の実行
-#[allow(clippy::too_many_arguments)]
-async fn do_update(
-    plugin_name: &str,
-    latest_sha: &str,
-    cache: &dyn PluginCacheAccess,
-    client: &dyn crate::host::HostClient,
-    repo: &Repo,
-    plugin_meta: &PluginMeta,
-    project_root: &Path,
-    target_filter: Option<&str>,
-) -> UpdateResult {
-    let current_sha = plugin_meta.commit_sha.clone();
-    let git_ref = plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
+async fn do_update(plugin_name: &str, latest_sha: &str, ctx: &UpdateCtx<'_>) -> UpdateResult {
+    let current_sha = ctx.plugin_meta.commit_sha.clone();
+    let git_ref = ctx.plugin_meta.git_ref.as_deref().unwrap_or("HEAD");
 
     // バックアップ作成
     println!("  Creating backup...");
-    if let Err(e) = cache.backup(Some("github"), plugin_name) {
+    if let Err(e) = ctx.cache.backup(Some("github"), plugin_name) {
         return UpdateResult::failed(plugin_name, format!("Backup failed: {}", e));
     }
 
     // ダウンロード（リトライ付き）
     println!("  Downloading...");
-    let archive = match with_retry(|| client.download_archive(repo), 3).await {
+    let archive = match with_retry(|| ctx.client.download_archive(ctx.repo), 3).await {
         Ok(a) => a,
         Err(e) => {
             // ロールバック
-            let _ = cache.restore(Some("github"), plugin_name);
+            let _ = ctx.cache.restore(Some("github"), plugin_name);
             return UpdateResult::failed(plugin_name, format!("Download failed: {}", e));
         }
     };
 
     // アトミック更新
     println!("  Extracting...");
-    let plugin_path = match cache.atomic_update(Some("github"), plugin_name, &archive) {
+    let plugin_path = match ctx
+        .cache
+        .atomic_update(Some("github"), plugin_name, &archive)
+    {
         Ok(p) => p,
         Err(e) => {
             // ロールバック
-            let _ = cache.restore(Some("github"), plugin_name);
+            let _ = ctx.cache.restore(Some("github"), plugin_name);
             return UpdateResult::failed(plugin_name, format!("Extraction failed: {}", e));
         }
     };
 
     // 再デプロイ
     println!("  Deploying...");
-    let enabled = plugin_meta.enabled_targets();
-    let targets: Vec<&str> = match target_filter {
+    let enabled = ctx.plugin_meta.enabled_targets();
+    let targets: Vec<&str> = match ctx.target_filter {
         Some(f) => enabled.into_iter().filter(|t| *t == f).collect(),
         None => enabled,
     };
 
-    let (deployed, failed) = redeploy_to_targets(cache, plugin_name, &targets, project_root);
+    let (deployed, failed) =
+        redeploy_to_targets(ctx.cache, plugin_name, &targets, ctx.project_root);
 
     // メタデータ更新
-    let mut new_meta = plugin_meta.clone();
+    let mut new_meta = ctx.plugin_meta.clone();
     new_meta.set_git_info(git_ref, latest_sha);
     for t in &failed {
         new_meta.set_status(t, "disabled");
@@ -265,7 +267,7 @@ async fn do_update(
     }
 
     // バックアップ削除
-    let _ = cache.remove_backup(Some("github"), plugin_name);
+    let _ = ctx.cache.remove_backup(Some("github"), plugin_name);
 
     UpdateResult::updated(
         plugin_name,
@@ -424,17 +426,15 @@ pub async fn update_all_plugins(
         let update_factory = HostClientFactory::with_defaults();
         let update_client = update_factory.create(HostKind::GitHub);
 
-        let result = do_update(
-            name,
-            latest_sha,
+        let update_ctx = UpdateCtx {
             cache,
-            &*update_client,
-            &repo,
-            meta,
+            client: &*update_client,
+            repo: &repo,
+            plugin_meta: meta,
             project_root,
             target_filter,
-        )
-        .await;
+        };
+        let result = do_update(name, latest_sha, &update_ctx).await;
 
         // 結果表示
         match &result.status {
diff --git a/src/sync.rs b/src/sync.rs
index 1c8b484..eb19631 100644
--- a/src/sync.rs
+++ b/src/sync.rs
@@ -115,7 +115,12 @@ pub(crate) fn sync_with_fs(
     }
 
     // 5. 実行
-    execute_sync(source, dest, to_create, to_update, to_delete, fs)
+    let plan = SyncPlan {
+        to_create,
+        to_update,
+        to_delete,
+    };
+    execute_sync(source, dest, plan, fs)
 }
 
 /// 更新が必要かを判定（mtime または内容比較）
@@ -141,19 +146,24 @@ fn needs_update(
     Ok(fs.content_hash(&src.path)? != fs.content_hash(&dest.path)?)
 }
 
+/// 同期の実行計画
+struct SyncPlan {
+    to_create: Vec<PlacedComponent>,
+    to_update: Vec<PlacedComponent>,
+    to_delete: Vec<PlacedComponent>,
+}
+
 /// 同期を実行
 fn execute_sync(
     source: &SyncSource,
     dest: &SyncDestination,
-    to_create: Vec<PlacedComponent>,
-    to_update: Vec<PlacedComponent>,
-    to_delete: Vec<PlacedComponent>,
+    plan: SyncPlan,
     fs: &dyn FileSystem,
 ) -> Result<SyncResult> {
     let mut result = SyncResult::default();
 
     // Create
-    for component in to_create {
+    for component in plan.to_create {
         match execute_create(source, dest, &component, fs) {
             Ok(()) => result.created.push(component),
             Err(e) => result.failed.push(SyncFailure::new(
@@ -165,7 +175,7 @@ fn execute_sync(
     }
 
     // Update
-    for component in to_update {
+    for component in plan.to_update {
         match execute_update(source, dest, &component, fs) {
             Ok(()) => result.updated.push(component),
             Err(e) => result.failed.push(SyncFailure::new(
@@ -177,7 +187,7 @@ fn execute_sync(
     }
 
     // Delete
-    for component in to_delete {
+    for component in plan.to_delete {
         match execute_delete(&component, fs) {
             Ok(()) => result.deleted.push(component),
             Err(e) => result.failed.push(SyncFailure::new(
diff --git a/src/tui/manager/screens/installed/view.rs b/src/tui/manager/screens/installed/view.rs
index afd1354..bb3d26f 100644
--- a/src/tui/manager/screens/installed/view.rs
+++ b/src/tui/manager/screens/installed/view.rs
@@ -11,6 +11,13 @@ use ratatui::prelude::*;
 use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
 use std::collections::{HashMap, HashSet};
 
+/// 描画用共通コンテキスト
+struct ViewCtx<'a> {
+    data: &'a DataStore,
+    filter_text: &'a str,
+    filter_focused: bool,
+}
+
 /// ComponentKind の表示用タイトルを取得（複数形）
 fn component_kind_title(kind: ComponentKind) -> &'static str {
     match kind {
@@ -30,6 +37,11 @@ pub fn view(
     filter_text: &str,
     filter_focused: bool,
 ) {
+    let ctx = ViewCtx {
+        data,
+        filter_text,
+        filter_focused,
+    };
     match model {
         Model::PluginList {
             state,
@@ -37,25 +49,17 @@ pub fn view(
             update_statuses,
             ..
         } => {
-            view_plugin_list(
-                f,
-                *state,
-                data,
-                filter_text,
-                filter_focused,
-                marked_ids,
-                update_statuses,
-            );
+            view_plugin_list(f, *state, &ctx, marked_ids, update_statuses);
         }
         Model::PluginDetail {
             plugin_id, state, ..
         } => {
-            view_plugin_detail(f, plugin_id, *state, data, filter_text, filter_focused);
+            view_plugin_detail(f, plugin_id, *state, &ctx);
         }
         Model::ComponentTypes {
             plugin_id, state, ..
         } => {
-            view_component_types(f, plugin_id, *state, data, filter_text, filter_focused);
+            view_component_types(f, plugin_id, *state, &ctx);
         }
         Model::ComponentList {
             plugin_id,
@@ -63,15 +67,7 @@ pub fn view(
             state,
             ..
         } => {
-            view_component_list(
-                f,
-                plugin_id,
-                *kind,
-                *state,
-                data,
-                filter_text,
-                filter_focused,
-            );
+            view_component_list(f, plugin_id, *kind, *state, &ctx);
         }
     }
 }
@@ -114,17 +110,14 @@ fn update_status_span(status: &UpdateStatusDisplay) -> Span<'_> {
 }
 
 /// プラグイン一覧画面を描画
-#[allow(clippy::too_many_arguments)]
 fn view_plugin_list(
     f: &mut Frame,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
     marked_ids: &HashSet<PluginId>,
     update_statuses: &HashMap<PluginId, UpdateStatusDisplay>,
 ) {
-    let filtered = filter_plugins(&data.plugins, filter_text);
+    let filtered = filter_plugins(&ctx.data.plugins, ctx.filter_text);
     let content_height = (filtered.len() as u16).max(1) + 9; // +3 for filter bar
     let dialog_width = 55u16;
     let dialog_height = content_height.min(24);
@@ -156,7 +149,7 @@ fn view_plugin_list(
     f.render_widget(tabs, chunks[0]);
 
     // フィルタバー
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);
 
     // タイトル（マーク数表示付き）
     let marked_count = marked_ids.len();
@@ -164,14 +157,14 @@ fn view_plugin_list(
         format!(
             " Installed Plugins ({}/{}) [{} marked] ",
             filtered.len(),
-            data.plugins.len(),
+            ctx.data.plugins.len(),
             marked_count
         )
     } else {
         format!(
             " Installed Plugins ({}/{}) ",
             filtered.len(),
-            data.plugins.len()
+            ctx.data.plugins.len()
         )
     };
 
@@ -250,11 +243,9 @@ fn view_plugin_detail(
     f: &mut Frame,
     plugin_id: &PluginId,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
 ) {
-    let Some(plugin) = data.find_plugin(plugin_id) else {
+    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
         return;
     };
 
@@ -289,7 +280,7 @@ fn view_plugin_detail(
     f.render_widget(tabs, chunks[0]);
 
     // フィルタバー（read-only）
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);
 
     // プラグイン情報
     let marketplace_str = plugin
@@ -353,15 +344,13 @@ fn view_component_types(
     f: &mut Frame,
     plugin_id: &PluginId,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
 ) {
-    let Some(plugin) = data.find_plugin(plugin_id) else {
+    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
         return;
     };
 
-    let counts = data.available_component_kinds(plugin);
+    let counts = ctx.data.available_component_kinds(plugin);
     let has_marketplace = plugin.marketplace.is_some();
     let base_lines = if has_marketplace { 4 } else { 3 };
     let type_lines = if counts.is_empty() { 1 } else { counts.len() };
@@ -382,7 +371,7 @@ fn view_component_types(
         .split(dialog_area);
 
     // フィルタバー（read-only）
-    render_filter_bar(f, chunks[0], filter_text, filter_focused);
+    render_filter_bar(f, chunks[0], ctx.filter_text, ctx.filter_focused);
 
     let title = format!(" {} ", plugin.name);
 
@@ -437,15 +426,13 @@ fn view_component_list(
     plugin_id: &PluginId,
     kind: ComponentKind,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
 ) {
-    let Some(plugin) = data.find_plugin(plugin_id) else {
+    let Some(plugin) = ctx.data.find_plugin(plugin_id) else {
         return;
     };
 
-    let components = data.component_names(plugin, kind);
+    let components = ctx.data.component_names(plugin, kind);
     let items: Vec<ListItem> = components
         .iter()
         .map(|c| ListItem::new(format!("  {}", c.name)))
@@ -468,7 +455,7 @@ fn view_component_list(
         .split(dialog_area);
 
     // フィルタバー（read-only）
-    render_filter_bar(f, chunks[0], filter_text, filter_focused);
+    render_filter_bar(f, chunks[0], ctx.filter_text, ctx.filter_focused);
 
     let title = format!(
         " {} > {} ({}) ",
diff --git a/src/tui/manager/screens/marketplaces/actions.rs b/src/tui/manager/screens/marketplaces/actions.rs
index ec39269..d59d9fb 100644
--- a/src/tui/manager/screens/marketplaces/actions.rs
+++ b/src/tui/manager/screens/marketplaces/actions.rs
@@ -214,24 +214,30 @@ fn make_all_failed_summary(plugin_names: &[String], error: &str) -> InstallSumma
     build_install_summary(results)
 }
 
+/// インストール処理のコンテキスト
+struct InstallCtx<'a> {
+    handle: &'a tokio::runtime::Handle,
+    targets: &'a [Box<dyn crate::target::Target>],
+    scope: Scope,
+    project_root: &'a Path,
+    cache: &'a dyn PluginCacheAccess,
+}
+
 /// 個別プラグインの download -> scan -> place パイプライン
 fn install_single_plugin(
-    handle: &tokio::runtime::Handle,
+    ctx: &InstallCtx<'_>,
     marketplace_name: &str,
     plugin_name: &str,
-    targets: &[Box<dyn crate::target::Target>],
-    scope: Scope,
-    project_root: &Path,
-    cache: &dyn PluginCacheAccess,
 ) -> PluginInstallResult {
     // Download (async -> sync bridge)
     let downloaded = match tokio::task::block_in_place(|| {
-        handle.block_on(install::download_marketplace_plugin_with_cache(
-            plugin_name,
-            marketplace_name,
-            false,
-            cache,
-        ))
+        ctx.handle
+            .block_on(install::download_marketplace_plugin_with_cache(
+                plugin_name,
+                marketplace_name,
+                false,
+                ctx.cache,
+            ))
     }) {
         Ok(d) => d,
         Err(e) => {
@@ -258,9 +264,9 @@ fn install_single_plugin(
     // Place
     let place_result = install::place_plugin(&PlaceRequest {
         scanned: &scanned,
-        targets,
-        scope,
-        project_root,
+        targets: ctx.targets,
+        scope: ctx.scope,
+        project_root: ctx.project_root,
     });
 
     if !place_result.failures.is_empty() {
@@ -341,19 +347,16 @@ pub fn install_plugins(
     };
 
     // 各プラグインに対して download -> scan -> place
+    let install_ctx = InstallCtx {
+        handle: &handle,
+        targets: &targets,
+        scope,
+        project_root: &project_root,
+        cache: &cache,
+    };
     let results: Vec<PluginInstallResult> = plugin_names
         .iter()
-        .map(|plugin_name| {
-            install_single_plugin(
-                &handle,
-                marketplace_name,
-                plugin_name,
-                &targets,
-                scope,
-                &project_root,
-                &cache,
-            )
-        })
+        .map(|plugin_name| install_single_plugin(&install_ctx, marketplace_name, plugin_name))
         .collect();
 
     build_install_summary(results)
diff --git a/src/tui/manager/screens/marketplaces/update.rs b/src/tui/manager/screens/marketplaces/update.rs
index 88bdfce..e4705f4 100644
--- a/src/tui/manager/screens/marketplaces/update.rs
+++ b/src/tui/manager/screens/marketplaces/update.rs
@@ -446,9 +446,12 @@ fn enter_form(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
             // Add は直接実行（source 情報を保持するため 2段階方式を使わない）
             let source = source.clone();
             let name = name.clone();
+            let entry = AddEntry {
+                source: &source,
+                name: &name,
+            };
             execute_add_with(
-                &source,
-                &name,
+                &entry,
                 model,
                 data,
                 |s, n| actions::add_marketplace(s, n, None),
@@ -459,17 +462,22 @@ fn enter_form(model: &mut Model, data: &mut DataStore) -> UpdateEffect {
     }
 }
 
+/// マーケットプレイス追加エントリ
+struct AddEntry<'a> {
+    source: &'a str,
+    name: &'a str,
+}
+
 /// ExecuteAdd の実装本体（依存関数注入パターン）
 fn execute_add_with(
-    source: &str,
-    name: &str,
+    entry: &AddEntry<'_>,
     model: &mut Model,
     data: &mut DataStore,
     run_add: impl FnOnce(&str, &str) -> Result<actions::AddResult, String>,
     reload: impl FnOnce(&mut DataStore),
 ) -> UpdateEffect {
-    let source_owned = source.to_string();
-    let name_owned = name.to_string();
+    let source_owned = entry.source.to_string();
+    let name_owned = entry.name.to_string();
 
     match run_add(&source_owned, &name_owned) {
         Ok(_result) => {
diff --git a/src/tui/manager/screens/marketplaces/update_test.rs b/src/tui/manager/screens/marketplaces/update_test.rs
index 79fb988..e2c49c4 100644
--- a/src/tui/manager/screens/marketplaces/update_test.rs
+++ b/src/tui/manager/screens/marketplaces/update_test.rs
@@ -1,4 +1,4 @@
-use super::{execute_add_with, execute_remove_with, execute_update_with, update};
+use super::{execute_add_with, execute_remove_with, execute_update_with, update, AddEntry};
 use crate::tui::manager::core::{DataStore, MarketplaceItem};
 use crate::tui::manager::screens::marketplaces::actions::AddResult;
 use crate::tui::manager::screens::marketplaces::model::{
@@ -1565,9 +1565,12 @@ fn execute_add_success_transitions_to_market_list() {
         error_message: None,
     });
 
+    let entry = AddEntry {
+        source: "owner/repo",
+        name: "my-repo",
+    };
     execute_add_with(
-        "owner/repo",
-        "my-repo",
+        &entry,
         &mut model,
         &mut data,
         |_source, name| Ok(make_add_result(name)),
@@ -1598,9 +1601,12 @@ fn execute_add_failure_returns_to_confirm_with_error() {
         error_message: None,
     });
 
+    let entry = AddEntry {
+        source: "owner/repo",
+        name: "my-repo",
+    };
     execute_add_with(
-        "owner/repo",
-        "my-repo",
+        &entry,
         &mut model,
         &mut data,
         |_source, _name| Err("network error".to_string()),
diff --git a/src/tui/manager/screens/marketplaces/view.rs b/src/tui/manager/screens/marketplaces/view.rs
index c805ea6..8fe662f 100644
--- a/src/tui/manager/screens/marketplaces/view.rs
+++ b/src/tui/manager/screens/marketplaces/view.rs
@@ -10,6 +10,26 @@ use ratatui::prelude::*;
 use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs};
 use std::collections::HashSet;
 
+/// 描画用共通コンテキスト（DataStore + フィルタ情報）
+struct ViewCtx<'a> {
+    data: &'a DataStore,
+    filter_text: &'a str,
+    filter_focused: bool,
+}
+
+/// フィルタ情報のみのコンテキスト
+struct FilterCtx<'a> {
+    text: &'a str,
+    focused: bool,
+}
+
+/// ブラウズ画面のデータ
+struct BrowseData<'a> {
+    plugins: &'a [BrowsePlugin],
+    selected_plugins: &'a HashSet<String>,
+    highlighted_idx: usize,
+}
+
 /// 画面を描画
 pub fn view(
     f: &mut Frame,
@@ -18,6 +38,15 @@ pub fn view(
     filter_text: &str,
     filter_focused: bool,
 ) {
+    let ctx = ViewCtx {
+        data,
+        filter_text,
+        filter_focused,
+    };
+    let filter = FilterCtx {
+        text: filter_text,
+        focused: filter_focused,
+    };
     match model {
         Model::MarketList {
             state,
@@ -25,15 +54,7 @@ pub fn view(
             error_message,
             ..
         } => {
-            view_market_list(
-                f,
-                *state,
-                data,
-                filter_text,
-                filter_focused,
-                operation_status,
-                error_message,
-            );
+            view_market_list(f, *state, &ctx, operation_status, error_message);
         }
         Model::MarketDetail {
             marketplace_name,
@@ -41,15 +62,7 @@ pub fn view(
             error_message,
             ..
         } => {
-            view_market_detail(
-                f,
-                marketplace_name,
-                *state,
-                data,
-                filter_text,
-                filter_focused,
-                error_message,
-            );
+            view_market_detail(f, marketplace_name, *state, &ctx, error_message);
         }
         Model::PluginList {
             marketplace_name,
@@ -57,14 +70,7 @@ pub fn view(
             plugins,
             ..
         } => {
-            view_plugin_list(
-                f,
-                marketplace_name,
-                *state,
-                plugins,
-                filter_text,
-                filter_focused,
-            );
+            view_plugin_list(f, marketplace_name, *state, plugins, &filter);
         }
         Model::AddForm(form) => {
             view_add_form(f, form, filter_text, filter_focused);
@@ -76,16 +82,12 @@ pub fn view(
             highlighted_idx,
             state,
         } => {
-            view_plugin_browse(
-                f,
-                marketplace_name,
+            let browse = BrowseData {
                 plugins,
                 selected_plugins,
-                *highlighted_idx,
-                *state,
-                filter_text,
-                filter_focused,
-            );
+                highlighted_idx: *highlighted_idx,
+            };
+            view_plugin_browse(f, marketplace_name, &browse, *state, &filter);
         }
         Model::TargetSelect {
             targets,
@@ -132,18 +134,15 @@ fn render_tab_bar(f: &mut Frame, area: Rect) {
 }
 
 /// マーケットプレイス一覧画面を描画
-#[allow(clippy::too_many_arguments)]
 fn view_market_list(
     f: &mut Frame,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
     operation_status: &Option<OperationStatus>,
     error_message: &Option<String>,
 ) {
     // リスト長: マーケットプレイス数 + 1（"+ Add new"）
-    let list_len = data.marketplaces.len() + 1;
+    let list_len = ctx.data.marketplaces.len() + 1;
     let has_status = operation_status.is_some();
     let has_error = error_message.is_some();
     let extra_lines = if has_status { 1 } else { 0 } + if has_error { 1 } else { 0 };
@@ -169,7 +168,7 @@ fn view_market_list(
     render_tab_bar(f, chunks[0]);
 
     // フィルタバー
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);
 
     // コンテンツ領域を分割（リスト + ステータス/エラー）
     let content_chunks = Layout::default()
@@ -181,8 +180,9 @@ fn view_market_list(
         .split(chunks[2]);
 
     // マーケットプレイスリスト
-    let title = format!(" Marketplaces ({}) ", data.marketplaces.len());
-    let mut items: Vec<ListItem> = data
+    let title = format!(" Marketplaces ({}) ", ctx.data.marketplaces.len());
+    let mut items: Vec<ListItem> = ctx
+        .data
         .marketplaces
         .iter()
         .map(|m| {
@@ -252,14 +252,11 @@ fn view_market_list(
 }
 
 /// マーケットプレイス詳細画面を描画
-#[allow(clippy::too_many_arguments)]
 fn view_market_detail(
     f: &mut Frame,
     marketplace_name: &str,
     mut state: ListState,
-    data: &DataStore,
-    filter_text: &str,
-    filter_focused: bool,
+    ctx: &ViewCtx<'_>,
     error_message: &Option<String>,
 ) {
     let dialog_width = 65u16;
@@ -287,10 +284,10 @@ fn view_market_detail(
     render_tab_bar(f, chunks[0]);
 
     // フィルタバー（read-only）
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], ctx.filter_text, ctx.filter_focused);
 
     // マーケットプレイス情報
-    let marketplace = data.find_marketplace(marketplace_name);
+    let marketplace = ctx.data.find_marketplace(marketplace_name);
     let title = format!(" {} ", marketplace_name);
 
     let info_lines = if let Some(m) = marketplace {
@@ -364,8 +361,7 @@ fn view_plugin_list(
     marketplace_name: &str,
     mut state: ListState,
     plugins: &[(String, Option<String>)],
-    filter_text: &str,
-    filter_focused: bool,
+    filter: &FilterCtx<'_>,
 ) {
     let content_height = (plugins.len() as u16).max(1) + 2; // +2 for borders
     let dialog_width = 65u16;
@@ -388,7 +384,7 @@ fn view_plugin_list(
     render_tab_bar(f, chunks[0]);
 
     // フィルタバー（read-only）
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], filter.text, filter.focused);
 
     // プラグインリスト
     let title = format!(" {} > Plugins ({}) ", marketplace_name, plugins.len());
@@ -572,16 +568,12 @@ fn view_add_form(f: &mut Frame, form: &AddFormModel, filter_text: &str, filter_f
 }
 
 /// プラグインブラウズ画面を描画
-#[allow(clippy::too_many_arguments)]
 fn view_plugin_browse(
     f: &mut Frame,
     marketplace_name: &str,
-    plugins: &[BrowsePlugin],
-    selected_plugins: &HashSet<String>,
-    highlighted_idx: usize,
+    browse: &BrowseData<'_>,
     mut state: ListState,
-    filter_text: &str,
-    filter_focused: bool,
+    filter: &FilterCtx<'_>,
 ) {
     let dialog_width = 80u16;
     let dialog_height = 24u16;
@@ -603,20 +595,20 @@ fn view_plugin_browse(
     render_tab_bar(f, chunks[0]);
 
     // フィルタバー
-    render_filter_bar(f, chunks[1], filter_text, filter_focused);
+    render_filter_bar(f, chunks[1], filter.text, filter.focused);
 
     // コンテンツ領域
-    let title = format!(" {} > Browse ({}) ", marketplace_name, plugins.len());
+    let title = format!(" {} > Browse ({}) ", marketplace_name, browse.plugins.len());
     let content_area = chunks[2];
 
-    if plugins.is_empty() {
+    if browse.plugins.is_empty() {
         let msg = Paragraph::new("  No plugins available.")
             .block(Block::default().title(title).borders(Borders::ALL))
             .style(Style::default().fg(Color::DarkGray));
         f.render_widget(msg, content_area);
     } else if !should_split_layout(content_area.width) {
         // 狭い端末: リストのみ描画
-        let items = build_browse_list_items(plugins, selected_plugins);
+        let items = build_browse_list_items(browse.plugins, browse.selected_plugins);
         let list = List::new(items)
             .block(Block::default().title(title).borders(Borders::ALL))
             .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
@@ -630,7 +622,7 @@ fn view_plugin_browse(
             .split(content_area);
 
         // 左パネル: プラグインリスト
-        let items = build_browse_list_items(plugins, selected_plugins);
+        let items = build_browse_list_items(browse.plugins, browse.selected_plugins);
         let list = List::new(items)
             .block(Block::default().title(title).borders(Borders::ALL))
             .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
@@ -638,8 +630,8 @@ fn view_plugin_browse(
         f.render_stateful_widget(list, h_chunks[0], &mut state);
 
         // 右パネル: プラグイン詳細（state.selected() から導出して一貫性を保つ）
-        let detail_idx = state.selected().unwrap_or(highlighted_idx);
-        render_plugin_detail(f, plugins, detail_idx, h_chunks[1]);
+        let detail_idx = state.selected().unwrap_or(browse.highlighted_idx);
+        render_plugin_detail(f, browse.plugins, detail_idx, h_chunks[1]);
     }
 
     // ヘルプ

## レビュー観点
1. 実装計画との整合性: 計画通りに実装されているか
2. コード品質: 可読性、保守性、命名規則は適切か
3. エッジケース: 空データ、エラー、境界値を考慮しているか
4. セキュリティ: インジェクション、XSSなどの脆弱性はないか
5. パフォーマンス: 不要なループ、N+1問題などはないか

問題がなければ「問題なし」と回答してください。
問題があれば具体的な指摘と改善案を提示してください。

mcp startup: no servers
codex
対象差分を実装計画と照らして確認します。まず実ファイルの該当箇所を読み、Clippy 対応が副作用や未解消の問題を持っていないかを見ます。
exec
/bin/bash -lc "rg --files /workspace/src | rg \"plugin_intent|converter|host\\.rs|plugin/update\\.rs|sync\\.rs|installed/view\\.rs|marketplaces/actions\\.rs|marketplaces/update\\.rs|marketplaces/update_test\\.rs|marketplaces/view\\.rs\"" in /workspace succeeded in 1ms:
/workspace/src/host.rs
/workspace/src/sync.rs
/workspace/src/hooks/converter_test.rs
/workspace/src/hooks/converter.rs
/workspace/src/plugin/update.rs
/workspace/src/tui/manager/screens/marketplaces/view.rs
/workspace/src/tui/manager/screens/marketplaces/update_test.rs
/workspace/src/tui/manager/screens/marketplaces/update.rs
/workspace/src/tui/manager/screens/marketplaces/actions.rs
/workspace/src/application/plugin_intent.rs
/workspace/src/application/plugin_intent_test.rs
/workspace/src/tui/manager/screens/installed/view.rs
/workspace/src/commands/sync.rs
exec
/bin/bash -lc 'git status --short' in /workspaceexec
/bin/bash -lc "rg -n \"allow\\(clippy::|type_complexity|too_many_arguments|clippy.toml\" -S /workspace" in /workspace succeeded in 0ms:
 M .claude/settings.json
 M .claude/settings.local.json
 M src/application/plugin_intent.rs
 M src/hooks/converter.rs
 M src/host.rs
 M src/plugin/update.rs
 M src/sync.rs
 M src/tui/manager/screens/installed/view.rs
 M src/tui/manager/screens/marketplaces/actions.rs
 M src/tui/manager/screens/marketplaces/update.rs
 M src/tui/manager/screens/marketplaces/update_test.rs
 M src/tui/manager/screens/marketplaces/view.rs
?? .DS_Store
?? .spec-to-issues/
?? clippy.toml
?? docs/code-review-2026-02-22.md
?? docs/design-marketplace-browse-install.md
?? docs/impl-claude-code-target.md
?? docs/impl-copilot-hooks.md
?? docs/impl-marketplace-browse-install.md
 exited 1 in 0ms:

codex
差分自体は指定ファイルに限定されていて、`allow(clippy::...)` は現状見当たりません。次に `clippy.toml` と各変更箇所の前後文脈を読み、実装計画の 4 項目が実際に成立しているかを確認します。
exec
/bin/bash -lc "sed -n '1,120p' /workspace/clippy.toml" in /workspace succeeded in 0ms:
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 5
too-many-lines-threshold = 80
type-complexity-threshold = 200
exec
/bin/bash -lc 'git diff -- /workspace/src/application/plugin_intent.rs /workspace/src/hooks/converter.rs /workspace/src/host.rs /workspace/src/plugin/update.rs /workspace/src/sync.rs /workspace/src/tui/manager/screens/installed/view.rs /workspace/src/tui/manager/screens/marketplaces/actions.rs /workspace/src/tui/manager/screens/marketplaces/update.rs /workspace/src/tui/manager/screens/marketplaces/update_test.rs /workspace/src/tui/manager/screens/marketplaces/view.rs /workspace/clippy.toml' in /workspace succeeded in 0ms:
diff --git a/src/application/plugin_intent.rs b/src/application/plugin_intent.rs
index a40cab5..edb5fa3 100644
--- a/src/application/plugin_intent.rs
+++ b/src/application/plugin_intent.rs
@@ -16,6 +16,10 @@ use crate::fs::{FileSystem, RealFs};
 use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
 use std::path::{Path, PathBuf};
 
+/// 単一コンポーネントの操作生成結果
+type CreateOperationResult =
+    std::result::Result<Option<(TargetId, FileOperation)>, (TargetId, String)>;
+
 /// `expand()` の結果
 #[derive(Debug)]
 pub struct ExpandResult {
@@ -135,7 +139,7 @@ impl PluginIntent {
         target: &dyn Target,
         component: &Component,
         origin: &PluginOrigin,
-    ) -> Result<Option<(TargetId, FileOperation)>, (TargetId, String)> {
exec
/bin/bash -lc "nl -ba /workspace/src/tui/manager/screens/marketplaces/update.rs | sed -n '430,510p'" in /workspaceexec
/bin/bash -lc "nl -ba /workspace/src/plugin/update.rs | sed -n '180,290p'" in /workspaceexec
/bin/bash -lc "nl -ba /workspace/src/tui/manager/screens/marketplaces/actions.rs | sed -n '210,380p'" in /workspace succeeded in 0ms:
   180	    }
   181	
   182	    // commit_sha 未保存時は警告表示
   183	    if current_sha.is_none() {
   184	        eprintln!(
   185	            "Warning: No commit SHA recorded for '{}'. Forcing update.",
   186	            plugin_name
   187	        );
   188	    }
   189	
   190	    // 更新処理実行
   191	    let ctx = UpdateCtx {
   192	        cache,
   193	        client: &*client,
   194	        repo: &repo,
   195	        plugin_meta: &plugin_meta,
   196	        project_root,
   197	        target_filter,
   198	    };
   199	    do_update(plugin_name, &latest_sha, &ctx).await
 succeeded in 0ms:
   210	            success: false,
   211	            error: Some(error.to_string()),
   212	        })
   213	        .collect();
   214	    build_install_summary(results)
   215	}
   216	
   217	/// インストール処理のコンテキスト
   218	struct InstallCtx<'a> {
   219	    handle: &'a tokio::runtime::Handle,
   220	    targets: &'a [Box<dyn crate::target::Target>],
   221	    scope: Scope,
   222	    project_root: &'a Path,
   223	    cache: &'a dyn PluginCacheAccess,
   224	}
   225	
   226	/// 個別プラグインの download -> scan -> place パイプライン
   227	fn install_single_plugin(
   228	    ctx: &InstallCtx<'_>,
   229	    marketplace_name: &str,
 succeeded in 0ms:
   430	
   431	            // 重複チェック
   432	            if data.find_marketplace(&name).is_some() {
   433	                *error_message = Some(format!("Marketplace '{}' already exists", name));
   434	                return UpdateEffect::none();
   435	            }
   436	
   437	            let source = source.clone();
   438	            *model = Model::AddForm(AddFormModel::Confirm {
   439	                source,
   440	                name,
   441	                error_message: None,
   442	            });
   443	            UpdateEffect::none()
   444	        }
   445	        Model::AddForm(AddFormModel::Confirm { source, name, .. }) => {
   446	            // Add は直接実行（source 情報を保持するため 2段階方式を使わない）
   447	            let source = source.clone();
   448	            let name = name.clone();
   449	            let entry = AddEntry {
codex
文脈確認は済みました。次はビルド系で裏取りし、厳しくした `clippy.toml` を入れた状態で未解消警告や想定外の退行がないかを確認します。
exec
/bin/bash -lc 'cargo clippy --all-targets --all-features' in /workspaceexec
/bin/bash -lc 'cargo test' in /workspacecodex
`cargo clippy` と `cargo test` を回しています。並行して、差分の中で構造体導入が意味的に妥当か、単なる引数束ねに留まらず不整合を生んでいないかも見ています。
exec
/bin/bash -lc "nl -ba /workspace/src/application/plugin_intent.rs | sed -n '1,190p'" in /workspaceexec
/bin/bash -lc "nl -ba /workspace/src/host.rs | sed -n '1,120p'" in /workspace succeeded in 0ms:
     1	//! プラグイン操作意図
     2	//!
     3	//! PluginIntent は事前スキャン済みデータを保持し、
     4	//! 低レベルファイル操作への展開と実行を担う。
     5	
     6	// Re-exported for tests
     7	#[cfg(test)]
     8	pub use super::plugin_action::PluginAction;
     9	#[cfg(not(test))]
    10	use super::plugin_action::PluginAction;
    11	use super::plugin_action_types::{FileOperation, ScopedPath, TargetId};
    12	use crate::component::{
    13	    Component, ComponentKind, ComponentRef, PlacementContext, PlacementScope, ProjectContext, Scope,
    14	};
    15	use crate::fs::{FileSystem, RealFs};
    16	use crate::target::{all_targets, AffectedTargets, OperationResult, PluginOrigin, Target};
    17	use std::path::{Path, PathBuf};
    18	
    19	/// 単一コンポーネントの操作生成結果
    20	type CreateOperationResult =
 succeeded in 0ms:
     1	//! ホスト別クライアント
     2	//!
     3	//! GitHub, GitLab, Bitbucket 等のホスティングサービス用クライアント。
     4	
     5	pub mod github;
     6	
     7	pub use github::GitHubClient;
     8	
     9	use crate::config::{AuthProvider, HttpConfig};
    10	use crate::error::Result;
    11	use crate::repo::Repo;
    12	use std::future::Future;
    13	use std::pin::Pin;
    14	
    15	/// 非同期メソッドの戻り値型エイリアス
    16	type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
    17	
    18	/// ホスト種別
    19	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    20	pub enum HostKind {
exec
/bin/bash -lc "nl -ba /workspace/src/tui/manager/screens/marketplaces/view.rs | sed -n '1,240p'" in /workspaceexec
/bin/bash -lc "nl -ba /workspace/src/hooks/converter.rs | sed -n '450,880p'" in /workspace succeeded in 0ms:
   450	        .and_then(|t| t.as_str())
   451	        .unwrap_or("command");
   452	
   453	    match hook_type {
   454	        "command" => {
   455	            let converted = convert_command_hook(hook, matcher, event, warnings, wrapper_scripts)?;
   456	            Ok(Some(converted))
   457	        }
   458	        "http" => {
   459	            let converted = convert_http_hook(hook, event, matcher, wrapper_scripts)?;
   460	            Ok(Some(converted))
   461	        }
   462	        "prompt" | "agent" => {
   463	            let mut collector = ConversionCollector {
   464	                warnings,
   465	                wrapper_scripts,
   466	            };
   467	            let converted =
   468	                convert_prompt_agent_hook(hook, hook_type, event, matcher, &mut collector);
   469	            Ok(Some(converted))
 succeeded in 0ms:
     1	//! Marketplaces タブの view（描画）
     2	
     3	use super::model::{
     4	    AddFormModel, BrowsePlugin, DetailAction, InstallSummary, Model, OperationStatus,
     5	};
     6	use crate::component::Scope;
     7	use crate::marketplace::PluginSource;
     8	use crate::tui::manager::core::{dialog_rect, render_filter_bar, DataStore, Tab};
     9	use ratatui::prelude::*;
    10	use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs};
    11	use std::collections::HashSet;
    12	
    13	/// 描画用共通コンテキスト（DataStore + フィルタ情報）
    14	struct ViewCtx<'a> {
    15	    data: &'a DataStore,
    16	    filter_text: &'a str,
    17	    filter_focused: bool,
    18	}
    19	
    20	/// フィルタ情報のみのコンテキスト
 exited 101 in 14.59s:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling plm v0.3.1 (/workspace)
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
   --> src/tui/manager/screens/marketplaces/update_test.rs:722:9
 succeeded in 17.83s:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on build directory
    Checking num-traits v0.2.19
    Checking regex-automata v0.4.13
    Checking rand_core v0.9.3
    Checking zerocopy v0.8.31
    Checking wait-timeout v0.2.1
    Checking predicates-core v1.0.9
    Checking bit-vec v0.8.0
    Checking float-cmp v0.10.0
    Checking quick-error v1.2.3
    Checking fnv v1.0.7
    Checking difflib v0.4.0
    Checking termtree v0.5.1
    Checking normalize-line-endings v0.3.0
    Checking chrono v0.4.42
    Checking predicates-tree v1.0.12
    Checking rusty-fork v0.3.1
    Checking bit-set v0.8.0
codex
全体 `cargo test` の失敗は、このリポジトリ既存の `assert_cmd` 系テストが `/workspace/.target/debug/plm` を前提にしているためで、今回差分の直接起因ではありません。変更箇所そのものの妥当性を切り分けるため、関連モジュールのテストだけ追加で確認します。
exec
/bin/bash -lc 'cargo test tui::manager::screens::installed::tests tui::manager::screens::marketplaces::actions_test tui::manager::screens::marketplaces::view_test' in /workspace exited 1 in 0ms:
error: unexpected argument 'tui::manager::screens::marketplaces::actions_test' found

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

For more information, try '--help'.
exec
/bin/bash -lc 'cargo test application::plugin_intent::tests hooks::converter::tests plugin::update::tests sync::tests tui::manager::screens::marketplaces::tests' in /workspace exited 1 in 0ms:
error: unexpected argument 'hooks::converter::tests' found

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

For more information, try '--help'.
exec
/bin/bash -lc 'cargo test tui::manager::screens::marketplaces::update_test' in /workspace succeeded in 387ms:
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
   --> src/tui/manager/screens/marketplaces/update_test.rs:722:9
    |
722 |     use std::collections::HashSet;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^

codex
切り分け用の `cargo test` では、モジュール名指定が `cargo` の仕様と合わず一部空振りしました。変更に直接触れている代表テスト名を個別に実行して、少なくとも差分のコンパイルと代表ケースは通ることを確認します。
exec
/bin/bash -lc 'cargo test execute_add_failure_returns_to_confirm_with_error' in /workspaceexec
/bin/bash -lc 'cargo test execute_add_success_transitions_to_market_list' in /workspaceexec
/bin/bash -lc 'cargo test test_plugin_intent_expand_empty' in /workspaceexec
/bin/bash -lc 'cargo test hook_convert_multiple_hooks_no_name_collision' in /workspace succeeded in 936ms:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
   --> src/tui/manager/screens/marketplaces/update_test.rs:722:9
    |
 succeeded in 1.12s:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
   --> src/tui/manager/screens/marketplaces/update_test.rs:722:9
    |
 succeeded in 1.36s:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on shared package cache
    Blocking waiting for file lock on artifact directory
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
 succeeded in 1.35s:
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on artifact directory
warning: unused import: `super::copilot_agent::CopilotAgent`
 --> src/parser/claude_code_agent_test.rs:3:5
  |
3 | use super::copilot_agent::CopilotAgent;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `ScannedComponent`
 --> src/target/scanner_test.rs:3:30
  |
3 | use super::{scan_components, ScannedComponent};
  |                              ^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashSet`
   --> src/tui/manager/screens/marketplaces/update_test.rs:722:9
    |
codex
問題なしです。

実装計画との整合性は取れています。`clippy.toml` には 4 つの閾値が追加され、`too-many-arguments` は `UpdateCtx` / `InstallCtx` / `SyncPlan` / `ViewCtx` / `FilterCtx` / `BrowseData` / `AddEntry` などの導入で解消、`type-complexity` は [src/host.rs](/workspace/src/host.rs) と [src/application/plugin_intent.rs](/workspace/src/application/plugin_intent.rs) の型エイリアス化で解消、`allow(clippy::...)` も差分対象では除去されていました。可読性と保守性も、単なる引数削減ではなく文脈オブジェクト化として概ね妥当です。

検証では `cargo clippy --all-targets --all-features` は通過しました。`cargo test` は既存の `assert_cmd` 系テストが `/workspace/.target/debug/plm` を前提にして失敗しましたが、今回差分に直接関係する代表テストとして `execute_add_success_transitions_to_market_list`、`execute_add_failure_returns_to_confirm_with_error`、`test_plugin_intent_expand_empty`、`test_hook_convert_multiple_hooks_no_name_collision` は通過しています。残る注意点は、この環境では全体テストが最後まで健全性確認できていないことだけです。
tokens used
67,041
問題なしです。

実装計画との整合性は取れています。`clippy.toml` には 4 つの閾値が追加され、`too-many-arguments` は `UpdateCtx` / `InstallCtx` / `SyncPlan` / `ViewCtx` / `FilterCtx` / `BrowseData` / `AddEntry` などの導入で解消、`type-complexity` は [src/host.rs](/workspace/src/host.rs) と [src/application/plugin_intent.rs](/workspace/src/application/plugin_intent.rs) の型エイリアス化で解消、`allow(clippy::...)` も差分対象では除去されていました。可読性と保守性も、単なる引数削減ではなく文脈オブジェクト化として概ね妥当です。

検証では `cargo clippy --all-targets --all-features` は通過しました。`cargo test` は既存の `assert_cmd` 系テストが `/workspace/.target/debug/plm` を前提にして失敗しましたが、今回差分に直接関係する代表テストとして `execute_add_success_transitions_to_market_list`、`execute_add_failure_returns_to_confirm_with_error`、`test_plugin_intent_expand_empty`、`test_hook_convert_multiple_hooks_no_name_collision` は通過しています。残る注意点は、この環境では全体テストが最後まで健全性確認できていないことだけです。
