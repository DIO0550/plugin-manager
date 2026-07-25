# Hearing Notes: 配置ディレクトリ名・ファイル名リテラル集約 (Issue #339)

## 目的

コンポーネント配置のディレクトリ名・ファイル名という中核ドメイン知識が、(a) `ComponentKind::plural()`、(b) `scan/constants.rs`、(c) ヘルパ / env / cleanup / wire のベアリテラル、で並立している状態を解消する。ターゲット追加やパス規約変更を **1 系統の修正** にし、「掃除されないゴミ」「スキャン漏れ」系バグを構造的に防ぐ。

## スコープ

- **種別**: リファクタリング（振る舞い不変）
- **影響範囲**: `src/component/`、`src/scan/`、`src/target/placed/`、`src/target/env/`、`src/plugin/cache/cleanup.rs`、`src/commands/{info,list,deploy}/`
- **優先度**: 中〜高（#96 Claude Code 追加前に文字列二重定義を減らしたい。#338 hearing でも「並行可」と明記）
- **対象外**:
  - #338 で完了した制御フロー骨格の再設計
  - 配置パス規約そのものの変更（例: Copilot を `commands/` に移す等）
  - プラグインマニフェストのスキーマ変更

## Issue 提案の要約（原文）

1. `ComponentKind` に `placement_subdir(target?)` / `file_suffix()` / `skill_manifest()` を持たせ、`scan/constants.rs` は削除または re-export。`plural()` は表示専用と明記。
2. 各 `Target` に `instruction_filename()` を持たせ、`scan/placement.rs` の除外集合は `all_targets()` から構築。
3. 環境ディレクトリ名は `Target`（または `TargetKind` キーの const テーブル）から一元提供し、`cleanup.rs` はそれを消費する。

## #338 完了後の前提（原文との差分）

| Issue 作成時（2026-07-04） | 現行（#338 後） |
|---------------------------|-----------------|
| env 内に `CODEX_SUBDIR` 等の const | **廃止** → 各 env の private `LAYOUT` |
| `SKILL.md` が各 env に散在 | 多くは `filter_skill_dir`（`filter.rs`）へ集約。ただしヘルパ内リテラルは残存 |
| 制御フローが 5 重コピペ | `placed/` ヘルパ + `LAYOUT` / `CAPABILITIES` |
| `plural()` vs Copilot `"prompts"` | **未解決**（本 Issue の核心） |

原文の提案は方向として妥当。実装は **#338 の LAYOUT / ヘルパを消費側に寄せる** 形が、新規 DSL を増やさず安全。

## 推奨する責務境界（レビュー結論）

Issue 提案の `placement_subdir(target?)` はシグネチャが曖昧になりやすい。次の分割を推奨する。

### A. ターゲット非依存（`ComponentKind` または共有 const）

| 概念 | 例 | 置き場所 |
|------|-----|----------|
| 表示用複数形 | `"skills"`, `"commands"` | 既存 `plural()`（表示専用と doc 明記） |
| スキルマニフェスト | `"SKILL.md"` | `ComponentKind::skill_manifest()` または共有 const |
| ファイルサフィックス | `".agent.md"`, `".prompt.md"` | `ComponentKind::file_suffix()`（該当 kind のみ `Some`） |
| プラグイン内デフォルト subdir | `"skills"` 等 | `plural()` と同一ソース（`scan/constants` の `DEFAULT_*` は re-export） |

### B. ターゲット依存（`Target` / `LAYOUT`）

| 概念 | 例 | 置き場所 |
|------|-----|----------|
| 配置サブディレクトリ | Copilot Command → `"prompts"`、Cursor Command → `"commands"` | `LAYOUT` または `Target::component_subdir(kind)` |
| Instruction ファイル名 | `"AGENTS.md"` / `"copilot-instructions.md"` / `"GEMINI.md"` | `LAYOUT.instruction_file` → trait 公開 |
| 環境ルート | `".codex"`, `".github"`, `".cursor"` | `LAYOUT` → cleanup が消費できる公開 API |

**ポイント**: `placement_subdir` を `ComponentKind` に載せるより、**デフォルトは `plural()`、例外だけ Target が上書き** する方が Issue の意図（表示と配置の分離）に沿い、かつ #338 LAYOUT と整合する。

## 品質要件

- **振る舞い不変**: 配置パス、`list_placed` 結果、cleanup 対象ディレクトリを変えない
- **テスト**: 既存 `*_test.rs` の期待値変更なし。不変条件テスト（「ヘルパが ComponentKind 定数を使う」「INSTRUCTION_FILE_NAMES ⊆ all_targets の instruction」等）を追加
- **TDD**: Red → Green → Refactor
- **テスト配置**: `foo_test.rs` 分離（リポジトリ規約）
- **外向き CLI**: 変更なし。JSON キーは既に `plural()` 相当なので、ベア配列を `plural()` 呼び出しに置換するだけ（出力同一）

## ユーザー確認が必要な点

実装着手前に次を確定したい（計画レビュー向け）。

1. **API 公開範囲**: `instruction_filename()` / env root を `Target` trait の公開メソッドにするか、`pub(crate)` の `TargetKind` テーブルに留めるか  
   - 推奨: まず `pub(crate)` テーブル or LAYOUT 公開ヘルパ。trait 拡張は消費側が dyn Target 経由で必要な場合のみ。
2. **`placement_subdir` の置き場**: Issue 原文どおり `ComponentKind` + target 引数か、本メモの「デフォルト plural + Target 上書き」か  
   - 推奨: 後者（#338 LAYOUT と整合）
3. **`Scope::description()`**: 全ターゲットを列挙する静的文字列に更新するか、動的生成に変えるか、現状の要約のまま残すか  
   - 推奨: Phase E で現行ターゲットを含む静的文言に更新（動的化は過剰）
4. **`scan/constants.rs`**: 削除して `ComponentKind` 経由に統一か、薄い re-export に残すか  
   - 推奨: Phase B で re-export 化し、消費者移行完了後に Phase F で削除可否を判断

## 追加コンテキスト

- 関連: #338（骨格完了）、#96（Claude Code — 新ターゲット追加時に本 Issue の効果が効く）
- 探索の正本: [exploration-report.md](./exploration-report.md)
- Copilot Command の `"prompts"` は仕様上の差分であり、バグではない。集約後も **表示 `"commands"` ≠ 配置 `"prompts"`** を型/API で表現し続けること
