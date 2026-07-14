# レビュー: Issue #342

| 項目 | 内容 |
|------|------|
| Issue | [#342 [refactor/domain] enable/disable ハンドラの95%コピペを解消し、状態遷移を TargetStatus enum としてドメイン層に移管する](https://github.com/DIO0550/plugin-manager/issues/342) |
| レビュー日 | 2026-07-14 |
| 対象ブランチ | `main`（`ea40cdf` 時点） |
| ラベル | `refactor` |

## サマリー

Issue #342 は、enable/disable コマンドの重複実装と、プラグインステータス（`enabled` / `disabled`）の管理漏れ・型安全性不足を解消するリファクタリング課題である。根本原因は「状態遷移ルールがユースケース層に存在せず CLI ハンドラに漏れている」ことと、ステータスが生文字列で全層を流れていることにある。

現状の `main` では Issue が指摘する問題は**未解消**。本ドキュメントは現状分析・実装方針・作業分解をまとめたものである。

## Issue が指摘する問題

### 1. enable.rs / disable.rs の行単位コピペ

| 関数 | enable.rs | disable.rs | 差分 |
|------|-----------|------------|------|
| `run` | L43–88 | L43–89 | ドメイン呼び出し名と表示語のみ |
| `update_status_after_*` | L96–110 | L97–111 | `"enabled"` / `"disabled"` リテラルのみ |
| `display_result` | L119–138 | L120–139 | 同構造 |

両ファイルとも `run` 内で `enable_plugin` / `disable_plugin` 実行後に CLI 側で `.plm-meta.json` を更新しており、**配置とステータス更新が分離**している。

### 2. 状態遷移ロジックの漏出と三重定義

`enable_plugin` / `disable_plugin`（`application/lifecycle.rs`）はファイル配置・除去のみ行い、`.plm-meta.json` は更新しない。`set_status` の書き込みは以下 3 系統に独立実装されている。

| 系統 | ファイル | 行 | 役割 |
|------|---------|-----|------|
| enable/disable CLI | `commands/lifecycle/enable.rs` | L101 | 成功ターゲットを `"enabled"` に |
| enable/disable CLI | `commands/lifecycle/disable.rs` | L102 | 成功ターゲットを `"disabled"` に |
| install | `install.rs` | L434–441, L476 | 失敗ターゲット除外・no-op skip 付きで `"enabled"` |
| update | `plugin/lifecycle/update.rs` | L548, L1042 | 失敗ターゲットを `"disabled"` に |

この分散により、「配置は成功したが `statusByTarget` が未更新」といった不整合が起きうる。また enable/disable の修正が常に両ファイルへ同時適用される保証がない。

### 3. ステータスの primitive obsession

```rust
// plugin/meta/meta.rs
pub status_by_target: HashMap<String, String>,

pub fn is_enabled(&self, target: &str) -> bool {
    self.get_status(target) == Some("enabled")  // 文字列比較
}
```

- 不正な文字列（`"enable"` 等）を型が防げない
- bool → ラベル変換が `commands/list/table.rs`（L53–58）と `commands/info/table.rs`（L180–184）に二重実装されている

## 現状コードの詳細分析

### application/lifecycle.rs — ステータス更新なし

`enable_plugin` / `disable_plugin` は `PluginIntent::apply()` の結果をそのまま返す。メタ更新の責務は CLI ハンドラ側にある。

```43:88:src/commands/lifecycle/enable.rs
pub async fn run(args: Args) -> Result<(), String> {
    // ...
    let result = enable_plugin(/* ... */);

    let plugin_path = cache.plugin_path(Some(marketplace), &args.name);
    update_status_after_enable(&plugin_path, &result);  // ← CLI 層でステータス更新
    // ...
}
```

`disable.rs` も同一構造（`update_status_after_disable`）。

### install.rs — より厳密な昇格条件

install 経路は enable ハンドラより慎重なロジックを持つ。

- 同一ターゲットに failure があれば `enabled` に昇格しない
- 既に `"enabled"` なら no-op で skip（mtime 汚染防止）
- Hook 配置時は `managedFiles` も同時更新

enable/disable ハンドラの `update_status_after_*` にはこれらのガードがない。ただし `intent.rs::execute_file_operations` では同一ターゲット内で 1 件でも失敗すると `record_error` のみが記録されるため、**同一ターゲット内の部分成功で誤って enabled になるリスクは現状ない**。

### ローカル TargetKind の重複（#321 隣接）

`enable.rs` / `disable.rs` / `update.rs` がそれぞれ Codex/Copilot のみの独自 `TargetKind` を定義している。`crate::target::TargetKind`（4 ターゲット対応）は main に存在するが、CLI ハンドラ側では未使用。

## Issue の提案と推奨実装方針

### 提案 1: `TargetStatus` enum の導入

```rust
// plugin/meta/meta.rs（案）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetStatus {
    Enabled,
    Disabled,
}

impl TargetStatus {
    pub fn as_str(&self) -> &'static str { /* ... */ }
}

impl fmt::Display for TargetStatus { /* ... */ }

// PluginMeta
pub status_by_target: HashMap<String, TargetStatus>,
```

- serde `rename_all = "lowercase"` で既存 JSON（`"enabled"` / `"disabled"`）との後方互換を維持
- `get_status` → `Option<TargetStatus>`、`set_status` → `TargetStatus` 引数に変更
- Issue 原文の `HashMap<TargetId, TargetStatus>` は、`TargetId` 廃止後（`6203d2f`）は `HashMap<String, TargetStatus>` または `TargetKind` キー化を検討。キー型変更は別ステップでもよい

### 提案 2: 状態遷移をユースケース層へ移管

配置先: `application/lifecycle.rs`（Issue タイトルの「ドメイン層」は、Feature ベース構成ではアプリケーション層＝ユースケース層と解釈するのが妥当）。

```rust
// application/lifecycle.rs（案）
fn update_meta_status(
    cache: &dyn PackageCacheAccess,
    plugin_name: &str,
    marketplace: Option<&str>,
    result: &OperationOutcome,
    status: TargetStatus,
) { /* affected_targets の成功ターゲットに set_status */ }

pub fn enable_plugin(...) -> OperationOutcome {
    let result = intent.apply();
    update_meta_status(cache, plugin_name, marketplace, &result, TargetStatus::Enabled);
    result
}
```

- CLI ハンドラから `update_status_after_*` を削除
- `uninstall_plugin` → `disable_plugin` 経路でも自動的にステータス更新される
- 書き込みエラーは best-effort（警告出力、操作全体は失敗扱いにしない）— 現行ハンドラと同じ

### 提案 3: enable/disable ハンドラの共通化

`commands/lifecycle/toggle.rs` を新設し、操作パラメータで差分を吸収する案が妥当。

| 差分要素 | Enable | Disable |
|---------|--------|---------|
| ユースケース呼び出し | `enable_plugin` | `disable_plugin` |
| 表示語（過去形） | `Enabled` | `Disabled` |
| コンポーネント操作 | `deployed to` | `removed from` |
| キャッシュ不在時 | エラーのみ | エラー + Hint |

`enable.rs` / `disable.rs` は `Args` 構造体と薄い `run()` のみに縮小。1 ファイル統合より、Clap の引数定義を分離した 3 ファイル構成の方が可読性が高い。

## 作業分解

### Phase 1: 型安全化（影響範囲: meta + 呼び出し元）

1. `TargetStatus` enum を `plugin/meta/meta.rs` に追加
2. `PluginMeta::status_by_target` の型変更
3. 全 `set_status` / `get_status` / `is_enabled` 呼び出し元を `TargetStatus` に置換
   - `install.rs`, `plugin/lifecycle/update.rs`
   - `commands/lifecycle/enable.rs`, `disable.rs`
   - テスト: `meta_test.rs`, `install_test.rs`, `codex_test.rs` 等
4. `meta_test.rs` で serde 後方互換テストを維持

### Phase 2: 状態遷移の集約

1. `application/lifecycle.rs` に `update_meta_status` を追加
2. `enable_plugin` / `disable_plugin` 末尾から呼び出し
3. CLI ハンドラの `update_status_after_*` を削除
4. `application/lifecycle_test.rs` にメタ更新テストを追加
   - enable 成功 → `TargetStatus::Enabled`
   - disable 成功 → `TargetStatus::Disabled`
   - `affected_targets` 空 → メタ未変更
   - 部分成功 → 成功ターゲットのみ更新

### Phase 3: ハンドラ共通化 + #321

1. `commands/lifecycle/toggle.rs` に `run_toggle` / `ToggleOp` を実装
2. `enable.rs` / `disable.rs` を薄いラッパーに縮小
3. ローカル `TargetKind` を削除し `crate::target::TargetKind` に統一
4. 既存統合テスト（`enable_test.rs`, `disable_test.rs`）が通ることを確認

### Phase 4（follow-up / 別 Issue 可）: 残存する三重定義の統合

install / update 経路の `set_status` ロジックは Phase 2 後も独立したまま残る。

| 経路 | 固有ロジック | 統合方針 |
|------|------------|---------|
| install | 失敗ターゲット除外、no-op skip、`managedFiles` 更新 | 共有ヘルパー `promote_target_status` 等へ抽出 |
| update | 失敗ターゲットを `Disabled` に | 同上 |

表示層の bool → ラベル変換（`list/table.rs`, `info/table.rs`）は `TargetStatus::Display` または `TargetStatus::from_deployed(bool)` で一本化可能。

## リスクと注意点

### 振る舞い互換性

| シナリオ | 現行振る舞い | 移管後に維持すべき振る舞い |
|---------|------------|------------------------|
| 全ターゲット成功 | メタ更新 + 成功表示 | 同左 |
| ターゲット横断部分成功 | 成功ターゲットのみメタ更新 | 同左 |
| 同一ターゲット内部分失敗 | メタ未更新 | 同左（intent の error-only 記録に依存） |
| `--target` で非対応ターゲット | Skipped 表示、メタ未更新 | 同左 |
| メタ書き込み失敗 | 警告出力、CLI は成功扱い | 同左 |

### antigravity / gemini の CLI 対応

ローカル `TargetKind` 削除により `--target antigravity` / `--target gemini` が有効になる。破壊的変更ではないが、CLI ヘルプ文言の更新が必要。

### 用語

Issue タイトルは「ドメイン層」だが、実装先は `application/lifecycle.rs`（ユースケース層）。Feature ベース構成（`AGENTS.md`）との整合性は取れている。

## 関連 Issue

| Issue | 関係 |
|-------|------|
| [#321](https://github.com/DIO0550/plugin-manager/issues/321) | ローカル `TargetKind` enum の統一。同時着手が効率的 |
| [#331](https://github.com/DIO0550/plugin-manager/issues/331) | enabled 判定の曖昧さ。状態管理の隣接課題。Phase 4 後に整合確認 |

## 期待効果（Issue より）

- 「配置は成功したがステータス未更新」といった不整合の入り口が閉じる
- enable/disable の修正が常に両方へ同時適用される
- 不正ステータス文字列の混入を型で防止できる
- `list` / `info` 表示を含めたステータス表現の一本化が可能になる

## 結論

Issue #342 は妥当なリファクタリング課題であり、Phase 1–3 を一括で着手するのが効率的（#321 も同時対応）。Phase 4（install/update 統合・表示層）はスコープを分けてもよいが、Issue クローズ条件としては Phase 2 までが最低ライン、Phase 3 までで Issue 本体の目的は達成できる。
