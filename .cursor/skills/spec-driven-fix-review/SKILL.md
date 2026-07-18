---
name: spec-driven-fix-review
description: "spec-viewerで付けたレビューコメント（.comments/impl.json）を読み取り、implementation-plan.mdとtasks.mdに修正を反映するスキル。spec-driven-devの計画レビュー後に使用する。「レビュー反映」「コメント反映」「fix review」「レビュー修正」「コメントを適用」「spec-viewerの指摘を反映」「レビュー結果を反映して」などで積極的に使用すること。"
---

# Spec-Driven Fix Review

spec-viewerの未解決レビューコメントを implementation-plan.md と tasks.md に反映する。

---

## ワークフロー

### Step 1: コメントと計画を読み込む

現在作業中のspecフォルダ（会話コンテキストから既知）を `$SPEC_DIR` とする。
不明な場合は引数で指定された番号のspecフォルダを、それもなければ `.plugin-workspace/.specs/` 直下（`archive/` を除く）の最大番号フォルダを対象とする。

`$SPEC_DIR/.comments/impl.json` と `$SPEC_DIR/implementation-plan.md` を Read で読み込む。

impl.json が存在しない、または `resolved: false` のコメントが0件なら、その旨を通知して終了。

### Step 1.5: ユーザー直接指示への対応

impl.json が存在しない場合でも、ユーザーが直接修正を指示する場合がある。
ユーザーの指示が曖昧な場合は AskUserQuestion で以下を確認する:

- 対象特定:「implementation-plan のどの部分を修正しますか？」
  選択肢: 設計方針 / 変更案（コード） / システム図 / タスク構成
- 修正方向:「どのように修正しますか？」
  選択肢: もっと詳細に / 別のアプローチ / 具体例追加 / 削除・簡略化

明確な指示の場合はそのまま Edit で修正に進む。

### Step 2: コメントを修正指示として適用

未解決コメントを `anchor.blockIndex` の昇順で処理する。コメントごとに:

0. **曖昧判定**: `body` が以下に該当する場合、AskUserQuestion で具体化する
   - 修正内容が不明確（「要修正」「改善して」等、具体的な変更指示がない）
   - 対象箇所と `body` の指示が矛盾している
   - 質問:「コメント "{body の冒頭20文字}..." の修正方針は？」
   - 選択肢: コメント内容から推測できる修正案2-3個 + 「該当箇所を削除する」
1. `anchor.textSnippet` で implementation-plan.md 内の対象箇所を特定する。見つからなければ `anchor.blockType` と `anchor.blockIndex` で推定する
2. `body`（または AskUserQuestion で具体化された指示）に従って Edit で修正する
3. 修正がタスク構成に影響する場合は `$SPEC_DIR/tasks.md` も更新する

**システム図の保護**: ASCII罫線図のシステム図は spec-driven-dev が必須としている成果物のため、削除せず図自体を更新する。

### Step 3: 修正結果を報告

コメントごとに1行で報告する:

```
- [cmt_xxx...xxx] 「{textSnippet}」 → {修正内容の要約}
```

spec-viewerでの確認を案内する。
