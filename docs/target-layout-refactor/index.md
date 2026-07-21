# Target Layout 宣言的ケイパビリティ集約

> **バージョン**: 1.0  
> **作成日**: 2026-07-21  
> **ステータス**: 計画（未実装）  
> **関連 Issue**: [#338](https://github.com/DIO0550/plugin-manager/issues/338)

## 概要

`target/env/` 配下のターゲット実装は、環境ごとに本当に異なるのが「ベースパス・サブディレクトリ・ファイルサフィックス・instruction ファイル名・kind × scope の対応」といった**データ**であるにもかかわらず、`list_placed` / `filter_component` / `placement_location` / `base_dir` の制御フローがターゲットごとにコピペされている。

加えて「何をサポートするか」が次の 2 系統で二重管理されており、乖離してもコンパイルが通る:

| 系統 | 用途 | 現状の問題 |
|------|------|-----------|
| `supported_components()` | `supports()` / UI・一覧の静的スライス | スコープ非依存。Copilot Skill が Personal 不可でもスライスには載る |
| `can_place` + `placement_location → None` | 実配置の可否 | 実装ごとに手書き。`supports_scope` はダミー `PlacementContext` でプロービングする hack |

本リファクタの目的は、差分を宣言的な記述子（`TargetLayout`）に落とし、共通制御フローを 1 箇所に集約し、サポート判定の単一真実源を確立することである。

## 背景（Issue 検出時点からの更新）

Issue 本文は 4 環境（codex / copilot / antigravity / gemini_cli）を対象としているが、その後 Cursor が追加され、**同じ骨格の 5 実装目**になっている。Claude Code ターゲット（#96）追加前に集約しないと 6 実装目のコピペが生まれる。

関連して既に進んだ周辺整理:

| 変更 | 本計画との関係 |
|------|----------------|
| PR #388 — 配置前後フックを `Target` trait へ移送 | **対象外を明確化**。`pre_place_check` / `post_place` / `component_conflict_error` / `legacy_cleanup_operations` は振る舞いフックとして残す |
| #336 — 識別子一元化 | 並行可能。`TargetLayout` は `TargetKind` をキーに持てばよい |
| #339 — サブディレクトリ名・instruction 名の一元化 | **連携必須**。文字列定数の単一真実源は #339、kind×scope×配置規則の単一真実源は本 Issue。順序は後述 |

## スコープ

### 対象

- `src/target/env/{codex,copilot,antigravity,gemini_cli,cursor}.rs` の配置レイアウト系ロジック
- `Target::supported_components` / `supports` / `supports_scope` の導出
- `list_placed` / `filter_component` / `placement_location` / `base_dir` の共通化
- `placed_common::list_instruction` と `supports_scope` のダミー `PlacementContext` プロービング廃止
- 既存ユニットテストの振る舞い維持（パス・サポート可否の期待値は変えない）

### 対象外

- Codex / Cursor の Hook 上書きガード・feature flag・legacy Skill 削除（PR #388 のフック群）
- パーサー / フォーマット変換（`parser/`）
- 配置実行本体（`component/deployment`）や install フローの変更
- Claude Code ターゲット本体の実装（#96）— ただし本リファクタ完了後は「レイアウト宣言 1 つ」で追加できる状態をゴールとする
- #339 単独のリテラル掃除（ただし本モデルが定数を参照する設計にする）

## ゴール / 非ゴール

| ID | 内容 |
|----|------|
| G-001 | 新ターゲット追加が「`TargetLayout` 定数を 1 つ書く + 必要なら振る舞いフックを override」になる |
| G-002 | `supports` / `supports_scope` / `placement_location.is_some()` / `list_placed` の空返りが同一ケイパビリティ表から導出され、乖離が構造的に不可能になる |
| G-003 | ダミー origin `"test"` による `PlacementContext` プロービングを廃止する |
| NG-001 | 配置パスやサポートマトリクスの**仕様変更**はしない（リファクタのみ） |
| NG-002 | 全ターゲットを 1 コミットで機械的に書き換える「ビッグバン」は避ける |

## ユーザーストーリー（開発者視点）

| ID | ～として | ～したい | ～のために |
|:---|:---------|:---------|:-----------|
| US-001 | メンテナ | 新ターゲットの配置規則をデータとして宣言したい | 制御フローをコピペせずに済むように |
| US-002 | メンテナ | サポート判定の真実源を 1 つにしたい | `supports` と実配置の乖離バグを防ぎたい |
| US-003 | レビュアー | kind × scope × パス規則を表で読みたい | PR 差分が「データ変更」としてレビューできるように |

## 仕様書一覧

| 仕様書 | 説明 |
|:-------|:-----|
| [capability-model-spec.md](./capability-model-spec.md) | `TargetLayout` / ケイパビリティ表 / 導出ルール / 現状マトリクス |
| [migration-plan.md](./migration-plan.md) | Phase 分割・受け入れ条件・テスト方針・#339 連携 |

## 成功条件（要約）

1. 5 ターゲットの `list_placed` / `placement_location` 骨格が共通ヘルパ（または trait デフォルト）1 系統になる
2. 各 env ファイルに残るのは「レイアウト定数 + 振る舞いフック override」のみ
3. `supports_scope` がダミー context なしでケイパビリティ表を参照する
4. 既存 `*_test.rs` がパスする（期待値変更は回帰でない限り禁止）
5. ドキュメント（`docs/concepts/targets.md` / `docs/architecture/core-design.md`）にモデル概要が反映される
