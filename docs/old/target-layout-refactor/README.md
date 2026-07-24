# Target Layout 集約（#338）— impl ベース計画（アーカイブ）

> **ステータス**: 実装済み（本ディレクトリは計画フェーズのアーカイブ）  
> **方針**: 理想 DSL 先行ではなく、既存 `impl Target` から共通骨格を bottom-up 抽出する

## ファイル

| ファイル | 説明 |
|----------|------|
| [hearing-notes.md](./hearing-notes.md) | ヒアリング + 方針転換メモ |
| [exploration-report.md](./exploration-report.md) | 探索 + 抽出候補（行番号付き） |
| [requirements.md](./requirements.md) | 要件（impl 抽出ベース） |
| [implementation-plan.md](./implementation-plan.md) | 実装計画 Phase A〜G |
| [tasks.md](./tasks.md) | タスクリスト |
| [test-cases.html](./test-cases.html) | テストケース詳細 |

## Phase 概要

| Phase | 内容 |
|-------|------|
| A | 5 impl 差分確定 |
| B | `list_placed` 骨格抽出 |
| C | Skill filter 共通化 |
| D | `supports_scope` ダミー廃止（`can_place_scope`） |
| E | `placement_location` 共通パターン抽出 |
| F | 薄い定数化（`LAYOUT` / `CAPABILITIES`） |
| G | docs / 掃除 |
