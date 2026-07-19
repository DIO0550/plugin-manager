---
name: spec-based-code-review
description: コードレビュースキル（オーケストレーター）。実装全体を多観点（パフォーマンス・設計・コメント品質・テスト品質・仕様整合性）で統合レビューする場合に使用する。専門サブエージェントが並列でレビューを実行し、結果を統合する。spec番号を指定すると計画ドキュメントも参考にする。テストコード単体を素早くレビューしたい場合は test-review スキルを使用すること。「specレビュー」「仕様レビュー」「spec review」「仕様整合性チェック」「計画に基づくレビュー」「実装が計画通りか確認」「意図ベースレビュー」「コードレビュー」「統合レビュー」「spec-based-code-review」「コメントレビュー」「docコメント」「コメント品質」などでトリガー。
disable-model-invocation: true
argument-hint: "[番号]"
---

# Spec-Based Code Review（オーケストレーター）

ユニバーサルな原則に基づくコードレビューを実行するスキル。
自分ではレビューせず、専門サブエージェントを並列起動してレビューを委譲し、結果を統合する。

- **常に起動**: performance-reviewer, design-reviewer, test-quality-reviewer, comment-quality-reviewer（ユニバーサル原則ベース）
- **specがある場合のみ追加**: spec-alignment-reviewer（仕様整合性チェック）

spec番号を指定すると計画ドキュメントも参照し、仕様整合性チェックが追加される。

## ワークフロー

```
Step 0: Specフォルダの特定（任意）
  ↓
Step 1: 計画ドキュメントの読み込み（specがある場合のみ）
  ↓
Step 2: 実装コードの収集（git diff + 変更ファイル）
  ↓
Step 3: サブエージェントを並列起動（4 or 5エージェント）
  ↓
Step 4: 結果統合・重複排除・保存
  ↓
Step 5: ユーザーへの提示とアクション提案
```

## Step 0: Specフォルダの特定（任意）

### 番号が指定された場合

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/$0-* 2>/dev/null | head -1)
```

マッチしない場合は archive 内も検索する:

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/archive/$0-* 2>/dev/null | head -1)
```

いずれもマッチしない場合はエラーメッセージを表示して終了（番号を明示指定したのに見つからない場合はユーザーの意図と異なるため）。

### 番号が省略された場合

archive外で最大番号のspecを自動選択:

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/[0-9][0-9][0-9]-* 2>/dev/null | sort -rn | head -1)
```

**specフォルダが見つかった場合**: 自動選択したspecのフォルダ名をユーザーに表示する。

**specフォルダが見つからない場合**: 「specフォルダが見つかりません。ユニバーサル原則に基づくレビューを実行します。」と表示して Step 2 に進む。

### 検証（specフォルダがある場合のみ）

以下の存在を確認する（.md または .html）:

1. `implementation-plan.md`（推奨）— 存在しない場合は WARNING を表示して続行
2. `tasks.md`（推奨）— 存在しない場合は WARNING を表示して続行
3. `hearing-notes.md`（推奨）— 存在しない場合は WARNING を表示して続行
4. `exploration-report.md`（推奨）— 存在しない場合は WARNING を表示して続行

### PLANNINGファイルの検出（specフォルダがある場合のみ）

`PLANNING` ファイルが存在する場合、計画フェーズがまだ進行中。AskUserQuestion で確認する:

```yaml
question: "PLANNINGファイルが残っています。計画フェーズがまだ進行中の可能性があります。現時点のコードでレビューを実行しますか？"
header: "PLANNING検出"
options:
  - label: "はい、レビューを実行"
    description: "現時点の実装コードに対してレビューを行います"
  - label: "いいえ、中止"
    description: "計画完了後に再度実行してください"
