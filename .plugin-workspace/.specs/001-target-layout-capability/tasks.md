# Task: target/env impl ベース共通骨格抽出

> 方針: bottom-up impl 抽出（2026-07-21 方針転換済み）
> 旧計画（top-down DSL 設計）のタスクリストは廃止。

---

## Phase A: 5 impl 差分表確定（コード変更なし）

- □ `exploration-report.md §8` の差分表（BL-005 相当）が完成しているか確認
- □ `antigravity_test.rs`, `gemini_cli_test.rs`, `codex_test.rs`, `copilot_test.rs`, `cursor_test.rs` の既存テストが全 green であることを `cargo test` で確認
- □ 各 env の `supported_components` / `can_place` / `placement_location` の期待値を差分表と照合し、乖離がないか確認
- □ `FakeTarget`（`src/sync_test.rs` / `src/sync/endpoint/endpoint_test.rs`）の実装内容を確認し、Phase D の影響範囲を把握

---

## Phase B: `list_placed` 共通骨格抽出

> **TDD**: Red（テスト先行）→ Green（実装）→ Refactor の順で進める

### 共通ヘルパ追加

- □ RED: `scan_and_filter(base, subdir, filter)` ヘルパの単体テストを `src/target/placed/` に追加
  - TempDir に `skills/plugin_skill/SKILL.md` を作成し、`scan_and_filter(&base, "skills", filter_skill_dir)` → `["plugin_skill"]` を確認
  - TempDir に空ディレクトリのみで `scan_and_filter` → `[]` を確認
- □ GREEN: `src/target/placed/list_helpers.rs`（または `scanner.rs` への追加）に `scan_and_filter` を実装
- □ REFACTOR: 関数名・引数の型の整理

### Antigravity 移行（Phase B の先頭）

- □ `antigravity.rs::list_placed` を `scan_and_filter` 利用に書き換え
- □ `cargo test target::env::antigravity` で既存テスト全 green 確認

### Gemini CLI 移行

- □ `gemini_cli.rs::list_placed` の Skill 部分を `scan_and_filter` 利用に書き換え
  - Instruction 分岐は Phase D で対応するため、`placed_common::list_instruction` の呼び出しはそのまま残す
- □ `cargo test target::env::gemini_cli` で既存テスト全 green 確認

### Codex / Copilot / Cursor も順次適用

- □ `codex.rs::list_placed` の Skill / Agent / Hook の各分岐を `scan_and_filter` 利用に書き換え（Instruction は Phase D）
- □ `copilot.rs::list_placed` を `scan_and_filter` 利用に書き換え
- □ `cursor.rs::list_placed` を `scan_and_filter` 利用に書き換え（Cursor は `is_plain_markdown` フィルタがあるため慎重に）
- □ `cargo test` で全テスト green 確認

---

## Phase C: `filter_component` Skill アーム共通化

- □ RED: `filter_skill_dir` の単体テストを書く
  - SKILL.md あり → `Some("plugin_skill")`
  - SKILL.md なし → `None`
  - ファイル（非ディレクトリ）エントリ → `None`
- □ GREEN: `src/target/placed/filter.rs`（または `list_helpers.rs`）に `filter_skill_dir` 関数を追加
- □ REFACTOR: 各 env の `filter_component` から Skill アームを削除し、`filter_skill_dir` 呼び出しに置換（antigravity → gemini → codex → copilot → cursor）
- □ Skill アームを削除後も差分アーム（Agent `.agent.md` など）は各 env に残す
- □ `cargo test` で全テスト green 確認

---

## Phase D: `supports_scope` ダミープロービング廃止

### `supports_scope` デフォルト実装の書き換え

- □ `src/target.rs::supports_scope` のダミー実装（行 258-266）の廃止方式を確定
  - **方式1**: `Target` trait に `fn can_place_scope(kind, scope) -> bool` を追加（デフォルト: `supported_components().contains(&kind)`）し、`supports_scope` でそれを呼ぶ
  - **方式2**: `supported_combinations() -> &[(ComponentKind, Scope)]` static スライスを返す関数を追加
- □ 選択した方式で `supports_scope` を実装
- □ Copilot / Cursor の scope 制約を `can_place_scope` override で表現
- □ `grep -r 'dummy_origin\|PluginOrigin.*"test"\|PlacementContext.*"test"' src/target.rs` でダミーコードが消えているか確認

