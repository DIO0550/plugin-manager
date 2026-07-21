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

---

## 方針転換（ユーザーフィードバック 2026-07-21）

### フィードバック要旨

「基本的に impl をベースにして欲しいです」

### 方針転換の内容

初回計画は `TargetLayout` / `PlacementRule` / `DiscoverRule` など理想的な宣言的 DSL を先に設計する **top-down** アプローチだった。ユーザーフィードバックにより、**既存の `impl Target for XxxTarget` 実装を起点に bottom-up で共通骨格を抽出する**アプローチに全面改訂する。

### 新しい設計原則

1. **実装が真実源**: 抽象は「5 つの env 実装に既にあるコード」から帰納する。まだコードにない概念（過剰な enum 階層）は作らない
2. **抽出ファースト**: Phase の最初は「同一骨格のコピペを共通ヘルパへ切り出す」。DSL 完成を Phase の前提にしない
3. **ケイパビリティ表は薄い符号化**: `can_place` / `supported_components` / `placement_location` match / `filter_component` match の現状の分岐をそのまま表にしたデータにする。型は既存 match アームに 1:1 対応する最小セットに縮小
4. **trait デフォルトは既存シグネチャを維持**: `placement_location` / `list_placed` / `supported_components` の外向き API は変えない。内部実装を共通化する
5. **Cursor / Copilot / Codex の例外は当面各 impl に残す**: OriginalNameRequired・PlainMarkdown・分散 Hook・振る舞いフックは共通骨格抽出後に必要なものだけデータ化する（最初から全部 DSL 化しない）
6. **先行設計ドキュメントは「参考」に格下げ**: `docs/target-layout-refactor/capability-model-spec.md` の FR は「参考指針」。impl 抽出ベースの要件が優先。CON-009（BL-001〜BL-008 正式採用）は「最終形の指針」に弱める

### 推奨する計画骨格

- **Phase A**: 現状 5 impl の差分表を確定（exploration-report の情報を正とする）
- **Phase B**: `list_placed` 共通骨格を自由関数/ヘルパに抽出（Antigravity/Gemini から適用）
- **Phase C**: `filter_component` の共通アーム（特に Skill）を共有
- **Phase D**: `supports_scope` ダミープロービング廃止（`can_place` 系を単一化）
- **Phase E**: `placement_location` 共通パターン（Skill dir / Agent file）をヘルパ化
- **Phase F**: 残った「純粋データ差分」だけを薄い `TargetLayout` const にまとめる（最初から完全 DSL を目指さない）
- **Phase G**: docs / 不変条件テスト / 死コード削除
