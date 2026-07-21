# Task: target/env 宣言的ケイパビリティ集約

## Research & Planning (Phase 0: 受け入れテスト棚卸し)

> **Note (WARN)**: Phase 0 は Phase 1 の前提。不足テストがあれば独立 PR で先に追加すること。

- □ BL-005 現状マトリクス（期待値）を確認し、5 ターゲットのテストカバレッジ棚卸し
- □ 既存テストで `supports` / `supports_scope` / `placement_location` / `list_placed` が各組み合わせをカバーしているか確認（T-SUP / T-SCOPE / T-PATH / T-LIST）
- □ Cursor の `original_name=None` ケース・Copilot Personal スコープ制約のテスト有無確認
- □ `FakeTarget`（sync_test, endpoint_test）への影響範囲確認

## Phase 1: TargetLayout モデル + 導出ヘルパ（TDD）

### モデル定義

- □ RED: `ScopeSet::contains` の単体テストを `model_test.rs` に書く
- □ GREEN: `src/target/layout/model.rs` に `TargetLayout` / `ComponentCapability` / `ScopeSet` / `PlacementRule` / `DiscoverRule` / `NamingPolicy` / `InstructionProjectLocation` を定義
- □ REFACTOR: 型名・モジュール分割の整理

### 導出ヘルパ

- □ RED: 架空の `TargetLayout` で `derive_supports_scope` のテストを `derive_test.rs` に書く
- □ GREEN: `derive_supports_scope` 実装
- □ RED: `derive_placement_location` のテスト（ComponentDir / ComponentFile / FixedAtBase / InstructionFile / scope 外 / OriginalNameRequired）
- □ GREEN: `derive_placement_location` 実装（`resolve_base` / `resolve_placement` / `resolve_name` ヘルパ含む）
- □ RED: `derive_list_placed` のテスト（SkillManifestDir / SuffixFile / PlainMarkdownFile / ExactFile / JsonSuffixFile / InstructionExists / scope 外 / 空ディレクトリ）
- □ GREEN: `derive_list_placed` 実装
- □ REFACTOR: ヘルパ関数の整理・重複除去
- □ `src/target/layout.rs` + `src/target.rs` の `mod layout;` 追加・モジュール接続

### Phase 1 検証

- □ `cargo test target::layout` で Phase 1 の新規テストが green
- □ `cargo test` で既存テスト全体も引き続き green
- □ **統合確認**: `derive_list_placed` の `InstructionExists` ルールを実際のファイルシステム（TempDir）で検証するテストを `derive_test.rs` に追加（ファイルあり/なしの両ケース）

## Phase 2: Antigravity / Gemini CLI 移行

### Antigravity

- □ `antigravity.rs` に `LAYOUT` 定数（`Nested { parent: ".gemini", child: "antigravity" }` + Skill capability）を追加
- □ `impl Target for AntigravityTarget` の `supports_scope` を `derive_supports_scope(&LAYOUT, ...)` で置換
- □ `placement_location` を `derive_placement_location(&LAYOUT, ...)` で置換
- □ `list_placed` を `derive_list_placed(&LAYOUT, ...)` で置換
- □ `can_place` / `base_dir` / `filter_component` プライベート関数を削除
- □ `cargo test target::env::antigravity` で既存テスト全 green

### Gemini CLI

- □ `gemini_cli.rs` に `LAYOUT` 定数（`.gemini` + Skill + Instruction capability）を追加
- □ 同様の置換（`supports_scope` / `placement_location` / `list_placed`）
- □ `can_place` / `base_dir` / `filter_component` 削除
- □ `cargo test target::env::gemini_cli` で既存テスト全 green

## Phase 3: Codex / Copilot 移行

### Codex

- □ `codex.rs` に `LAYOUT` 定数（Skill / Agent / Instruction / Hook capability）を追加
- □ `supports_scope` / `placement_location` / `list_placed` を置換（振る舞いフックはそのまま）
- □ `placed_common::list_instruction` 呼び出しを `derive_list_placed` に統合
- □ `can_place` / `base_dir` / `filter_component` 削除
- □ `cargo test target::env::codex` で既存テスト全 green

### Copilot

- □ `copilot.rs` に `LAYOUT` 定数（Skill/Command/Instruction ProjectOnly + Agent/Hook Both capability）を追加
- □ 同様の置換（`can_place(kind, scope)` の 2 引数 → `ScopeSet::ProjectOnly` に統一）
- □ `can_place` / `base_dir` / `filter_component` 削除
- □ `cargo test target::env::copilot` で既存テスト全 green（Skill Personal が `None` であることを確認）

## Phase 4: Cursor 移行

- □ `cursor.rs` に `LAYOUT` 定数（Skill OriginalNameRequired / Agent+Command PlainMarkdownFile / Instruction ProjectOnly / Hook FixedAtBase）を追加
- □ `supports_scope` / `placement_location` / `list_placed` を置換
- □ `placed_common::list_instruction` 呼び出しを `derive_list_placed` に統合
- □ `can_place` / `base_dir` / `filter_component` / `is_plain_markdown` 削除
- □ 振る舞いフック（`pre_place_check` / `post_place` / `component_conflict_error` / `legacy_cleanup_operations`）はそのまま維持
- □ `cargo test target::env::cursor` で既存テスト全 green（`original_name=None` の Skill が `None` を返すことを確認）

## Phase 5: trait デフォルト接続・ダミー廃止

- □ `Target::supports_scope` のダミー `PlacementContext("test")` プロービングを `derive_supports_scope` へ置換（各 env の override がある間は段階的）
- □ `placed_common::list_instruction` の `dummy_origin("test")` 廃止（derive_list_placed が InstructionExists を直接計算するため不要になる）
- □ `placed_common.rs` を縮小または削除
- □ `src/sync_test.rs` / `src/sync/endpoint/endpoint_test.rs` の `FakeTarget` が影響を受けないか確認
- □ `cargo test` で全テスト green

## Phase 6: ドキュメント同期・掃除

- □ `docs/architecture/core-design.md` または `docs/concepts/targets.md` に `TargetLayout` モデル概要を追記
- □ `docs/target-layout-refactor/` の 3 ファイルを `docs/old/` へアーカイブ（正式採用済み）
- □ 死コード（使われなくなった `const` / import 等）を削除

## Verification

- □ `cargo test` で全テスト（既存 1888 行 + 新規テーブル駆動テスト）が green
- □ `cargo build` でコンパイルエラーなし
- □ `cargo clippy` で新規 warning なし
- □ `cargo fmt` でフォーマット適用済み
- □ `supports_scope` のダミープロービング（`"test"` origin）が全コードから消えていることを Grep で確認
- □ `can_place` / `filter_component` プライベート関数が各 env から消えていることを確認
- □ `placed_common::list_instruction` の利用箇所がゼロになっていることを確認（または削除済み）
- □ Definition of Done チェックリストを全項目確認