```

## Step 1: 計画ドキュメントの読み込み

specフォルダが見つかった場合、以下のファイルを Read で読み込む（存在するもののみ）:

1. `{spec_dir}/hearing-notes.md`（または `.html`）
2. `{spec_dir}/exploration-report.md`（または `.html`）
3. `{spec_dir}/implementation-plan.md`（または `.html`）
4. `{spec_dir}/tasks.md`（または `.html`）

### 抽出する情報

各文書から以下の情報をサブエージェントへの入力用に整理する:

- **目的とスコープ**（hearing-notes）
- **技術的制約と既存パターン**（exploration-report）
- **変更ファイル一覧**: `[NEW]` / `[MODIFY]` / `[DELETE]`（implementation-plan）
- **設計方針・データフロー・状態遷移**（implementation-plan）
- **テスト戦略とテストTODOリスト**（implementation-plan）
- **Definition of Done**（implementation-plan）
- **タスク完了状態**: ■ / □（tasks.md）

## Step 2: 実装コードの収集

### レビュー対象の決定

AskUserQuestion でレビュー範囲を確認する:

```yaml
question: "レビュー対象を選択してください"
header: "レビュー範囲"
options:
  - label: "全実装コード（推奨）"
    description: "implementation-planに記載された全ファイルを対象にレビュー"
  - label: "完了タスクのみ"
    description: "tasks.mdで■になっているタスクの関連ファイルのみ"
  - label: "git diff（直近の変更）"
    description: "最新のgit diffを対象にレビュー"
```

### コード情報の収集

選択に応じてコード情報を収集する:

**「全実装コード」の場合**:
1. implementation-plan の変更ファイル一覧（`[NEW]`/`[MODIFY]`/`[DELETE]`）を抽出
2. 各ファイルの存在確認
3. `git diff` で変更内容を取得

**「完了タスクのみ」の場合**:
1. tasks.md から ■ のタスクを抽出
2. 対応するファイルを特定
3. 該当ファイルの `git diff` を取得

**「git diff」の場合**:
1. `git diff` で変更一覧を取得
2. `git diff --name-only` で変更ファイル一覧を取得

### 収集結果の整理

サブエージェントに渡すための情報を整理する:
- 変更ファイル一覧（パス）
- git diff の内容
- `[NEW]` ファイルのフルパス

## Step 3: サブエージェントを並列起動

**パス解決**: 以下のプロンプト内の `{spec-based-code-review-plugin-path}` は、このプラグインのルートディレクトリ（本スキルの SKILL.md があるディレクトリから2階層上、`.../plugins/spec-based-code-review-plugin`）を指す。実際の絶対パスに置き換えてサブエージェントに渡すこと。

**specフォルダがある場合**: 5エージェント（performance, design, spec-alignment, test-quality, comment-quality）を起動
**specフォルダがない場合**: 4エージェント（performance, design, test-quality, comment-quality）を起動。spec-alignment-reviewer は仕様整合性チェックが本質のため、spec不在時は起動しない。

出力先ディレクトリを作成（specフォルダがある場合のみ）:

```bash
mkdir -p {spec_dir}/spec-based-code-review
```

### 連番の決定

既存のレビューファイルから次の連番を決定する:

```bash
next_num=$(printf "%03d" $(( $(ls -1 {spec_dir}/spec-based-code-review/review-*.md 2>/dev/null | wc -l | tr -d ' ') + 1 )))
```

### サブエージェントを Task で起動

```
Task tool: (並列起動)

1. performance-reviewer:
   description: "performance-reviewer: {feature-name}"
   run_in_background: true
   prompt: |
     あなたは performance-reviewer エージェントです。
     ユニバーサルなパフォーマンス原則に基づいてレビューしてください。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md

     ## 計画ドキュメント（参考情報）
     {specフォルダがある場合:
     - {spec_dir}/implementation-plan.md
     - {spec_dir}/exploration-report.md
     }
     {specフォルダがない場合: 計画ドキュメントのセクションごと省略}

     ## 実装コード情報
     変更ファイル一覧:
     {変更ファイルリスト}

     git diff:
     {git diff の内容（長すぎる場合はファイル単位で分割）}

     ## 出力先
     {specフォルダがある場合: {spec_dir}/spec-based-code-review/performance-{NNN}.md}
     {specフォルダがない場合: 「ファイル出力不要。結果をテキストで返してください。」}

     ## 重要
     - すべての指摘に「根拠」を含めること（ルール根拠 or 仕様根拠）
     - ユニバーサル原則（N+1, メモリリーク等）はルール根拠のみで指摘可能

