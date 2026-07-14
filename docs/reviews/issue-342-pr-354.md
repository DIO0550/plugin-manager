# レビュー: Issue #342 / PR #354

| 項目 | 内容 |
|------|------|
| Issue | [#342 [refactor/domain] enable/disable ハンドラの95%コピペを解消](https://github.com/DIO0550/plugin-manager/issues/342) |
| PR | [#354 refactor: TargetStatus enum 導入・状態遷移のドメイン層移管・enable/disable コピペ解消](https://github.com/DIO0550/plugin-manager/pull/354) |
| ブランチ | `copilot/refactor-domain-enable-disable-handler` |
| レビュー日 | 2026-07-14 |
| 対象コミット | `01222b8`（`9fcbada` Initial plan 含む） |
| 総合判定 | **条件付き承認（Approve with comments）** |

## サマリー

PR #354 は Issue #342 の中核要件（`TargetStatus` enum 導入、enable/disable ハンドラ共通化、状態遷移の CLI ハンドラからの除去）を概ね満たしている。`enable.rs` / `disable.rs` の 95% コピペは `toggle.rs` への抽出で解消され、`.plm-meta.json` の `statusByTarget` 更新は `application/lifecycle.rs` の `update_meta_status` に集約された。

一方で、Issue が挙げた「3 系統の独立実装」の完全統合や表示層の primitive obsession 解消は未完であり、テスト追加も不足している。Draft PR としてマージ前に下記コメントへの対応を推奨する。

## Issue 要件との対応表

| # | Issue 提案 | PR での対応 | 判定 |
|---|-----------|------------|------|
| 1 | `TargetStatus` enum 導入、`status_by_target` の型安全化 | `plugin/meta/meta.rs` に `TargetStatus { Enabled, Disabled }` を追加。serde `rename_all = "lowercase"` で JSON 互換維持 | ✅ 完了 |
| 2 | 状態遷移を `enable_plugin` / `disable_plugin` へ移管 | `application/lifecycle.rs::update_meta_status` を両関数末尾から呼び出し | ✅ 完了（ただし install / update 経路は未統合） |
| 3 | enable/disable ハンドラ共通化 | `commands/lifecycle/toggle.rs` に `run_toggle` を抽出。各ハンドラは薄いラッパー | ✅ 完了（1 ファイル統合ではなく 3 ファイル構成） |
| — | #321 ローカル `TargetKind` 統一 | `crate::target::TargetKind`（4 ターゲット）に統一 | ✅ 完了 |
| — | `list` / `info` の bool→ラベル二重実装解消 | 未対応（`commands/list/table.rs`, `commands/info/table.rs` に `"enabled"` / `"disabled"` リテラルが残存） | ⚠️ スコープ外として許容可 |
| — | install / update の `set_status` 統合 | `TargetStatus` 型への置換のみ。ロジックは `install.rs` / `plugin/lifecycle/update.rs` に残存 | ⚠️ 部分対応 |

## 変更の評価

### 良い点

1. **`TargetStatus` enum の設計が妥当**
   - `as_str()` / `Display` / serde を備え、既存 JSON（`"enabled"` / `"disabled"`）との後方互換を維持している。
   - `get_status` の戻り値を `Option<TargetStatus>` に変更し、比較箇所の型安全性が向上した。

2. **ハンドラ共通化のアプローチが実用的**
   - `ToggleOp` で表示語（過去形・動詞・コンポーネント操作説明）を集約し、`run_toggle` + クロージャで `enable_plugin` / `disable_plugin` を差し替える設計は、過度な抽象化を避けつつ重複を除去できている。
   - disable 固有のキャッシュ不在ヒントは `map_err` で局所的に付与しており、既存テスト（`disable_test.rs::test_disable_cache_not_found_shows_error_once`）との互換を維持している。

3. **状態遷移の移管先が適切**
   - CLI ハンドラから `update_status_after_*` を削除し、`enable_plugin` / `disable_plugin` 内で `update_meta_status` を呼ぶことで、「配置成功だがメタ未更新」という不整合の入り口が閉じられる。
   - `uninstall_plugin` → `disable_plugin` の呼び出し経路でも、disable 時のステータス更新が自動的に行われる。

4. **#321 との同時対応**
   - Codex/Copilot のみだったローカル `TargetKind` を削除し、CLI で antigravity / gemini も指定可能になった。Issue #342 単体より効率的な着手。

5. **既存テストの追随**
   - `meta_test.rs`, `install_test.rs`, `codex_test.rs` 等で `TargetStatus` への置換が行われており、型変更に伴うテスト破損は抑えられている。

### 懸念点・改善提案

#### [重要] `update_meta_status` のユニットテストがない

`application/lifecycle_test.rs` には enable/disable の成功・失敗テストがあるが、**メタ更新の振る舞いを検証するテストが追加されていない**。Issue の主目的は状態遷移の集約であるため、以下のテスト追加を推奨する。

- enable 成功後に `statusByTarget` が `Enabled` になること
- disable 成功後に `Disabled` になること
- `affected_targets` が空（全ターゲット失敗）のときメタが書き換わらないこと
- 部分成功時に成功ターゲットのみ更新されること

#### [中] install / update 経路とのロジック重複が残存

Issue が指摘した「3 系統の独立実装」のうち、enable/disable 経路のみが `update_meta_status` に集約された。以下は依然として独立している。

| 経路 | ファイル | 役割 |
|------|---------|------|
| enable/disable | `application/lifecycle.rs::update_meta_status` | 操作後に unconditional で `set_status` |
| install | `install.rs::update_meta_after_place` | 失敗ターゲットを除外し、既に Enabled なら skip |
| update | `plugin/lifecycle/update.rs` | 失敗ターゲットを `Disabled` に設定 |

enable/disable の `update_meta_status` は `install.rs` と比べて **no-op スキップ（mtime 汚染防止）や失敗ターゲット除外ロジックを持たない**。ただし `intent.rs::execute_file_operations` の実装では、同一ターゲット内で 1 件でも失敗すると `record_error` のみが記録され `record_success` は呼ばれないため、**同一ターゲット内の部分成功で誤って Enabled になるリスクは現状ない**。

一方、ターゲット横断の部分成功（codex 成功・copilot 失敗）では、成功ターゲットに対して無条件で `set_status` する。これは旧ハンドラと同じ振る舞いであり、リグレッションではない。将来的には `install.rs` のガード条件を共有ヘルパーに抽出する follow-up が望ましい。

#### [中] 表示層の primitive obsession が残存

Issue #342 の「bool→ラベル変換の二重実装」は未対応。

```rust
// commands/list/table.rs:53-58
fn status_label(enabled: bool) -> &'static str { ... }

// commands/info/table.rs:180-184
let status = if info.installed.enabled() { "enabled" } else { "disabled" };
```

`TargetStatus::Display` または `TargetStatus::from_bool(enabled)` のような変換を導入すれば、Issue の期待効果を完全に達成できる。本 PR のスコープ外として許容するなら、Issue #342 をクローズする際に follow-up Issue を切ることを推奨。

#### [低] `HashMap` のキーが依然 `String`

Issue 原文では `HashMap<TargetId, TargetStatus>` を提案していたが、main ブランチでは `TargetId` が `TargetKind` に統合済み（`6203d2f`）。本 PR ではキー型の変更は行われておらず、`set_status(target_name, status)` の `target` 引数も `&str` のまま。型安全性の改善余地は残るが、本 PR のスコープとしては許容範囲。

#### [低] serde デシリアライズの堅牢性

`TargetStatus` に未知の文字列（例: `"unknown"`）が JSON に含まれると `load_meta` が失敗する。現行実装でも `String` 型だったため新規リスクではないが、マイグレーション期間中の手動編集や外部ツール生成 JSON に対するエラーメッセージの改善は将来課題。

#### [低] Draft PR / コミット構成

- PR は Draft のまま。マージ前に Draft 解除と CI グリーンの確認が必要。
- `9fcbada Initial plan` コミットは実装に寄与しないノイズ。squash merge なら問題ないが、rebase 時の整理を検討。

#### [情報] 用語のずれ

Issue タイトルは「ドメイン層に移管」だが、実装は `application/lifecycle.rs`（アプリケーション層）に配置されている。Feature ベース構成方針（`AGENTS.md`）との整合性は取れているが、PR 説明の「ドメイン層」表現は「アプリケーション層（ユースケース）」に修正すると正確。

## 振る舞いの互換性確認

| シナリオ | 旧実装 | PR 実装 | 互換 |
|---------|--------|---------|------|
| キャッシュ不在（enable） | エラー 1 回 | `run_toggle` 内で同様 | ✅ |
| キャッシュ不在（disable） | エラー + Hint | `map_err` で Hint 付与 | ✅ |
| 全ターゲット成功 | メタ更新 + 成功表示 | `update_meta_status` + `display_result` | ✅ |
| ターゲット横断部分成功 | 成功ターゲットのみメタ更新 | 同左（`affected_targets.target_names()` ベース） | ✅ |
| 同一ターゲット内部分失敗 | メタ未更新（error のみ記録） | 同左（intent が success を記録しない） | ✅ |
| `--target` で非対応ターゲット | Skipped 表示、メタ未更新 | 同左 | ✅ |
| antigravity / gemini 指定 | 旧: ローカル enum で不可 | 新: `TargetKind` で可能 | 🔄 機能拡張（破壊的変更ではない） |

## テスト実行

レビュー環境（Cargo 1.83.0）では依存クレート `time-core` の `edition2024` 要求により `cargo test` が実行不能だった。CI 上でのグリーン確認をマージ条件とする。

ローカルで確認可能な既存テスト:

- `enable_test.rs` / `disable_test.rs`: 統合テスト（キャッシュ不在・操作失敗時の stderr）
- `meta_test.rs`: serde 互換・`is_enabled` / `enabled_targets`
- `install_test.rs`: install 後のステータス昇格条件

## 推奨アクション

### マージ前（推奨）

1. `application/lifecycle_test.rs` に `update_meta_status` の振る舞いテストを追加
2. Draft を解除し CI を通す
3. PR 説明の「ドメイン層」→「アプリケーション層（`application/lifecycle.rs`）」に修正

### マージ後（follow-up）

1. `install.rs` / `plugin/lifecycle/update.rs` の `set_status` ロジックを `update_meta_status` と統合または共有化（Issue #342 残タスク）
2. `list` / `info` 表示の `TargetStatus::Display` 一本化
3. #331（enabled 判定の曖昧さ）との整合確認

## 結論

PR #354 は Issue #342 の**主要目的を達成**しており、コピペ解消・型安全化・状態遷移の集約という方向性は正しい。残タスク（install/update 統合、表示層、テスト追加）は follow-up として切り出してもよいが、**`update_meta_status` のテスト追加**は本 PR に含めることを強く推奨する。

**判定: 条件付き承認** — 上記マージ前推奨事項（特に lifecycle テスト）への対応後にマージ可。
