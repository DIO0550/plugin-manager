# Hearing Notes: target/env 宣言的ケイパビリティ集約 (Issue #338)

## 目的

`target/env/` の各実装（Antigravity / Gemini CLI / Codex / Copilot / Cursor）で発生している `list_placed` / `filter_component` / `placement_location` / `base_dir` の制御フローコピペを解消し、`supported_components` と `can_place/placement_location` の二重真実源を排除する。差分を `TargetLayout` 的な宣言的記述子に集約し、新ターゲット追加をデータ宣言中心に変える。

## スコープ

- **種別**: リファクタリング（振る舞い不変）
- **影響範囲**: 既存修正（`src/target/` 中心）。ユーザー向け CLI 仕様変更なし
- **優先度**: 高（Issue #96 Claude Code 追加前に完了させたい）

## 技術的詳細

- **技術スタック**: Rust 2021
- **フレームワーク**: 既存 Target trait + Feature ベースモジュール構成（mod.rs 禁止）
- **依存関係**: 新規クレートなし。既存 `scan_components` / `paths::base_dir` 再利用
- **データ構造**:
  - `TargetLayout` — ターゲットごとの宣言的記述子
  - `ComponentCapability` — kind × scope × `PlacementRule` × `DiscoverRule`
  - `ScopeSet` — PersonalOnly / ProjectOnly / Both
  - `PlacementRule` / `DiscoverRule` — 配置・探索ルールの enum
- **段階移行**: Antigravity → Gemini CLI → Codex → Copilot → Cursor の順。ビッグバン禁止
- **対象外**: PR #388 の配置フック (`pre_place_check` / `post_place` / `component_conflict_error` / `legacy_cleanup_operations`) — 振る舞いフックは override のまま
- **#339 関連**: 文字列定数一元化は並行作業可。レイアウト内リテラルは後で差し替え可

## 品質要件

- **エッジケース**:
  - Cursor `Skill` の `original_name` 欠落時の処理（`OriginalNameRequired` ポリシー）
  - Copilot Personal スコープで `Skill` 非サポート
  - `Instruction` の `list_placed` が `name` に依存しない（固定ファイル）
  - `supports_scope` / `placed_common` のダミー `PlacementContext("test")` プロービング hack 廃止
- **エラーハンドリング**: リファクタリングのため既存エラー処理を維持。振る舞い変更なし
- **テスト要件**:
  - 既存 `*_test.rs` を全て維持（リグレッション防止）
  - `supports` / `supports_scope` / `placement_location` の不変条件テストを追加
  - TDD（Red → Green → Refactor）サイクルで進める
  - テストコードは本体と別ファイル（`foo_test.rs` 形式）
- **パフォーマンス**: 要件なし（静的テーブル、実行時コストほぼゼロ）

## 追加コンテキスト

- Issue #338 は 4 環境と書いているが、Cursor も対象（計 5 環境）
- 先行下書きあり: `docs/target-layout-refactor/`（`capability-model-spec.md`, `migration-plan.md`, `index.md`）— 参考資料として活用する。正式成果物は `.plugin-workspace/.specs/001-target-layout-capability/`
- 関連 Issue: #96 Claude Code 追加（本リファクタが前提）, #321, #336, #339（文字列定数）
- PR #388 の配置フック移送は完了済み — 本作業対象外
- Target trait の `supported_components`、`supports_scope`、`can_place`、`placement_location` がすべてケイパビリティテーブルから導出されることが最終ゴール