2. design-reviewer:
   description: "design-reviewer: {feature-name}"
   run_in_background: true
   prompt: |
     あなたは design-reviewer エージェントです。
     ユニバーサルな設計原則に基づいてレビューしてください。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md

     ## 計画ドキュメント（参考情報）
     {specフォルダがある場合:
     - {spec_dir}/implementation-plan.md
     - {spec_dir}/exploration-report.md
     }
     {specフォルダがない場合: 計画ドキュメントのセクションごと省略}

     ## 実装コード情報
     変更ファイル一覧:
     {変更ファイルリスト}

     git diff:
     {git diff の内容}

     ## 出力先
     {specフォルダがある場合: {spec_dir}/spec-based-code-review/design-{NNN}.md}
     {specフォルダがない場合: 「ファイル出力不要。結果をテキストで返してください。」}

     ## 重要
     - すべての指摘に「根拠」を含めること（ルール根拠 or 仕様根拠）
     - ユニバーサル原則（SRP, DIP, KISS, 凝集度, 不要な防御的コードの排除）はルール根拠のみで指摘可能
     - 不要なフォールバック・防御的コード（次元16）の判定では、仕様が要求する劣化提供かを次元5と併せて確認すること

3. spec-alignment-reviewer:（specフォルダがある場合のみ起動）
   description: "spec-alignment-reviewer: {feature-name}"
   run_in_background: true
   prompt: |
     あなたは spec-alignment-reviewer エージェントです。
     全計画ドキュメントを読み込んで実装の意図を完全に理解してから、仕様整合性の6次元でレビューしてください。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md

     ## 計画ドキュメント（全て読むこと）
     - {spec_dir}/hearing-notes.md
     - {spec_dir}/exploration-report.md
     - {spec_dir}/implementation-plan.md
     - {spec_dir}/tasks.md

     ## 実装コード情報
     変更ファイル一覧:
     {変更ファイルリスト}

     git diff:
     {git diff の内容}

     ## 出力先
     {spec_dir}/spec-based-code-review/alignment-{NNN}.md

     ## 重要
     - 4つ全ての計画ドキュメントを読んでからコードを読むこと
     - すべての指摘に「仕様根拠」を含めること
     - 仕様根拠のない指摘は出さないこと
     - 意図的複雑性の保護（次元5）を最優先で判定すること

4. test-quality-reviewer:
   description: "test-quality-reviewer: {feature-name}"
   run_in_background: true
   prompt: |
     あなたは test-quality-reviewer エージェントです。
     古典学派のテスト原則（普遍的ルール）に基づいてテストコードをレビューしてください。
     指摘の根拠はルール自体です。計画ドキュメントがあれば参考にしてください。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/test-review-rules.md

     ## 計画ドキュメント（参考情報）
     {specフォルダがある場合:
     - {spec_dir}/implementation-plan.md
     - {spec_dir}/exploration-report.md
     }
     {specフォルダがない場合: 計画ドキュメントのセクションごと省略}

     ## テストコード情報
     テストファイル一覧:
     {テストファイルリスト}

     git diff（テスト部分）:
     {git diff の内容（テストファイル部分）}

     ## 出力先
     {specフォルダがある場合: {spec_dir}/spec-based-code-review/test-quality-{NNN}.md}
     {specフォルダがない場合: 「ファイル出力不要。結果をテキストで返してください。」}

     ## 重要
     - テストコードだけでなく、テスト対象の実装コードも必ず読むこと（次元12・13に必須）
     - 指摘の根拠はルール（普遍的原則）— spec文書の有無に関わらず指摘を出す
     - 外部依存への spy アサーションは CRITICAL にしないこと

5. comment-quality-reviewer:
   description: "comment-quality-reviewer: {feature-name}"
   run_in_background: true
   prompt: |
     あなたは comment-quality-reviewer エージェントです。
     ユニバーサルなコメント原則に基づいてレビューしてください。
     コメントのwhy原則（次元14）とdocコメント整合性（次元15）が担当です。

     ## レビュー基準
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/review-dimensions.md
     Read: {spec-based-code-review-plugin-path}/skills/spec-based-code-review/references/finding-classification.md

     ## 計画ドキュメント（参考情報）
     {specフォルダがある場合:
     - {spec_dir}/implementation-plan.md
     - {spec_dir}/exploration-report.md
     }
     {specフォルダがない場合: 計画ドキュメントのセクションごと省略}

     ## 実装コード情報
     変更ファイル一覧:
     {変更ファイルリスト}

     git diff:
     {git diff の内容}

     ## 出力先
     {specフォルダがある場合: {spec_dir}/spec-based-code-review/comment-quality-{NNN}.md}
     {specフォルダがない場合: 「ファイル出力不要。結果をテキストで返してください。」}

     ## 重要
     - すべての指摘に「根拠」を含めること（ルール根拠 or 仕様根拠）
     - whyコメント（制約・理由・注意喚起）は保護すること — 指摘対象は what / how / 履歴コメントのみ
     - docコメント整合性の検証は diff だけでなく関数全体を読んでから行うこと
