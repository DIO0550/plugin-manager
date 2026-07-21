# Requirements: target/env impl ベース共通骨格抽出 (Issue #338)

> exploration-report と hearing-notes（方針転換フィードバック含む）を統合し、「誰が・何のために・どう使うか」と要件・制約を確定する中間ドキュメント。
> 初回計画（top-down DSL 設計）から bottom-up impl 抽出方針へ**全面改訂済み**（2026-07-21）。

## ユースケース

### UC-1: メンテナがコピペ骨格を一箇所で直す

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: `list_placed` の共通制御フロー（scope 確認 → base_dir → scan → filter）が 5 環境ファイルにコピペされており、バグ修正を全ファイルに手で反映する必要がある
- **達成したいこと**: 共通骨格を抽出したヘルパ関数 1 か所だけ直せばすべての環境に反映される
- **成功条件**: `list_placed` 骨格を変更する PR の diff が `target/placed/` または `target/layout/` のヘルパのみで完結し、5 env ファイルに同じパターンが出現しない

### UC-2: メンテナがサポート可否の不整合を検出する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: `supported_components` スライスと `can_place()`/`placement_location()` match の分岐が別々に記述されており、乖離が起きていても気付きにくい
- **達成したいこと**: サポート可否の情報源を 1 か所（または 2 か所以内の明示的な依存）に集約し、「一方だけ更新」ミスをコンパイラ/テストで検出できるようにする
- **成功条件**: `supported_components` の返り値変更と、`can_place`/`placement_location` の許可変更が連動する（同じデータ源、または一方を変えれば他方が自動追従する）

### UC-3: メンテナが `supports_scope` のダミープロービングを除去する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: `supports_scope(kind, scope)` の現行実装が `PlacementContext("test")` ダミーを生成して `placement_location` の戻り値で判定する hack になっている
- **達成したいこと**: ダミー生成なしで直接 `can_place` 系のデータを参照して判定する
- **成功条件**: `src/` に `PlacementContext("test")` または `dummy_origin("test")` を使って scope サポートを判定するコードが残らない

### UC-4: テストが移行中の振る舞い不変を保証する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: 移行中に既存の `*_test.rs`（約 1888 行）がリグレッションを検出できなければならない
- **達成したいこと**: 既存テストの期待値（パス・サポート可否・list 結果）を変更せずに全 Phase の移行を進められる
- **成功条件**: `cargo test` が全 Phase 完了後も全 green を維持する

---

## 要件・制約

### 機能要件

- **FR-001**: `list_placed` の共通制御フロー（scope チェック → base_dir 計算 → scan_components → フィルタ）を共通ヘルパ関数として 1 か所に抽出する。各環境は kind→subdir / フィルタ判定の差分だけを渡す
- **FR-002**: `filter_component` の Skill 共通アーム（`SKILL.md` 存在確認）を共有ヘルパまたは共通関数に抽出する。差分アーム（Agent/Command/Instruction など）は各 env に残すか最小の match テーブルに
- **FR-003**: `supports_scope(kind, scope)` のダミー `PlacementContext("test")` プロービングを廃止し、`can_place` 系のデータを直接参照して判定する
- **FR-004**: `supported_components()` と `can_place()`/`placement_location()` の情報源を統一する。実装方法は問わない（`can_place` から導出、または共通テーブルを参照、など）
- **FR-005**: `placement_location` の共通パターン（Skill ディレクトリ、Agent ファイル、Instruction ファイル）を共通ヘルパ化する。例外（Cursor OriginalNameRequired 等）は各 env の override に残してよい
- **FR-006**: 最終的に残った純粋データ差分（各環境の base_dir パス名、kind×scope 可否、subdir 文字列など）を薄い定数（`TargetLayout` 相当）にまとめることを最終 Phase の目標とする（義務ではなく方向性）
- **FR-007**: `placed_common::list_instruction` の `dummy_origin("test")` 生成を廃止し、パスを直接計算する方法に置き換える

### 非機能要件

- **NFR-001**: 振る舞い不変（パス・サポート可否・`list_placed` 結果を変えない）
- **NFR-002**: ビッグバン禁止（Phase A〜G の段階移行。各 Phase が独立コミット/PR）
- **NFR-003**: 既存 `*_test.rs` の期待値変更なし（テスト追加のみ可）
- **NFR-004**: 外向き API の変更なし（`placement_location` / `list_placed` / `supported_components` / `supports_scope` のシグネチャを変えない）

### 制約・設計方針

- **CON-001**: Rust 2021 edition、`mod.rs` 禁止（Rust 2018+ スタイル）
- **CON-002**: 新規クレート追加なし。既存 `scan_components` / `paths::base_dir` 再利用
- **CON-003**: PR #388 の振る舞いフック（`pre_place_check` / `post_place` / `component_conflict_error` / `legacy_cleanup_operations`）は本作業対象外。override のまま維持
- **CON-004**: Cursor / Copilot / Codex の例外（OriginalNameRequired・PlainMarkdown フィルタ・分散 Hook・振る舞いフック）は当面各 impl に残す。共通骨格抽出後に必要なものだけデータ化する
- **CON-005**: `Target` trait の外向き API シグネチャを変えない。共通化は内部実装（private 関数/ヘルパ）で行う
- **CON-006**: 移行順序は Antigravity → Gemini CLI → Codex → Copilot → Cursor（シンプルなものから）
- **CON-007**: テストファイルは本体と同ディレクトリの `*_test.rs` 形式（`#[path = "xxx_test.rs"]`）
- **CON-008**: テストは TDD（Red → Green → Refactor）で進める
- **CON-009**: `docs/target-layout-refactor/capability-model-spec.md` の BL-001〜BL-008 は「最終形の指針」として参照可。ただし最初のフェーズで全部 DSL 化することは要件ではない
- **CON-010**: `FakeTarget`（sync テスト等）に影響しないよう、共通化は public API の変更ではなく private ヘルパで実施する

---

## 未解決の確認事項

なし

> hearing-notes の方針転換フィードバック（2026-07-21）と exploration-report の統合により、impl 抽出ベースの要件が確定済み。未確定事項ゼロ。
