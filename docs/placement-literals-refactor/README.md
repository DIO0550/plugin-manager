# 配置リテラル集約（#339）— 計画

> **ステータス**: 計画フェーズ（未実装）  
> **関連 Issue**: [#339](https://github.com/DIO0550/plugin-manager/issues/339)  
> **前提**: [#338](https://github.com/DIO0550/plugin-manager/issues/338) Target Layout 集約（roadmap ✅ 完了）  
> **方針**: 表示用 `plural()` と配置用パス断片を分離し、文字列の単一真実源を `ComponentKind` / `Target`（LAYOUT）に寄せる

## ファイル

| ファイル | 説明 |
|----------|------|
| [hearing-notes.md](./hearing-notes.md) | スコープ・設計論点・ユーザー確認事項 |
| [exploration-report.md](./exploration-report.md) | 現状の 3+ 系統並立（行番号付き） |
| [requirements.md](./requirements.md) | UC / FR / NFR / CON |
| [implementation-plan.md](./implementation-plan.md) | 実装計画 Phase A〜F |
| [tasks.md](./tasks.md) | TDD タスクリスト |

## Phase 概要

| Phase | 内容 |
|-------|------|
| A | リテラル棚卸し固定・責務境界の確定（コード変更なし） |
| B | ターゲット非依存定数を `ComponentKind`（または共有 const）へ集約 |
| C | ヘルパ / deployment / scan を B の定数消費に切替 |
| D | Target 依存パス（instruction / env root / Command subdir）を公開し `cleanup`・`placement` が消費 |
| E | wire / import の表示・入出力キーを `plural()` 呼び出しへ |
| F | docs / 死コード削除 / 不変条件テスト |

## #338 との関係

```text
#338: 制御フロー・サポート判定の骨格抽出（完了）
#339: 配置文字列の単一真実源（本計画）
```

#338 で各 env に薄い `LAYOUT` / `CAPABILITIES` と `placed/` ヘルパができた。本 Issue はその **中身のリテラル** と、scan / cleanup / wire に残る **二重定義** を解消する。

## 実装開始の前提

- 振る舞い不変（配置パス・スキャン結果・クリーンアップ対象を変えない）
- ビッグバン禁止（Phase 単位で独立コミット）
- Issue 原文の行番号は #338 後にドリフトしている → [exploration-report.md](./exploration-report.md) §9 を正とする
