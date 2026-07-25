# Requirements: 配置ディレクトリ名・ファイル名リテラル集約 (Issue #339)

> exploration-report と hearing-notes を統合し、要件・制約を確定する中間ドキュメント。
> Issue 原文の提案を、#338 完了後のコードベースに合わせて具体化している。

## ユースケース

### UC-1: メンテナが配置サブディレクトリ名を一系統で直す

- **アクター**: PLM メンテナ
- **状況/前提**: `"skills"` / `"agents"` / `"hooks"` 等が `placement_helpers`・各 env・`cleanup.rs`・`scan/constants` に分散している
- **達成したいこと**: デフォルト配置名の変更が単一の定義（または明示的な Target 上書き 1 箇所）で済む
- **成功条件**: デフォルト subdir 文字列の定義箇所が 1 系統になり、ベアリテラルの再定義が本番コードから消える（テスト内フィクスチャは除外可）

### UC-2: メンテナが Copilot の `"prompts"` を表示用 `"commands"` と混同しない

- **アクター**: PLM メンテナ
- **状況/前提**: `ComponentKind::plural()` は `"commands"`、Copilot 実配置は `"prompts"`
- **達成したいこと**: 表示用 API と配置用 API が分離され、誤用がコンパイルまたはレビューで防げる
- **成功条件**: `plural()` の doc に「表示専用」と明記。配置パス組み立ては `plural()` を直接使わず、Target/ヘルパ経由の配置 API を使う

### UC-3: メンテナが instruction ファイル名の二重定義を解消する

- **アクター**: PLM メンテナ
- **状況/前提**: `INSTRUCTION_FILE_NAMES`（scan）と各 env の `LAYOUT.instruction_file` が手同期
- **達成したいこと**: 除外集合が Target 側の定義から構築される
- **成功条件**: 新ターゲットの instruction を LAYOUT/公開 API に追加するだけで、scan の除外集合にも自動反映される（手編集の第 2 箇所が無い）

### UC-4: メンテナが cleanup の環境パス漏れを防ぐ

- **アクター**: PLM メンテナ
- **状況/前提**: `cleanup_specs` が `".codex"` 等をベア再定義しており、LAYOUT と乖離しうる
- **達成したいこと**: cleanup が Target/LAYOUT 由来のルート・subdir を消費する
- **成功条件**: `cleanup.rs` 本番コードに環境ルート / kind subdir のベアリテラルが残らない（または単一テーブルからの参照のみ）

### UC-5: テストが移行中の振る舞い不変を保証する

- **アクター**: PLM メンテナ
- **達成したいこと**: 既存テスト期待値を変えずに移行できる
- **成功条件**: 各 Phase 完了後 `cargo test` 全 green。追加は不変条件テストのみ

---

## 要件・制約

### 機能要件

- **FR-001**: `ComponentKind::plural()` を表示・シリアライズキー専用と文書化する。配置パス組み立てから直接呼ばない
- **FR-002**: ターゲット非依存のマニフェスト名 / サフィックス（`SKILL.md`, `.agent.md`, `.prompt.md`）を `ComponentKind` メソッドまたは共有 const に集約し、`scan/constants.rs` はそれへの re-export または削除とする
- **FR-003**: `placement_helpers`（`skill_dir` / `agent_file`）と `filter_skill_dir` / `component/deployment.rs` が FR-002 の定数を消費する
- **FR-004**: ターゲット依存の配置 subdir（特に Copilot Command → `"prompts"`、Cursor Command → `"commands"`）は Target/LAYOUT 側の単一定義から供給する
- **FR-005**: Instruction ファイル名を Target 側から取得できる API（trait メソッドまたは `pub(crate)` テーブル）を用意し、`scan/placement.rs` の `INSTRUCTION_FILE_NAMES` をそれから構築する
- **FR-006**: 環境ルート（personal / project subdir）を Target/LAYOUT から取得できる形にし、`plugin/cache/cleanup.rs` の `cleanup_specs` がそれを消費する
- **FR-007**: `commands/info/wire.rs` / `commands/list/wire.rs` の kind キー配列、および `commands/deploy/import.rs` の kind 文字列マッチを `ComponentKind::plural()` / `as_str` 系に寄せる（出力値は不変）
- **FR-008**: （任意）`Scope::description()` を現行サポートターゲットを含む説明に更新する

### 非機能要件

- **NFR-001**: 振る舞い不変（配置パス・list 結果・cleanup 対象・CLI/JSON 出力を変えない）
- **NFR-002**: ビッグバン禁止（Phase A〜F。各 Phase 独立コミット）
- **NFR-003**: 既存 `*_test.rs` の期待値変更なし（テスト追加のみ可）
- **NFR-004**: ユーザー向け CLI フラグ・サブコマンド変更なし

### 制約・設計方針

- **CON-001**: Rust 2021、`mod.rs` 禁止
- **CON-002**: 新規クレート追加なし
- **CON-003**: #338 の `placed/` ヘルパ・`LAYOUT` / `CAPABILITIES` / `can_place_scope` を破壊せず、その上に定数層を載せる
- **CON-004**: 表示用と配置用の概念を混同する API（例: 「常に `plural()` を join する」）を導入しない
- **CON-005**: プラグインパッケージ内のデフォルト相対パス（`DEFAULT_SKILLS_DIR` 等）はターゲット配置パスと概念は別だが、**文字列ソースは共有**してよい
- **CON-006**: テストは TDD。本体と同ディレクトリの `*_test.rs`
- **CON-007**: FakeTarget / sync テストへの影響を最小化（公開 trait シグネチャ変更は必要最小限）
- **CON-008**: 移行順序の目安 — 定数定義（B）→ ヘルパ/scan（C）→ Target 公開 + cleanup（D）→ wire（E）→ docs（F）

---

## 未解決の確認事項

hearing-notes の「ユーザー確認が必要な点」参照。計画レビューで未回答でも実装 Phase A は進行可。API 公開範囲（trait vs `pub(crate)`）は Phase D 着手前に確定すること。
