---
name: spec-setup
description: spec-pluginのワークスペース初期化と設定を行う。.plugin-workspace/.specs/ディレクトリの作成、デフォルトのレビューツール設定、ファイルごとの出力形式設定(.config.yml)を対話的にセットアップする。「spec setup」「spec初期化」「specセットアップ」「レビューツール設定」「デフォルトレビュー変更」「spec config」「出力形式変更」などでトリガー。設定ファイルにより各スキル実行時のAskUserQuestion問い合わせを省略できる。
allowed-tools: Bash(ls *), Bash(mkdir *), Write
---

# Spec Setup

spec-plugin のワークスペース初期化と設定を行うスキル。

- **初回**: `.plugin-workspace/.specs/` ディレクトリ + `.config.yml` を作成
- **2回目以降**: 既存の設定を表示し、変更を受け付ける

## 設定ファイル

```
.plugin-workspace/.specs/.config.yml
```

| キー | 値 | 説明 |
|------|-----|------|
| `review-tool` | `none` / `codex` / `copilot` / `claude-code` | デフォルトのレビューツール |
| `output-formats.hearing-notes` | `md` / `html` | hearing-notes の出力形式 |
| `output-formats.exploration-report` | `md` / `html` | exploration-report の出力形式 |
| `output-formats.implementation-plan` | `md` / `html` | implementation-plan の出力形式 |
| `output-formats.tasks` | `md` / `html` | tasks の出力形式 |
| `output-formats.tech-reference` | `md` / `html` | tech-reference の出力形式 |
| `skip-files` | リスト | 生成をスキップするファイル（例: `[tech-reference, test-cases, understanding-quiz]`）。`test-cases` は spec-driven-dev で有効。`understanding-quiz` は spec-driven-dev（実装前クイズ）と spec-implement（実装後クイズ）で有効 |
| `quiz-output.plan` | `file` / `artifact` / `interactive` | 実装前クイズ（understanding-quiz-plan）の出力先。`file` = specフォルダにHTMLファイル、`artifact` = Artifact ツールで公開、`interactive` = HTMLを生成せずセッション内で AskUserQuestion により1問ずつ対話出題。**未設定時のデフォルト: `file`** |
| `quiz-output.impl` | `file` / `artifact` / `interactive` | 実装後クイズ（understanding-quiz-impl）の出力先。値の意味は plan と同じ。**未設定時のデフォルト: `artifact`** |

## ワークフロー

### Step 1: ワークスペースの確認・作成

`.plugin-workspace/.specs/` が存在するか確認する。

- **存在しない場合**: `mkdir -p .plugin-workspace/.specs/.guard` で作成し、ユーザーに報告
- **存在する場合**: 既存のspecフォルダ数を表示

### Step 2: レビューツールの選択

`.plugin-workspace/.specs/.config.yml` が存在する場合は現在の設定を表示する。
設定の有無に関わらず、常に AskUserQuestion でレビューツールを選択:

- **レビューなし（最速）** → `review-tool: none`
- **Codex CLI** → `review-tool: codex`
- **GitHub Copilot CLI** → `review-tool: copilot`
- **Claude Code CLI** → `review-tool: claude-code`

### Step 3: 出力形式の選択

AskUserQuestion（multiSelect）で、**HTMLで出力するファイル**を選択させる。
選択されなかったファイルは Markdown（デフォルト）で出力される。

既存の `.config.yml` に `output-formats` が設定済みの場合は、現在の設定を表示してから質問する。

```
question: "HTMLで出力するファイルを選んでください（未選択のファイルはMarkdownで出力されます）"
header: "出力形式"
multiSelect: true
options:
  - label: "hearing-notes"
    description: "ヒアリング結果"
  - label: "exploration-report"
    description: "コードベース探索レポート"
  - label: "implementation-plan"
    description: "実装計画書"
  - label: "tasks"
    description: "タスクリスト"
  - label: "tech-reference"
    description: "技術リファレンス（初学者向け）"
```

選択結果から `output-formats` マップを構築する:
- 選択されたファイル → `html`
- 選択されなかったファイル → `md`

**何も選択されなかった場合**: すべて `md` として設定する。

### Step 3.5: スキップするファイルの選択

