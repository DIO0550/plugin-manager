# Issue #358: CursorTarget を実装する（Skills 配置）

## 背景

Epic #356 Phase 2。`Target` trait を実装する `CursorTarget` を追加し、まず Skills の配置・列挙をサポートする。

blocked_by: #357（`TargetKind::Cursor`）。#338（宣言的ケイパビリティ化）は未着手のため、既存の `gemini_cli.rs` / `antigravity.rs` と同型の実装で進める。

## 前提（#357 を同 PR で実施）

`CursorTarget` は `TargetKind::Cursor` が無いとコンパイルできないため、本作業に #357 を含める。

| 項目 | 内容 |
|------|------|
| `TargetKind::Cursor` | `ValueEnum` / serde は `"cursor"` |
| `as_str()` / `command_format()` / `agent_format()` | Claude Code 互換（`CommandFormat::ClaudeCode` / `AgentFormat::ClaudeCode`） |
| `parse_target` / `all_targets` | `"cursor"` → `CursorTarget` |
| `TargetsConfig::default()` | `Cursor` を追加（GeminiCli 欠落は本 Issue 範囲外・触れない） |
| 網羅 match | `skill_allowed_fields` / `target_display_name` / `cleanup_specs` 等 |

CLI ローカル `TargetKind`（enable/disable/update、#321）は Codex/Copilot のみのまま据え置き。

## Phase 2 スコープ（本 Issue）

Skills のみ。Agents / Commands / Instructions / Hooks は #359〜#361。

### 配置先（現行のフラット 2 階層に合わせる）

仕様書の `<marketplace>/<plugin>/<skill>/` 表記はあるが、既存ターゲットは `flatten_name` 後のフラット配置（`skills/<plugin>_<skill>/`）に統一済み。Cursor も同じ。

| スコープ | パス |
|----------|------|
| Personal | `~/.cursor/skills/<flattened_name>/` |
| Project | `.cursor/skills/<flattened_name>/` |

### 実装ファイル

1. `src/target/env/cursor.rs` — `CursorTarget`（`antigravity.rs` 相当、Skills のみ）
2. `src/target/env/cursor_test.rs` — サポート判定 / スコープ / 配置先 / `list_placed`
3. `src/target/env.rs` / `src/target.rs` — モジュール登録・re-export・`parse_target` / `all_targets`
4. `src/plugin/cache/cleanup.rs` — `cleanup_specs` に `.cursor` + `skills`（ベアリテラル問題 #339 に注意しつつ既存パターン踏襲）
5. 付随: `src/target_test.rs`、`cleanup_test.rs`、`install/format.rs`、`component/convert.rs`

### `skill_allowed_fields`

Cursor は `name` / `description` / `paths` / `disable-model-invocation` / `metadata` をサポートするため、フィールド除去しない（`None`、Antigravity/Copilot と同型）。

## TDD

1. **Red**: `cursor_test.rs` と `TargetKind` / `parse_target` / cleanup の失敗テスト
2. **Green**: 最小実装でパス
3. **Refactor**: 重複が気になる場合のみ（#338 は別 Issue）

## 非スコープ

- Agents / Commands（#359）
- Instructions / AGENTS.md 共有（#360）
- Hooks / `hooks.json` マージ（#361）
- ドキュメント最終整合（#362）
- #338 の宣言的ケイパビリティ化
- `TargetsConfig::default()` への GeminiCli 追加（別途ドリフト修正）

## 検証

```bash
cargo fmt
cargo check
cargo test cursor -- --nocapture
cargo test cleanup -- --nocapture
cargo test --lib
```