### `placed_common::list_instruction` のダミー廃止

- □ RED: `list_instruction_at(path: &Path, filename: &str)` の単体テスト
  - ファイルあり → `vec!["AGENTS.md"]`
  - ファイルなし → `vec![]`
- □ GREEN: `src/target/placed/placed_common.rs`（または `list_helpers.rs`）に `list_instruction_at` を追加
- □ REFACTOR: 各 env の `list_placed` の Instruction 分岐を `list_instruction_at` に置き換え
  - Gemini: `list_instruction_at(&base.join("GEMINI.md"), "GEMINI.md")` または scope に応じた分岐
  - Codex: scope に応じて `project_root.join("AGENTS.md")` または `base.join("AGENTS.md")`
  - Copilot: `base.join("copilot-instructions.md")`
  - Cursor: `project_root.join("AGENTS.md")`（Project のみ）
- □ 旧 `list_instruction`（ダミー使用）がすべて `list_instruction_at` に置き換わったら旧関数を削除
- □ `cargo test` で全テスト green 確認（Instruction list の期待値が変わっていないことを確認）
- □ `grep -r '"test"' src/target/placed/placed_common.rs` でダミーが消えているか確認

---

## Phase E: `placement_location` 共通パターンをヘルパ化

- □ RED: `skill_dir` / `agent_file` / `instruction_file` ヘルパの単体テスト
- □ GREEN: `src/target/placed/placement_helpers.rs` に上記 3 関数を追加
- □ REFACTOR: 各 env の `placement_location` で共通パターンを対応するヘルパ呼び出しに置き換え
  - **Antigravity / Gemini / Codex / Copilot**: Skill → `skill_dir(&base, name)`
  - **Cursor**: Skill → `original_name` 取得後 `skill_dir(&base, original_name)`（例外は override に残す）
  - **Codex / Copilot / Cursor**: Agent → `agent_file(&base, name)`
  - **Gemini / Codex / Cursor**: Instruction → `instruction_file(scope, project_root, &base, "<FILENAME>")`
- □ `cargo test` で全テスト green 確認

---

## Phase F: 薄い定数まとめ（省略可能）

> Phase B〜E 完了後に、各 env ファイルに残った純粋データ差分（文字列リテラル・static スライス）が多い場合のみ実施する。コードが十分に整理されていれば本 Phase はスキップしてよい。

- □ Phase B〜E 完了後に各 env ファイルの残り差分を評価
- □ （必要なら）各 env の差分を `struct EnvConfig` または薄い `TargetLayout` const に集約
- □ （必要なら）定数からヘルパを呼ぶ形に `placement_location` / `list_placed` を再整理

---

## Phase G: ドキュメント同期・死コード削除

- □ 各 env の `can_place` プライベート関数を削除（Phase D の `can_place_scope` に統一後）
- □ 各 env の `base_dir` プライベート関数を削除（直接 `paths::base_dir` を呼ぶ形に変更）
- □ 各 env の `filter_component` プライベート関数を削除（Phase C の共通ヘルパ + インライン差分アームに統一後）
- □ `placed_common::list_instruction`（ダミー使用）を削除（Phase D 完了後）
- □ `docs/target-layout-refactor/` を `docs/old/` にアーカイブ
- □ `docs/architecture/` に共通ヘルパ関数（`scan_and_filter` / `filter_skill_dir` / `skill_dir` 等）の説明を追記

---

## Verification

- □ `cargo test` で全テスト（既存 + 新規追加分）が green
- □ `cargo build` でコンパイルエラーなし
- □ `cargo clippy` で新規 warning なし
- □ `cargo fmt` でフォーマット適用済み
- □ `grep -r '"test"' src/target.rs src/target/placed/placed_common.rs` でダミーコードがゼロ（G-001, G-002）
- □ `list_placed` の骨格コピペが各 env ファイルから消えている（G-003）
- □ `filter_component` の Skill アームが各 env ファイルから消えている（G-004）
- □ `placement_location` の Skill/Agent/Instruction 共通パターンが共通ヘルパを呼んでいる（G-005）
- □ Phase A の差分表（BL-005 相当）と全テストの期待値が一致している（G-006）
