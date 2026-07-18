---
name: test-review
description: テストコード品質レビュースキル。テストコード単体を素早くレビューしたい場合に使用する。古典学派（Classical School）のテスト原則に基づき、test-quality-reviewerエージェントを起動してモック制限・振る舞いテスト・テスト価値・テストケース網羅性の4次元でレビューする。specフォルダの有無に関わらず動作し、存在する場合は計画ドキュメントも参考にする。実装全体を多観点で統合レビューする場合は spec-based-code-review スキルを使用すること。「テストレビュー」「test review」「テスト品質チェック」「モック使いすぎ」「テストの書き方チェック」「古典学派」「classical school」「テストコードレビュー」「test-review」「テストケース不足」などでトリガー。
disable-model-invocation: true
argument-hint: "[ファイルパス or spec番号]"
---

# テストコード品質レビュー

テストコードを古典学派のテスト原則に基づいてレビューするスキル。
specフォルダの有無に関わらず動作する。specがあれば参考にする。

test-quality-reviewer エージェントを起動してレビューを委譲する。

## ワークフロー

```
Step 0: モード判定（spec有無）
  ↓
Step 1: テストファイルの収集
  ↓
Step 2: サブエージェントを起動
  ↓
Step 3: 結果の提示とアクション提案
```

## Step 0: モード判定

### 引数がspec番号の場合

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/$0-* 2>/dev/null | head -1)
```

マッチすればspecモード。

### 引数がファイルパス/ディレクトリの場合

直接そのパスをレビュー対象とする。specなしモード。

### 引数なしの場合

1. `.plugin-workspace/.specs/` にarchive外のspecフォルダがある → 最大番号を自動選択してspecモード
2. 該当なし → specなしモード（git diff のテストファイルを対象）

## Step 1: テストファイルの収集

### specモードの場合

AskUserQuestion:

```yaml
question: "レビュー対象を選択してください"
header: "レビュー範囲"
options:
  - label: "全テストコード（推奨）"
    description: "implementation-planのテストファイルとgit diffのテストファイル"
  - label: "git diffのテストファイルのみ"
    description: "直近の変更に含まれるテストファイルのみ"
  - label: "指定パス"
    description: "ファイルパスまたはディレクトリを指定"
```

### specなしモード（引数なし）

AskUserQuestion:

```yaml
question: "レビュー対象を選択してください"
header: "レビュー範囲"
options:
  - label: "git diffのテストファイル（推奨）"
    description: "直近の変更に含まれるテストファイルをレビュー"
  - label: "指定パス"
    description: "ファイルパスまたはディレクトリを指定"
  - label: "全テストファイル"
    description: "プロジェクト全体のテストファイルをスキャン"
```

### specなしモード（引数あり）

引数のパスを直接使用。

### テストファイルの検出パターン

`*.test.*`, `*.spec.*`, `test_*.*`, `*_test.*`, `__tests__/**`

## Step 2: サブエージェントを起動

### 出力先

- specモード: `{spec_dir}/spec-based-code-review/test-quality-{NNN}.md`
- specなしモード: ファイル出力なし（エージェントの出力を直接提示）

### エージェント起動

**パス解決**: 以下のプロンプト内の `{spec-based-code-review-plugin-path}` は、このプラグインのルートディレクトリ（本スキルの SKILL.md があるディレクトリから2階層上、`.../plugins/spec-based-code-review-plugin`）を指す。実際の絶対パスに置き換えてサブエージェントに渡すこと。

```
Task tool:

1. test-quality-reviewer:
   description: "test-quality-reviewer: テスト品質レビュー"
   run_in_background: true
   prompt: |
     あなたは test-quality-reviewer エージェントです。
     古典学派のテスト原則（普遍的ルール）に基づいてテストコードをレビューしてください。
     指摘の根拠はルール自体です。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/test-review-rules.md

     ## 計画ドキュメント（specモード時のみ — 参考情報）
     {specモード時: implementation-plan.md, exploration-report.md のパス}
     {specなしモード時: 「計画ドキュメントなし。CRITICAL/WARNING/INFO で分類。」}

     ## テストコード情報
     テストファイル一覧:
     {テストファイルリスト}

     git diff（テスト部分）:
     {git diff の内容（テストファイル部分）}

     ## 出力先
     {specモード時: ファイルパス}
     {specなしモード時: 「ファイル出力不要。結果をテキストで返してください。」}

     ## 重要
     - テスト対象の実装コードも必ず読むこと（次元12・13に必須）
     - 指摘の根拠はルール（普遍的原則）— spec 不要で指摘を出す
     - 外部依存への spy アサーションは CRITICAL にしないこと
```

### 完了待ち

```
TaskOutput:
  task_id: "{エージェントのtask_id}"
  block: true
  timeout: 300000
```

## Step 3: 結果の提示とアクション提案

CRITICAL または WARNING がある場合:

```yaml
question: "レビュー結果への対応を選択してください"
header: "対応方針"
options:
  - label: "指摘を修正する"
    description: "CRITICAL/WARNINGの指摘に対応します"
  - label: "レビュー結果を確認のみ"
    description: "後で対応します"
  - label: "再レビュー"
    description: "修正後にもう一度レビューを実行します"
```

再レビューは最大5回まで。

## 重要な制約

- **コードの変更はオーケストレーター自身では行わない**
- ルール（普遍的原則）が指摘の根拠。specはあくまで参考
- 実装コードを読んでロジックの有無を確認する
