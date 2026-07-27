# Issue #393 実装探索メモ（Plugin root attached resources）

レビュー日: 2026-07-28  
対象: [#393](https://github.com/DIO0550/plugin-manager/issues/393)  
仕様正本: `docs/architecture/file-formats.md`「Plugin 付属リソース」（#407 で確定・案 A）  
関連レビュー: `docs/review-issue-393-plugin-root-resources.md`

本メモは実装 PR 向けに、現状コードの型・関数署名・呼び出し経路と注入ポイントを具体化する。

---

## 1. `ComponentDeployment` / `deploy_skill` / `execute`

**ファイル:** `/workspace/src/component/deployment.rs`

### struct フィールド

```27:32:src/component/deployment.rs
pub struct ComponentDeployment {
    pub(super) component: Component,
    pub scope: Scope,
    pub(super) target_path: PathBuf,
    pub(super) conversion: ConversionConfig,
}
```

- **plugin root フィールドは無い。**
- ソースは `component.path` のみ（Skill なら Skill ディレクトリ）。
- `source_path()` は `pub(super)`:

```55:58:src/component/deployment.rs
    pub(super) fn source_path(&self) -> &Path {
        &self.component.path
    }
```

### `execute` / `execute_with_fs`

```68:81:src/component/deployment.rs
    pub fn execute(&self) -> Result<DeploymentOutput> {
        self.execute_with_fs(&RealFs)
    }

    pub fn execute_with_fs(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        match self.kind() {
            ComponentKind::Skill => self.deploy_skill(fs),
            // ...
        }
    }
```

### `deploy_skill`（全文相当）

```88:109:src/component/deployment.rs
    fn deploy_skill(&self, fs: &dyn FileSystem) -> Result<DeploymentOutput> {
        fs.replace_dir(self.source_path(), &self.target_path)?;

        if let ConversionConfig::Skill { target_kind } = &self.conversion {
            if let Some(allowed) = convert::skill_allowed_fields(*target_kind) {
                let manifest = self.target_path.join(ComponentKind::skill_manifest());
                if fs.exists(&manifest) && !fs.is_dir(&manifest) {
                    let original = fs.read_to_string(&manifest)?;
                    let stripped = convert::strip_skill_frontmatter_fields(&original, allowed);
                    if stripped != original {
                        convert::atomic_write(&manifest, &stripped)?;
                    }
                }
            }
        }

        Ok(DeploymentOutput::Copied)
    }
```

要点:
- Skill 内付属（#392）は `replace_dir` で副次コピー済み。
- Plugin 直下 overlay は **無い**。`Ok(DeploymentOutput::Copied)` に warnings も無い。
- `replace_dir` **後**に overlay しないと消える（仕様どおり）。

### plugin root の渡し方（既存先例）

`ConversionConfig::Hook` だけが `plugin_root: Option<PathBuf>` を持つ:

```15:35:src/component/deployment/conversion.rs
pub enum ConversionConfig {
    #[default]
    None,
    Command { source: CommandFormat, dest: CommandFormat },
    Agent { source: AgentFormat, dest: AgentFormat },
    Hook {
        target_kind: TargetKind,
        plugin_root: Option<PathBuf>,
    },
    Skill { target_kind: TargetKind },
}
```

Builder:

```12:18:src/component/deployment/builder.rs
pub struct ComponentDeploymentBuilder {
    component: Option<Component>,
    scope: Option<Scope>,
    target_path: Option<PathBuf>,
    conversion: ConversionConfig,
}
```

**plugin root を deploy に渡す最短拡張:** `ConversionConfig::Skill` に `plugin_root: Option<PathBuf>`（または付属リスト）を足す。Hook と同パターンで `place_plugin` が既に `request.scanned.plugin_root()` を持っている。

---

## 2. `place_plugin` → deployment（plugin root を知っているか）

**ファイル:** `/workspace/src/install.rs`

`ScannedPlugin::plugin_root()` はパッケージ path を返す:

```51:55:src/install.rs
    pub fn plugin_root(&self) -> &Path {
        self.package.path()
    }
```

`place_plugin` 内で Hook には渡すが、**Skill には渡さない**:

```246:277:src/install.rs
            let conversion = match component.kind {
                // ...
                ComponentKind::Hook
                    if matches!(
                        target.kind(),
                        TargetKind::Codex | TargetKind::Copilot | TargetKind::Cursor
                    ) =>
                {
                    ConversionConfig::Hook {
                        target_kind: target.kind(),
                        plugin_root: Some(request.scanned.plugin_root().to_path_buf()),
                    }
                }
                ComponentKind::Skill => ConversionConfig::Skill {
                    target_kind: target.kind(),
                },
                _ => ConversionConfig::None,
            };

            let deployment = match ComponentDeployment::builder()
                .component(component.clone())
                .scope(request.scope)
                .target_path(&target_path)
                .conversion(conversion)
                .build()
```

`ComponentDeployment` 自体は plugin root を知らない。  
`pre_place_check` / `post_place` には `plugin_root` が渡るが、配置本体には入らない。

---

## 3. `PluginManifest` パス解決

**ファイル:** `/workspace/src/plugin/meta/manifest.rs`

フィールド（相対パス Option）: `commands`, `agents`, `skills`, `instructions`, `hooks`, …

解決メソッド（いずれも `base: &Path` = plugin root）:

| メソッド | 署名 | デフォルト |
|----------|------|------------|
| `skills_dir` | `fn skills_dir(&self, base: &Path) -> PathBuf` | `ComponentKind::Skill.plural()` = `"skills"` |
| `agents_dir` | 同上 | `"agents"` |
| `commands_dir` | 同上 | `"commands"` |
| `instructions_path` | 同上 | `"instructions.md"` |
| `instructions_dir` | 同上 | `"instructions"` |
| `hooks_dir` | 同上 | `"hooks"` |

実装は `base.join_or(self.skills.as_deref(), …)`（`PathExt::join_or`）。

消費箇所: `Plugin::build_components`（`/workspace/src/plugin/content/plugin_content.rs` 127–157）が `manifest.skills_dir(path)` 等でスキャン。**付属リソース列挙は無い。**

`Plugin` / `MarketplaceContent` も同名の委譲アクセサを持つ（`skills_dir()` 等、内部で `self.path` を使う）。

除外判定は仕様どおり **マニフェスト解決後パス**で行う必要あり（リテラル `skills/` だけではカスタムパスを誤同梱する）。

---

## 4. enable の `CopyDir` 実行経路（`intent.rs`）

**ファイル:** `/workspace/src/plugin/lifecycle/intent.rs`

### 呼び出しチェーン

```
application::enable_plugin
  → load_plugin → plugin.components().to_vec()
  → PluginIntent::with_target_filter(PluginAction::Enable {..}, components, project_root, filter)
  → intent.apply()
       → expand() → create_operation / build_file_operation
       → execute_file_operations
```

`PluginIntent` フィールド:

```35:40:src/plugin/lifecycle/intent.rs
pub struct PluginIntent {
    action: PluginAction,
    components: Vec<Component>,
    project_root: PathBuf,
    target_filter: Option<String>,
}
```

**plugin root / attached resources フィールドは無い。** Skill の source は `component.path` のみ。

### Skill deploy 操作生成

```151:164:src/plugin/lifecycle/intent.rs
    fn build_file_operation(&self, component: &Component, scoped: ScopedPath) -> FileOperation {
        match (self.action.is_deploy(), component.kind) {
            (true, ComponentKind::Skill) => FileOperation::CopyDir {
                source: component.path.clone(),
                target: scoped,
            },
            (true, _) => FileOperation::CopyFile { .. },
            (false, ComponentKind::Skill) => FileOperation::RemoveDir { path: scoped },
            (false, _) => FileOperation::RemoveFile { path: scoped },
        }
    }
```

### 実行（実際は `replace_dir`）

```238:245:src/plugin/lifecycle/intent.rs
            let result = match &op {
                FileOperation::CopyFile { source, target } => {
                    fs.copy_file(source, target.as_path())
                }
                FileOperation::CopyDir { source, target } => {
                    fs.replace_dir(source, target.as_path())
                }
```

`FileOperation`（`/workspace/src/component/model/file_operation.rs`）:

```8:13:src/component/model/file_operation.rs
pub enum FileOperation {
    CopyFile { source: PathBuf, target: ScopedPath },
    CopyDir { source: PathBuf, target: ScopedPath },
    RemoveFile { path: ScopedPath },
    RemoveDir { path: ScopedPath },
}
```

enable 経路にも overlay が必要（仕様・既存レビューどおり）。現状 `execute_file_operations` は CopyDir 後に何もしない。

---

## 5. `FileSystem` trait（`src/fs.rs`）

```46:188:src/fs.rs
pub trait FileSystem: Send + Sync {
    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;      // merge（余剰残る）
    fn replace_dir(&self, src: &Path, dst: &Path) -> Result<()>;   // 完全置換
    fn remove(&self, path: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn remove_dir_all(&self, path: &Path) -> Result<()>;
    fn rename(&self, src: &Path, dst: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn mtime(&self, path: &Path) -> Result<SystemTime>;
    fn content_hash(&self, path: &Path) -> Result<u64>;
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write(&self, path: &Path, content: &[u8]) -> Result<()>;
    fn read_dir(&self, path: &Path) -> Result<Vec<FsNode>>;
}
```

**無いもの:** `file_size` / `dir_size` / `metadata` / バイト総量 API。  
overlay 実装は `exists` + `copy_file` / `copy_dir` + `read_dir` で足りる。衝突 skip は `fs.exists(dst)` で Skill 側優先。

---

## 6. 警告表示（ConversionWarning / install 後）

### 現状チャネルは Hook 専用

- `ConversionWarning`（`src/hooks/converter/converter.rs`）: Hook 変換専用 variant のみ。
- `DeploymentOutput::Copied` に warnings 無し。`HookConverted(HookConvertOutput { warnings, … })` のみ。
- `PlaceSuccess.hook_warnings: Vec<ConversionWarning>` — コメントどおり Hook 以外は空。

表示フロー:

1. `place_plugin` が `HookConverted` から `hook_warnings` を `PlaceSuccess` へ
2. `commands/deploy/install.rs` `render_place_success_to_strings` → `render_hook_success`
3. `component_kind != Hook` なら **空の stderr**（Skill 警告は出ない）

```232:247:src/install/format.rs
pub fn render_hook_success(input: HookRenderInput<'_>) -> HookRenderOutput {
    // ...
    if component_kind != ComponentKind::Hook {
        return HookRenderOutput {
            stdout_suffix: None,
            stderr_blocks: Vec::new(),
        };
    }
```

CLI 出力:

```202:207:src/commands/deploy/install.rs
            let (stdout_line, stderr_blocks) = render_place_success_to_strings(success);
            println!("{}", stdout_line);
            for block in &stderr_blocks {
                eprintln!("{}", block);
            }
```

**#393 では Hook の `ConversionWarning` に載せない。** 配置結果へ警告を集約する共通チャネルが必要（仕様書明示）。候補:

- `DeploymentOutput::Copied` → `Copied { warnings: Vec<PlacementWarning> }` など
- `PlaceSuccess` に `placement_warnings: Vec<…>` を追加し、`render_place_success_to_strings` で Skill も stderr に出す
- enable は現状 `OperationOutcome` に警告フィールド無し → 別途拡張か install のみ先に出す

---

## 7. Skill bundled resource テスト

**ファイル:** `/workspace/src/component/deployment_test.rs`（512 行付近〜）

| ヘルパー / テスト | 役割 |
|-------------------|------|
| `write_skill_with_bundled_resources(source: &Path)` | `SKILL.md` + `notes.md` + `references/a.md` + `assets/templates/x.html` |
| `assert_bundled_resources_copied(target: &Path)` | 同相対構造の存在・内容検証 |
| `test_execute_skill_copies_bundled_resources_same_structure` | 全 Skill 対応 TargetKind |
| `test_execute_skill_strip_does_not_touch_bundled_markdown` | Codex/Gemini の strip が付属 md を触らない |
| `test_execute_skill_replace_dir_removes_stale_bundled_resources` | stale 掃除 |

これらは **Skill 内**付属（#392）。Plugin 直下 overlay 用ヘルパーは未整備。同ファイルに plugin-root fixture + conflict テストを追加するのが自然。

---

## 8. `Component` / `PlacementContext` — plugin root を運べるか

### `Component`（`src/component/model/kind.rs`）

```134:145:src/component/model/kind.rs
pub struct Component {
    pub kind: ComponentKind,
    pub name: String,
    pub original_name: Option<String>,
    pub plugin_name: String,
    pub path: PathBuf,   // Skill なら skills/<skill>/ のパス。plugin root ではない
}
```

付属リソース用フィールド無し。`ComponentKind` も 5 種のまま（仕様: 拡張しない）。

### `PlacementContext`（`src/component/model/placement.rs`）

```115:120:src/component/model/placement.rs
pub struct PlacementContext<'a> {
    pub component: ComponentRef,
    pub origin: &'a PluginOrigin,
    pub scope: PlacementScope,
    pub project: ProjectContext<'a>,  // project_root のみ
}
```

`ComponentRef` も kind/name/original_name/plugin_name のみ。**plugin root / attached resources を載せる場ではない**（配置先決定用）。

### 参考: Hook の `plugin_root`

配置実行コンテキストとしては `ConversionConfig::Hook.plugin_root` が先例。Skill も同様に conversion / deployment 側へ注入するのが既存パターンに合う。

---

## 9. `install::place_plugin` フロー

```
download_plugin / scan_plugin(package, type_filter)
  → ScannedPlugin { package, components }
place_plugin(&PlaceRequest { scanned, targets, scope, project_root, enable_codex_hooks_flag })
  for target in targets:
    for component in scanned.components (supports?):
      PlacementContext → placement_location → target_path
      pre_place_check(..., scanned.plugin_root())
      ConversionConfig（Skill: target_kind のみ）
      ComponentDeployment::builder()...build()
      deployment.execute()
      → PlaceSuccess / PlaceFailure
      post_place(..., plugin_root, ...)
    cleanup_legacy_hierarchy（failure 無し時）
→ PlaceOutcome
update_meta_after_place(plugin_path, &result)
CLI: render_place_success_to_strings / eprintln warnings
```

付属リストを **プラグイン 1 回**で取るなら、`place_plugin` 先頭（target ループ前）か `scan_plugin` / `ScannedPlugin` 構築時が適切。

---

## 10. サイズ / バイト計測ユーティリティ

**専用の dir 総量・閾値ユーティリティは存在しない。**

あるもの:
- `http.rs` の download `content_length` → progress bar（無関係）
- `FileSystem::content_hash`（内容ハッシュ、サイズではない）
- `FileSystem::read_dir` / `exists` / `is_dir`

仕様（file-formats.md）: 「付属リソース総量が閾値を超える場合は配置せず警告」。閾値の定数値は docs にも未定義 → 実装時に `placement_names` か新モジュールへ定数追加が必要。

---

## 推奨注入ポイント

### A. 付属リソース列挙（プラグイン 1 回）

**場所:** `src/scan/` に新 API（例）

```rust
pub fn list_plugin_attached_resources(
    plugin_root: &Path,
    manifest: &PluginManifest,
) -> AttachedResources; // 相対 PathBuf のリスト + 総バイト数など
```

除外セットの合成:
1. `manifest.skills_dir(root)` / `agents_dir` / `commands_dir` / `hooks_dir` / instructions 解決パス
2. `placement_names::ALL_INSTRUCTION_FILENAMES` + `instructions.md`
3. 新規定数（`.claude-plugin`, `.plm-meta.json`, VCS, README*, LICENSE* …）— 現状 `placement_names.rs` には未追加

呼び出し:
- **install:** `scan_plugin` 後 or `place_plugin` 冒頭で 1 回。`ScannedPlugin` に `attached: AttachedResources` を載せるか、ローカル変数で Skill 配置時に渡す。
- **enable:** `load_plugin` 後、`Plugin::path()` + `Plugin::manifest()` で同じ API を呼ぶ。結果を `PluginIntent` に持たせるか、`execute_file_operations` に別引数で渡す。

`Plugin::build_components` に混ぜない（`ComponentKind` 非拡張）。

### B. overlay（各 Skill deploy 後）— install

**場所:** `ComponentDeployment::deploy_skill` の `replace_dir` + frontmatter strip **の後**

必要データ注入（既存型への最小差分）:

```rust
// conversion.rs 案
Skill {
    target_kind: TargetKind,
    plugin_root: Option<PathBuf>,           // または
    // attached: Option<Arc<AttachedResources>>,
}
```

`place_plugin` の Skill 分岐で Hook 同様に `plugin_root: Some(...)` をセット。

overlay ヘルパー（案）:

```rust
fn overlay_plugin_attached_resources(
    fs: &dyn FileSystem,
    attached: &AttachedResources, // plugin_root 相対エントリ
    skill_target_dir: &Path,
) -> Result<Vec<PlacementWarning>>;
// 各 relative: dst = skill_target_dir.join(rel)
//   if fs.exists(dst) → skip + PlacementWarning::Conflict { rel }
//   else if src is dir → fs.copy_dir / ファイル単位 copy_file
```

`DeploymentOutput` に warnings を載せて `PlaceSuccess` → CLI へ。

### C. overlay — enable

**場所:** `execute_file_operations` の `FileOperation::CopyDir` 成功直後

現状 `CopyDir` は Skill 専用。そこに:

```rust
FileOperation::CopyDir { source, target } => {
    fs.replace_dir(source, target.as_path())?;
    // if skill + attached present:
    //   overlay_plugin_attached_resources(fs, attached, target.as_path())?
}
```

そのためには:
- `PluginIntent` に `attached: AttachedResources`（または `plugin_root: PathBuf`）を追加し、`enable_plugin` でセット
- または `FileOperation::CopyDir` に overlay 用メタを足す（破壊的・非推奨）

**共通関数を `deploy_skill` と `execute_file_operations` の両方から呼ぶ**のが二重実装を避ける最善。

### D. やらない方がよい注入

| 候補 | 理由 |
|------|------|
| `Component` に付属を載せる | ComponentKind 汚染・全コンポーネントに無関係データ |
| `PlacementContext` | 配置先決定のみの責務 |
| `ConversionWarning` 拡張 | Hook 専用チャネル・render が Hook 限定 |
| overlay を `replace_dir` 前 | `replace_dir` が消す |

### E. sync（参考）

`src/sync.rs` `execute_create` は Skill に `fs.copy_dir`（replace ではない）。最低限同梱するなら同 overlay 呼び出し。stale 非対称は別 Issue でも可。

---

## 既存型シグネチャ早見

| 型 / 関数 | 場所 |
|-----------|------|
| `ComponentDeployment::{execute, execute_with_fs, deploy_skill}` | `src/component/deployment.rs` |
| `ConversionConfig` | `src/component/deployment/conversion.rs` |
| `DeploymentOutput` / `HookConvertOutput` | `src/component/deployment/output.rs` |
| `place_plugin(request: &PlaceRequest) -> PlaceOutcome` | `src/install.rs` |
| `ScannedPlugin::plugin_root(&self) -> &Path` | `src/install.rs` |
| `PluginManifest::*_dir(&self, base: &Path) -> PathBuf` | `src/plugin/meta/manifest.rs` |
| `PluginIntent::{expand, apply, build_file_operation, create_operation}` | `src/plugin/lifecycle/intent.rs` |
| `execute_file_operations(ExpandOutcome, &Path) -> OperationOutcome` | `src/plugin/lifecycle/intent.rs`（private） |
| `enable_plugin(...) -> OperationOutcome` | `src/application/lifecycle.rs` |
| `FileSystem` | `src/fs.rs` |
| `render_hook_success` / `render_place_success_to_strings` | `src/install/format.rs` / `src/commands/deploy/install.rs` |

---

## 結論（実装方針 1 行）

**列挙は `scan` + manifest 解決パスでプラグイン 1 回。overlay は共通ヘルパーを `deploy_skill`（install）と `execute_file_operations` の CopyDir 後（enable）に差し、plugin root / attached は Hook 先例どおり `ConversionConfig::Skill`（install）と `PluginIntent`（enable）から注入。警告は Hook の `ConversionWarning` ではなく `PlaceSuccess` / `DeploymentOutput` 側の新チャネル。**