```

### 完了待ち

```
TaskOutput: (起動したエージェントそれぞれ — 4つまたは5つ)
  task_id: "{各エージェントのtask_id}"
  block: true
  timeout: 300000
```

## Step 4: 結果統合・重複排除・保存

個別レポートを Read で読み込む:

**specフォルダがある場合（5レポート）**:
1. `{spec_dir}/spec-based-code-review/performance-{NNN}.md`
2. `{spec_dir}/spec-based-code-review/design-{NNN}.md`
3. `{spec_dir}/spec-based-code-review/alignment-{NNN}.md`
4. `{spec_dir}/spec-based-code-review/test-quality-{NNN}.md`
5. `{spec_dir}/spec-based-code-review/comment-quality-{NNN}.md`

**specフォルダがない場合（4レポート — テキスト出力を使用）**:
1. performance-reviewer の出力テキスト
2. design-reviewer の出力テキスト
3. test-quality-reviewer の出力テキスト
4. comment-quality-reviewer の出力テキスト

### 統合ルール

1. **重複排除**: 同じファイル・同じ行範囲に対する指摘をマージ
2. **通し番号で再採番**: C-001, W-001, I-001 から順に
3. **DoD充足状況**: spec-alignment-reviewer が起動している場合、DoD 検証結果をそのまま転記
4. **サマリー集計**: 各分類の件数を集計

### 統合レポート生成

テンプレート `spec-based-code-review:review-report` に沿って統合レポートを生成する。

**specフォルダがある場合**:
```
Write: {spec_dir}/spec-based-code-review/review-{NNN}.md
```

**specフォルダがない場合**: ファイル出力せず、統合結果をテキストで返す。

## Step 5: ユーザーへの提示とアクション提案

### レビュー結果サマリーの提示

統合レポートのサマリーをユーザーに提示する:

- CRITICAL / WARNING / INFO の件数
- CRITICAL がある場合は具体的な指摘内容を表示

### 対応アクションの提案

CRITICAL または WARNING がある場合:

```yaml
question: "レビュー結果への対応を選択してください"
header: "対応方針"
options:
  - label: "指摘を修正する"
    description: "CRITICAL/WARNINGの指摘に対応します"
  - label: "レビュー結果を確認のみ"
    description: "レビュー結果を保存し、後で対応します"
  - label: "再レビュー"
    description: "修正後にもう一度レビューを実行します（連番インクリメント）"
```

### 「指摘を修正する」選択時

CRITICAL → WARNING の優先順でユーザーに指摘内容を提示し、修正を進める。修正後に「再レビュー」を提案する。

### 「再レビュー」選択時

連番をインクリメントして Step 2 からやり直す。最大5回までループ。5回超えたらユーザーに相談。

### 全て OK の場合

CRITICAL も WARNING もない場合は「コードレビュー完了 — 問題なし」と報告。

## 出力ディレクトリ

**specフォルダがある場合**:
```
.plugin-workspace/.specs/{nnn}-{feature-name}/
└── spec-based-code-review/
    ├── performance-{NNN}.md       # performance-reviewer 出力
    ├── design-{NNN}.md            # design-reviewer 出力
    ├── alignment-{NNN}.md         # spec-alignment-reviewer 出力
    ├── test-quality-{NNN}.md      # test-quality-reviewer 出力
    ├── comment-quality-{NNN}.md   # comment-quality-reviewer 出力
    └── review-{NNN}.md            # 統合レポート
```

**specフォルダがない場合**: ファイル出力なし。結果はテキストで直接提示。

## 重要な制約

- **コードの変更はオーケストレーター自身では行わない** — サブエージェントもレビュー結果の出力のみ
- specフォルダがなくてもレビューは実行できる（ユニバーサル原則のみで動作）
- 計画ドキュメントがある場合は参考にする。ない場合はユニバーサル原則のみでレビューする
- 再レビューは最大5回まで
