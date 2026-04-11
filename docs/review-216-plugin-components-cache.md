# コードレビュー: Plugin components キャッシュ導入 (#216)

レビュー対象コミット: `be3e343` → `be3e343` (HEAD)

## 総合評価

✅ **承認** — 設計・実装・テストともに良好。前回の自動レビューコメント（重複インポート問題）は誤検知であることを確認済み。

---

## 前回コメントへの見解

### `plugin_content_test.rs` の `use crate::...` が重複インポートか？

> `use super::*;` already brings `Component`, `ComponentKind`, and `PluginManifest` into scope ... cause duplicate-import compile errors (E0252).

**この指摘は誤りです。**

`use super::*;` が取り込むのは、親モジュールで **`pub use` されている** アイテムのみです。`plugin_content.rs` の冒頭インポートは通常の `use`（再エクスポートなし）であるため、テストモジュールから `use super::*;` を呼んでも `Component`・`ComponentKind`・`PluginManifest` はスコープに入りません。

```rust
// plugin_content.rs — pub use ではない
use crate::component::{Component, ComponentKind};  // 非公開インポート
use crate::plugin::PluginManifest;                  // 非公開インポート
```

そのためテストファイルの明示的インポートは **必要** であり、削除すると `E0412 (cannot find type)` エラーになります。`cargo check --tests` で E0252 は発生しないことを確認済みです。

---

## 変更内容のレビュー

### `src/plugin/plugin_content.rs`

| 観点 | 評価 | コメント |
|------|------|---------|
| `Plugin::new()` コンストラクタ | ✅ | 構築時スキャン、スナップショット保証、不変条件の保護ができている |
| フィールド private 化 | ✅ | 外部から不整合な状態を作れなくなった |
| `name()` → `manifest.name` に一本化 | ✅ | `name` 引数の削除（ef677ad）で二重管理が解消された |
| `components()` 戻り値 `&[Component]` | ✅ | コピーなしの参照返却で効率的 |
| `resolve_instruction_path` 修正 | ✅ | `manifest.instructions = Some(...)` 時に AGENTS 特別扱いをスキップする正しい実装 |
| `resolve_*` 関連関数化 | ✅ | `self` 未確定の `new()` 内から呼べるよう適切に整理された |
| `skills_dir`/`agents_dir` 等の `pub` メソッド残存 | ⚠️ | 現状は外部利用がないようであれば `pub(crate)` への絞り込みを検討できる（機能上の問題なし） |

### `src/plugin/plugin_content_test.rs`

| 観点 | 評価 | コメント |
|------|------|---------|
| テスト分離（`TempDir` 使用） | ✅ | FS 依存を排除 |
| コンポーネント種別の網羅 | ✅ | Skill/Agent/Command/Instruction/Hook の全種をカバー |
| `test_plugin_new_with_instructions_dir_containing_agents_md` | ✅ | `resolve_instruction_path` バグの回帰テストとして有効 |
| `test_plugin_components_returns_slice` | ✅ | 戻り値型 `&[Component]` を型注釈で明示している |
| `test_plugin_clone_preserves_components` | ✅ | `Clone` でのスナップショット保持を検証 |
| インポート（lines 4–5） | ✅ | 前述のとおり必要なインポートであり削除不可 |

### `src/plugin/marketplace_content.rs`

| 観点 | 評価 | コメント |
|------|------|---------|
| `Plugin::new()` 経由への統一 | ✅ | 構造体リテラル廃止完了 |
| `components()` の `.flat_map(...).cloned().collect()` | ✅ | `&[Component]` 返却に合わせた適切な変更 |
| アクセサ経由への切り替え (`name()`/`path()`/`manifest()`) | ✅ | フィールド private 化に伴う必要な対応 |

### `src/application/plugin_deployment.rs`

| 観点 | 評価 | コメント |
|------|------|---------|
| `Plugin::new()` + `.components().to_vec()` | ✅ | 外部 API `Vec<Component>` を維持しつつ内部実装を整合 |

### `src/plugin/marketplace_content_test.rs`

| 観点 | 評価 | コメント |
|------|------|---------|
| 固定パス → `TempDir` 移行 | ✅ | FS 依存の排除 |
| `test_marketplace_content_components_from_cached_package` 追加 | ✅ | `CachedPackage` 変換後のコンポーネント保持を検証 |

---

## テスト結果

```
running 12 tests
test plugin::plugin_content::plugin_content_test::test_plugin_new_empty_dir ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_agent_single_file ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_components_returns_slice ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_clone_preserves_components ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_agents ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_command_md_fallback ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_commands ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_default_agents_md_instruction ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_instructions ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_hooks ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_skills ... ok
test plugin::plugin_content::plugin_content_test::test_plugin_new_with_instructions_dir_containing_agents_md ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

全 12 件パス。`cargo check --tests` エラー 0。

---

## 既知の制限事項（設計上の仕様）

- `Plugin` は構築時点の FS スナップショットを保持する。構築後に FS が変化しても `components()` の結果は更新されない。
- 現行アーキテクチャでは `Plugin` は短命（`CachedPackage` 変換のたびに再構築）なため実害なし。

---

## 結論

前回の自動レビューで指摘された重複インポート問題（E0252）は誤検知です。コードは正しく動作しており、変更に対する指摘事項はありません。マージ可能です。
