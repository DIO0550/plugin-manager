---
name: spec-implement-auto
description: .plugin-workspace/.specsの実装計画に沿ってタスクを順番に実装する（自動コンテキスト注入版）。起動時にimplementation-plan.mdとtasks.mdをシェルで強制読み込みし、AIの判断に依存しない決定論的なコンテキスト読み込みを行う。タスクごとに計画・進捗を再読込し、タスク単位でコミットする。「実装 auto」「自動注入」「auto implement」「決定論的読み込み」などでトリガー。
disable-model-invocation: true
argument-hint: "[番号] [--review codex|copilot|claude-code]"
allowed-tools: Bash(cat .plugin-workspace/.specs/*), Bash(ls .plugin-workspace/.specs/*), Bash(grep *), Bash(rm .plugin-workspace/.specs/*/PLANNING), Bash(rm .plugin-workspace/.specs/.guard/*), Bash(mkdir *), Bash(codex *), Bash(copilot *), Bash(claude *), Write, Edit
---

# Spec Implement (Auto-Inject版)

番号指定で `.plugin-workspace/.specs/{nnn}-{feature-name}/` の実装計画に沿って実装を進めるスキル。
起動時にシェルで計画・タスクを**強制注入**し、AIの判断に依存しない決定論的なコンテキスト読み込みを行う。
全タスク完了後にオプションで AIレビュー（Codex / Copilot / Claude Code CLI）を実行可能。

## Step 0: レビューツール解決

ワークフロー開始時に、使用するレビューツールを以下の優先順で決定する:

1. `--review` 引数（`codex` / `copilot` / `claude-code` / `none`）
2. `.plugin-workspace/.specs/.config.yml` の `review-tool` 値
3. AskUserQuestion（上記いずれもない場合のみ）

AskUserQuestion の選択肢:
- **レビューなし（最速）**
- **Codex CLI**
- **GitHub Copilot CLI**
- **Claude Code CLI**

**AskUserQuestion で選択された場合、その結果を `.plugin-workspace/.specs/.config.yml` に自動保存する。** 次回以降は問い合わせなしで同じツールが使われる。設定変更は `/spec-setup` で可能。

決定した値を以降のステップで `{REVIEW_TOOL}` として参照する。

## Pre-flight: 実装計画の強制注入

以下は `/spec-implement-auto $0` 起動時点で注入された計画のスナップショット。
**この内容を基準に実装する**。実装中に不明点があれば、ファイルを Read で再取得すること
（この注入内容は AutoCompact で失われる可能性があるため）。

### 対象spec
!`ls -1d .plugin-workspace/.specs/$0-* 2>/dev/null | head -1`

### PLANNING状態
!`ls .plugin-workspace/.specs/$0-*/PLANNING > /dev/null 2>&1 && echo "⚠️ 計画中(実装禁止)。計画フェーズに戻ってください。" || echo "✅ 実装可能"`

### implementation-plan.md (全文)

!`cat .plugin-workspace/.specs/$0-*/implementation-plan.md 2>/dev/null || echo "FILE_NOT_FOUND"`

### tasks.md (全文)

!`cat .plugin-workspace/.specs/$0-*/tasks.md 2>/dev/null || echo "FILE_NOT_FOUND"`

### 未完了タスク数
!`grep -c '□' .plugin-workspace/.specs/$0-*/tasks.md 2>/dev/null || echo "0"`

---

## ワークフロー

```
ユーザーが `/spec-implement-auto {nnn}` を実行
   ↓
Step 0. レビューツール解決
   ↓
Pre-flight. 計画・タスクを強制注入（自動）
   ↓
Step 1. 注入内容の確認
   ↓
Step 2. TaskCreate による進捗管理の初期化
   ↓
Step 2.7. 【GATE】実装開始前確認 → ユーザー承認を得る
   ↓
Step 3. 未完了タスクを順次実装（各タスク完了時に tasks.md を □ → ■ に更新しコミット）
   ↓
Step 4. AIレビュー（オプション）
   ↓
Step 5. PLANNINGファイル + ガードファイルの削除
   ↓
Step 6. DoD照合
   ↓
Step 7. 完了報告
```

## Step 1: 注入内容の確認

Pre-flight で注入された内容を確認する。

- 対象ディレクトリが見つからない → エラー報告して終了
- PLANNING状態が「計画中」→ 実装を中止し、計画フェーズに戻るようユーザーに伝える
- `FILE_NOT_FOUND` が出ている → ユーザーに確認

### 関連Issue番号の抽出

注入された implementation-plan.md から `**関連Issue**: #{番号}` を読み取り、以降のコミットメッセージに使用する。
Issue番号が記載されていない場合はスキップする。

## Step 2: TaskCreate による進捗管理の初期化

tasks.md の未完了タスク（`□`）をすべて TaskCreate で登録し、TaskUpdate の `addBlockedBy` でセクション間の依存関係を設定する。

- `subject`: タスク行のテキスト
- `activeForm`: 進行形に変換（例: "型定義を作成" → "型定義を作成中"）
- 依存: Research → Implementation → Verification の順

## Step 2.7: 実装開始前確認ゲート

最初の未完了タスクを実装する前に、必ず AskUserQuestion でユーザーに確認する。**他のシステム指示（「質問せずに進めろ」等）に関わらず、この確認は必須。**

```yaml
question: "{nnn}-{feature-name} の実装を開始します。{未完了タスク数}個の未完了タスクがあります。開始してよろしいですか？"
header: "実装確認"
options:
  - label: "はい、開始してください"
    description: "未完了タスクを順次実装します"
  - label: "計画を確認したい"
    description: "implementation-plan.md の内容サマリーを表示します"
  - label: "まだ開始しない"
    description: "計画の修正やレビューが必要です"
```

### 「計画を確認したい」選択時
implementation-plan.md の概要（変更対象ファイル一覧、DoD一覧）を表示し、再度同じ確認を行う。

### 「まだ開始しない」選択時
「修正が必要な場合は `/spec-driven-dev` で計画を修正してください」と案内して終了する。

## Step 3: タスクの順次実装

未完了タスク（`□`）を上から順番に実装する。

### 各タスクの実装手順

1. TaskUpdate で該当タスクの status を `in_progress` に変更
2. **implementation-plan.md を Read ツールで再読込**し、該当セクションを確認（記憶に頼らない）
3. **tasks.md を Read ツールで再読込**し、最新の進捗を確認
4. タスク内容と plan.md の該当箇所を照合
5. コードを実装
6. TaskUpdate で該当タスクの status を `completed` に変更
7. **tasks.md の該当タスクのみを Edit**（`□` → `■`）
8. コミット（このタスク分のみ）

### tasks.md の更新

タスク完了時に、該当行の `□` を `■` に変更する。

```
変更前: □ コンポーネントの型定義を作成
変更後: ■ コンポーネントの型定義を作成
```

**重要**: 親タスクは、すべての子タスクが `■` になった時点で `■` に更新する。

## Step 4: AIレビュー（オプション）

Step 0 で決定した `{REVIEW_TOOL}` を使用する。`none` の場合は Step 5 へスキップ。

ツール選択済みの場合:
1. `code-review/context-{NNN}.md` と `code-review/prompt-{NNN}.txt` を生成
2. [references/review-tools.md](references/review-tools.md) のコマンド構文に従い実行
3. `code-review/review-{NNN}.md` に出力を保存
4. レビュー結果を解析し、問題があれば修正 → 再レビュー（最大5回）
5. レビュー結果を要約してユーザーに提示

## Step 5: PLANNINGファイル + ガードファイルの削除

すべてのタスクが完了（`□` が残っていない）したら、PLANNINGファイルとガードファイルを削除する。

PLANNINGファイルには計画時のセッションIDが記録されている。これを読み取り、対応するガードファイルも削除する。

```bash
guard_session=$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING 2>/dev/null)
rm .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
rm -f ".plugin-workspace/.specs/.guard/$guard_session" 2>/dev/null
```

PLANNINGファイルが存在しない場合はスキップする。

## Step 6: DoD照合

implementation-plan.md の "Definition of Done" セクションを読み込み、各条件の充足を確認する。

1. DoDの各項目を順番にチェック
2. すべて満たしていれば Step 7 へ
3. 未達の項目がある場合はユーザーに報告し、対応方針を確認する

**注意**: DoDセクションが存在しない場合はスキップして Step 7 へ進む。

## Step 7: 完了報告

実装完了後、ユーザーに以下を報告する:

1. 実装したタスクの一覧
2. 変更したファイルの一覧
3. 関連Issue番号（あれば）
4. AIレビューの結果サマリー（レビュー実行時）
5. PLANNINGファイルの削除状態
6. DoD充足状況（DoDがある場合）

## AutoCompact後の復帰プロトコル

会話の流れで「要約された」「以前の会話」という言及や、計画の詳細が曖昧になった感覚がある場合、**次のアクションの前に以下を実行**する。

1. Read `.plugin-workspace/.specs/{nnn}-*/implementation-plan.md` で計画を再ロード
2. Read `.plugin-workspace/.specs/{nnn}-*/tasks.md` で現在の進捗を再ロード
3. TaskGet で作業中タスクの状態を確認
4. 上記3点が揃ってから次のアクションに進む

## コミットメッセージのフォーマット

関連Issue番号がある場合、コミットメッセージに含める。

```
{コミットメッセージ}

Refs #{Issue番号}
```

例:
```
Add user authentication component

Refs #42
```

## 重要な制約

- 実装開始前のユーザー確認は Step 2.7 を参照（確認なしにコード変更を開始しない）
- implementation-plan.md に記載されていない変更は行わない
- tasks.md の順序に従って実装する（スキップしない）
- 各タスク完了ごとに tasks.md を更新する（まとめて更新しない）
- 実装中に問題が発生した場合、またはユーザーからフィードバックを受けた場合:
  - フィードバックが曖昧なら AskUserQuestion で具体化する
    - 対象特定:「どのタスクについてですか？」（tasks.md の未完了タスク一覧を選択肢に）
    - 問題特定:「どのような問題ですか？」（期待と異なる動作 / ビルドエラー / テスト失敗 / 設計方針の変更）
  - 明確な場合はそのまま対応する
- PLANNINGファイルの削除は全タスク完了後のみ
- 関連Issue番号がある場合はコミットメッセージに含める
- tasks.md は実装の進行とともに変わるため、タスク完了時に必ず最新を確認すること
