# Requirements: target/env 宣言的ケイパビリティ集約 (Issue #338)

> exploration-report と hearing-notes を統合し、「誰が・何のために・どう使うか」と要件・制約を確定する中間ドキュメント。

## ユースケース

### UC-1: メンテナが新ターゲットをデータ宣言で追加する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: Claude Code ターゲット（#96）など新環境の追加が予定されており、現在は `env/` に同型コピペファイルを作成する必要がある
- **達成したいこと**: `TargetLayout` 定数を 1 つ宣言し、振る舞いフックが不要なら `impl Target` のフックメソッド override なしで新ターゲットを追加できる
- **成功条件**: `src/target/env/new_target.rs` に `LAYOUT` 定数のみ書けば、`placement_location` / `list_placed` / `supports` / `supports_scope` が動作する

### UC-2: メンテナがサポートマトリクスを単一表で確認・修正する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: 現状 `supported_components` スライス・`can_place` 関数・`placement_location` match の 3 箇所が種別×スコープのサポート可否を表現しており、乖離が検出できない
- **達成したいこと**: kind × scope × 配置規則を `ComponentCapability` の静的配列として 1 箇所で宣言し、3 箇所を同期する必要をなくす
- **成功条件**: `TargetLayout.components` を書き換えれば、`supported_components` / `supports_scope` / `placement_location` / `list_placed` がすべて自動的に一致する

### UC-3: レビュアーが PR の差分を「データ変更」として読む

- **アクター**: PLM コードレビュアー（開発者）
- **状況/前提**: 現状の `placement_location` match と `list_placed` 骨格は制御フローとして記述されており、差分がコードの意図を反映しにくい
- **達成したいこと**: ターゲット間の差分が `PlacementRule` / `DiscoverRule` / `ScopeSet` の宣言値の差として PR diff に現れる
- **成功条件**: `antigravity.rs` の変更が `LAYOUT` 定数の値変更のみで表現され、制御フロー差分がない

### UC-4: テストが振る舞い不変を保証する

- **アクター**: PLM メンテナ（開発者）
- **状況/前提**: 移行中に既存の `*_test.rs`（合計 1888 行）がリグレッションを検出できなければならない
- **達成したいこと**: 既存テストの期待値（パス・サポート可否・list 結果）を変更せずに全 Phase の移行を進められる
- **成功条件**: `cargo test` が全 Phase で green を維持する

---

## 要件・制約

### 機能要件

- **FR-001**: `ComponentCapability { kind, scopes: ScopeSet, placement: PlacementRule, discover: DiscoverRule }` でターゲットの差分を宣言できる
- **FR-002**: `TargetLayout` を持つターゲットは `placement_location` を `TargetLayout` から導出できる。ロジックは共通ヘルパ（または `Target` デフォルト実装）1 系統のみ
- **FR-003**: `supported_components()` は `TargetLayout.components` から「いずれかの scope で配置可能」な kind の集合として導出する
- **FR-004**: `supports_scope(kind, scope)` は `ScopeSet::contains(scope)` で判定する（ダミー `PlacementContext` プロービング廃止）
- **FR-005**: `list_placed` は `DiscoverRule` に従ってスキャン結果をフィルタリングする。`InstructionExists` は `placement_location` 計算後のパス存在確認で、`Target` trait を呼ばず直接計算
- **FR-006**: Cursor `Skill` の `OriginalNameRequired` ポリシーは `NamingPolicy::OriginalNameRequired` として明示し、`original_name` が `None` / 空文字の場合 `placement_location` が `None` を返す
- **FR-007**: Copilot の scope 制約（Skill/Command/Instruction は Project のみ）は `ScopeSet::ProjectOnly` として宣言する
- **FR-008**: Instruction 配置パスは `origin` / `component name` に非依存であることを `PlacementRule::InstructionFile` 型で保証する
- **FR-009**: 新規モジュール `src/target/layout.rs` + `src/target/layout/`（Feature ベース、`mod.rs` 禁止）を追加する

### 非機能要件

- **NFR-001**: 振る舞い不変（パス・サポート可否・`list_placed` 結果を変えない）
- **NFR-002**: 静的テーブル（`const` / `&'static [...]`）により実行時オーバーヘッドほぼゼロ
- **NFR-003**: ビッグバン禁止（Phase 0〜6 の段階移行。各 Phase が独立 PR）
- **NFR-004**: 既存 `*_test.rs` の期待値変更なし（テスト追加のみ可）

### 制約・設計方針

- **CON-001**: Rust 2021 edition、`mod.rs` 禁止（Rust 2018+ スタイル）
- **CON-002**: 新規クレート追加なし。既存 `scan_components` / `paths::base_dir` 再利用
- **CON-003**: PR #388 の振る舞いフック（`pre_place_check` / `post_place` / `component_conflict_error` / `legacy_cleanup_operations`）は本作業対象外。override のまま維持
- **CON-004**: `Target` trait に `fn layout() -> &'static TargetLayout` を追加する場合、既存の `FakeTarget`（sync テスト等）に影響しないよう trait デフォルト実装を提供する
- **CON-005**: #339（文字列定数一元化）は並行可。`PlacementRule.subdir` 等の文字列リテラルは後で #339 の定数に差し替え可とする
- **CON-006**: 移行順序は Antigravity → Gemini CLI → Codex → Copilot → Cursor（シンプルなものから）
- **CON-007**: テストファイルは本体と同ディレクトリの `*_test.rs` 形式（`#[path = "xxx_test.rs"]`）
- **CON-008**: テストは TDD（Red → Green → Refactor）で進める。Phase 1 では Red（失敗するテスト）から開始
- **CON-009**: `docs/target-layout-refactor/capability-model-spec.md` の BL-001〜BL-008 を正式設計として採用する

---

## 未解決の確認事項

なし

> hearing-notes と exploration-report の統合により、Issue #338 の要求は全て確定済み。  
> `docs/target-layout-refactor/` の先行設計（BL-001〜BL-008）が exploration-report で完全整合確認済みのため、コードベースを調べても解決できない未確定事項はゼロ。