AskUserQuestion（multiSelect）で、**生成をスキップするファイル**を選択させる。

既存の `.config.yml` に `skip-files` が設定済みの場合は、現在の設定を表示してから質問する。

```
question: "生成をスキップするファイルがあれば選んでください（未選択＝すべて生成）"
header: "スキップ"
multiSelect: true
options:
  - label: "tech-reference"
    description: "技術リファレンス（初学者向け解説）を生成しない"
  - label: "test-cases"
    description: "テストケース詳細仕様（test-cases.html）を生成しない。spec-driven-dev で有効"
  - label: "understanding-quiz"
    description: "理解度クイズ（実装前 understanding-quiz-plan.html / 実装後 understanding-quiz-impl.html）を生成しない"
```

選択結果から `skip-files` リストを構築する。何も選択されなかった場合は `skip-files` キーを出力しない。

`test-cases` は `spec-driven-dev` で意味を持つ（test-cases.html を生成するスキル。他のスキルでは無視される）。
`understanding-quiz` は `spec-driven-dev`（実装前クイズ）と `spec-implement`（実装後クイズ）で意味を持つ。

### Step 3.7: クイズの出力先の選択

**Step 3.5 で `understanding-quiz` がスキップ対象に選ばれた場合はこのステップを省略する**（クイズを生成しないため）。

AskUserQuestion（1回の呼び出しに2問）で、実装前クイズ・実装後クイズそれぞれの出力先を選択させる。

既存の `.config.yml` に `quiz-output` が設定済みの場合は、現在の設定を表示してから質問する。

```
質問1:
question: "実装前クイズ（understanding-quiz-plan）の出力先を選んでください"
header: "実装前クイズ"
multiSelect: false
options:
  - label: "ファイル (Recommended)"
    description: "specフォルダ内の自己完結HTMLとして出力する（未設定時のデフォルト）"
  - label: "アーティファクト"
    description: "Artifact ツールで公開する"
  - label: "対話モード"
    description: "HTMLを生成せず、セッション内で AskUserQuestion により1問ずつ出題する（回答ごとに正誤と解説を提示）"

質問2:
question: "実装後クイズ（understanding-quiz-impl）の出力先を選んでください"
header: "実装後クイズ"
multiSelect: false
options:
  - label: "アーティファクト (Recommended)"
    description: "Artifact ツールで公開する（未設定時のデフォルト）"
  - label: "ファイル"
    description: "specフォルダ内の自己完結HTMLとして出力する"
  - label: "対話モード"
    description: "HTMLを生成せず、セッション内で AskUserQuestion により1問ずつ出題する（回答ごとに正誤と解説を提示）"
```

選択結果から `quiz-output` マップを構築する:
- 「ファイル」 → `file` / 「アーティファクト」 → `artifact` / 「対話モード」 → `interactive`

### Step 4: 設定ファイルの書き出し

Write ツールで `.plugin-workspace/.specs/.config.yml` に書き出す:

```yaml
# spec-plugin 設定
# 各スキルの --review 引数で個別にオーバーライド可能

review-tool: {選択した値}

# ファイルごとの出力形式（md / html）
output-formats:
  hearing-notes: {md or html}
  exploration-report: {md or html}
  implementation-plan: {md or html}
  tasks: {md or html}
  tech-reference: {md or html}

# 理解度クイズの出力先
# file = specフォルダ内HTML / artifact = Artifactツールで公開 / interactive = セッション内で対話出題
# 未設定時のデフォルト: plan は file、impl は artifact
quiz-output:
  plan: {file / artifact / interactive}
  impl: {file / artifact / interactive}

# 生成をスキップするファイル（Step 3.5 で選択された場合のみ出力）
# test-cases は spec-driven-dev で有効
# understanding-quiz は spec-driven-dev（実装前）/ spec-implement（実装後）で有効
skip-files:
  - tech-reference
  - test-cases
  - understanding-quiz
```

### Step 5: 完了報告

設定内容をユーザーに提示し、以下を案内:

- この設定は `/spec-driven-dev`, `/spec-implement`, `/spec-implement-auto` で自動適用される
- `--review` 引数で個別にオーバーライド可能
- 設定を変更したい場合は `/spec-setup` を再実行
