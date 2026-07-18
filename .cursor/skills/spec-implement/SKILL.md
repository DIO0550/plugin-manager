---
name: spec-implement
description: .plugin-workspace/.specsの実装計画に沿ってタスクを順番に実装する。番号を指定すると該当specを、省略するとarchive外で最も番号が大きいspecを自動選択し、tasks.mdを読み込んで未完了タスクを順次実装していく。全タスク完了後にオプションでCodex/Copilot/Claude Codeによるコードレビューを実行可能。「実装」「implement」「タスク実装」「コードレビュー付き」「codexレビュー」「copilotレビュー」などでトリガー。
disable-model-invocation: true
argument-hint: "[番号] [--review codex|copilot|claude-code]"
allowed-tools: Bash(rm .plugin-workspace/.specs/*/PLANNING), Bash(rm .plugin-workspace/.specs/.guard/*), Bash(mkdir *), Bash(codex *), Bash(copilot *), Bash(claude *), Write, Edit
---

# Spec Implement

番号指定で `.plugin-workspace/.specs/{nnn}-{feature-name}/` の実装計画に沿って実装を進めるスキル。
番号を省略した場合はarchive外で最も番号が大きいspecを自動選択する。
全タスク完了後にオプションで AIレビュー（Codex / Copilot / Claude Code CLI）を実行可能。

## ワークフロー

```
ユーザーが `/spec-implement {nnn}` または `/spec-implement` を実行
   ↓
Step 0. レビューツール解決（引数 → .config.yml → 初回のみ AskUserQuestion）
   ↓
Step 1. specフォルダの特定
   （番号指定時: {nnn}-* にマッチするフォルダ / 番号省略時: archive外で最大番号のspecを自動選択）
   ↓
Step 2. implementation-plan.md を読み込み、変更内容を把握
   ↓
Step 3. tasks.md を読み込み、未完了タスクを確認
   ↓
Step 3.5. TaskCreate による進捗管理の初期化
   ↓
Step 3.7. 【GATE】実装開始前確認 → ユーザー承認を得る
   ↓
Step 4. タスクを順番に実装（各タスク完了時に tasks.md を □ → ■ に更新
        ＋ Deviations・既存依存の挙動を implementation-notes.md に随時記録）
   ↓
Step 5. AIレビュー（オプション）
   ↓
Step 6. PLANNINGファイル + ガードファイルの削除
   ↓
Step 7. DoD照合
   ↓
Step 7.5. 実装後理解度クイズ生成 → Artifact（デフォルト）/ understanding-quiz-impl.html / 対話出題（push 前ゲート・advisory）
   ↓
Step 8. 完了報告
```

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

## Step 1: specフォルダの特定

### 番号が指定された場合

指定された番号 `$0` を使い、`.plugin-workspace/.specs/` 配下からマッチするフォルダを検索する。

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/$0-* 2>/dev/null | head -1)
```

- マッチするフォルダが見つからない場合はエラーメッセージを表示して終了
- 複数マッチした場合は最初のものを使用

### 番号が省略された場合

archive外のspecから最も番号が大きいものを自動選択する。

```bash
spec_dir=$(ls -1d .plugin-workspace/.specs/[0-9][0-9][0-9]-* 2>/dev/null | sort -rn | head -1)
```

- `.plugin-workspace/.specs/` 直下のみ検索（`archive/` 配下は対象外）
- 該当するspecが存在しない場合はエラーメッセージを表示して終了
- 自動選択したspecのフォルダ名をユーザーに表示する（例: 「`003-feature-name` を自動選択しました」）

## Step 2: implementation-plan.md の読み込み

`.plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan.md` を読み込み、以下を把握する：

- 変更対象ファイル（`[NEW]` `[MODIFY]` `[DELETE]`）
- 設計方針
- データ構造・API設計
- 検証計画
- **関連Issue番号**（`**関連Issue**: #123` の形式で記載されている場合）

### 関連Issue番号の抽出

implementation-plan.md から `**関連Issue**: #{番号}` を読み取り、以降のコミットメッセージに使用する。
Issue番号が記載されていない場合はスキップする。

## Step 3: tasks.md の読み込み

`.plugin-workspace/.specs/{nnn}-{feature-name}/tasks.md` を読み込み、タスク状態を確認する。

### タスク状態の判定

- `□` — 未完了タスク（実装対象）
- `■` — 完了済みタスク（スキップ）

未完了タスク（`□`）がない場合は「全タスク完了済み」と報告して終了。

## Step 3.5: TaskCreate による進捗管理の初期化

tasks.md の未完了タスク（`□`）をすべて TaskCreate で登録し、TaskUpdate の `addBlockedBy` でセクション間の依存関係を設定する。

- `subject`: タスク行のテキスト
- `activeForm`: 進行形に変換（例: "型定義を作成" → "型定義を作成中"）
- 依存: Research → Implementation → Verification の順

## Step 3.7: 実装開始前確認ゲート

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

## Step 4: タスクの順次実装

未完了タスク（`□`）を上から順番に実装する。

### 各タスクの実装手順

1. TaskUpdate で該当タスクの status を `in_progress` に変更
2. タスク内容を確認
3. implementation-plan.md の該当セクションを参照
4. コードを実装
5. TaskUpdate で該当タスクの status を `completed` に変更
6. tasks.md の該当タスクを更新（`□` → `■`）

### tasks.md の更新

タスク完了時に、該当行の `□` を `■` に変更する。

```
変更前: □ コンポーネントの型定義を作成
変更後: ■ コンポーネントの型定義を作成
```

**重要**: 親タスクは、すべての子タスクが `■` になった時点で `■` に更新する。

### 実装中ノートの記録（implementation-notes.md）

> **このノートは実装後クイズ（Step 7.5）の出題材料。`.config.yml` の `skip-files` に `understanding-quiz` が含まれていて Step 7.5 をスキップする場合は、この記録も省略してよい。**

実装を進めながら、`implementation-notes.md` に「実際にやったこと」を随時記録する。これは Step 7.5 の実装後理解度クイズの出題材料になる（diff の表面に出ない挙動を問うため）。

- 初回の実装着手時にテンプレート `assets/templates/implementation-notes.md` を
  `.plugin-workspace/.specs/{nnn}-{feature-name}/implementation-notes.md` にコピーして起こす
- 実装中、以下が発生したらその都度追記する:
  - **Deviations**: implementation-plan から外れた点とその理由
  - **既存コードパスへの依存で生じた挙動**: diff の表面に出ないが既存の関数・フック・設定に依存して発生する挙動（根拠となる file:line も）
  - **想定 vs 実際**: hearing-notes で想定した設計と実装結果が食い違った点
  - **実装判断メモ**: 命名・分割・エラー処理など、後から「なぜこうしたか」を問われうる判断

逸脱が一切なければ Deviations に「なし」と明記する。

## Step 5: AIレビュー（オプション）

Step 0 で決定した `{REVIEW_TOOL}` を使用する。`none` の場合は Step 6 へスキップ。

ツール選択済みの場合:
1. `code-review/context-{NNN}.md` と `code-review/prompt-{NNN}.txt` を生成
2. [references/review-tools.md](references/review-tools.md) のコマンド構文に従い実行
3. `code-review/review-{NNN}.md` に出力を保存
4. レビュー結果を解析し、問題があれば修正 → 再レビュー（最大5回）
5. レビュー結果を要約してユーザーに提示

## Step 6: PLANNINGファイル + ガードファイルの削除

すべてのタスクが完了（`□` が残っていない）したら、PLANNINGファイルとガードファイルを削除する。

PLANNINGファイルには計画時のセッションIDが記録されている。これを読み取り、対応するガードファイルも削除する。

```bash
# PLANNINGファイルからセッションIDを読み取り、ガードファイルを削除
guard_session=$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING 2>/dev/null)
rm .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
rm -f ".plugin-workspace/.specs/.guard/$guard_session" 2>/dev/null
```

PLANNINGファイルが存在しない場合はスキップする。

## Step 7: DoD照合

implementation-plan.md の "Definition of Done" セクションを読み込み、各条件の充足を確認する。

1. DoDの各項目を順番にチェック
2. すべて満たしていれば Step 8 へ
3. 未達の項目がある場合はユーザーに報告し、対応方針を確認する

**注意**: DoDセクションが存在しない場合はスキップして Step 8 へ進む。

## Step 7.5: 実装後理解度クイズ生成

push / merge の前に、**実装者（人間）が「実際に何が変わったか」を理解できているか**を測るクイズを生成する。`tsc --noEmit` が「型の関門」なのに対し、これは「人間の理解の関門」。落ちた設問がある間はまだ merge / push すべきでない、という **advisory ゲート**。

**生成の有無は `.plugin-workspace/.specs/.config.yml` の `skip-files` で決まる**（`/spec-setup` で設定）。`skip-files` に `understanding-quiz` が含まれている場合はこのステップをスキップして Step 8 へ進む。含まれていなければ生成する。

**出力先は `.config.yml` の `quiz-output.impl` で決まる**: `artifact`（Artifact ツールで公開）、`file`（specフォルダ内の HTML ファイル）、または `interactive`（HTML を生成せず、セッション内で AskUserQuestion により1問ずつ対話出題）。**未設定時のデフォルトは `artifact`**。決定した値を以降 `{QUIZ_OUTPUT}` として参照する。ただし `artifact` でも実行環境に Artifact ツールが存在しない場合（CLI 等）は `file` にフォールバックする。`interactive` でも対話できない実行環境（非対話実行）では `file` にフォールバックする。

- **材料**: diff（実際の変更）＋ implementation-notes.md（実装中ノート / Deviations）＋ hearing-notes（事前の意図）
  - hearing-notes（事前の意図）と implementation-notes（実際にやったこと）が別ファイルで分かれているため、両方を読ませると「想定した設計 vs 実際の実装」のズレをそのまま出題材料にできる
- **問う対象**: 実装で実際に何が行われたか / 途中で入った Deviations とその理由 / 既存コードパス依存で発生した挙動（diff の表面に出ない挙動を優先）
- **出力**: 解説＋クイズ（4択・YES/NO・並べ替え。回答した時点で1問ずつ正誤と解説を表示する即時フィードバック方式。初回回答でロックされ答えは変更不可。自己完結HTML）。`{QUIZ_OUTPUT}` に応じて Artifact 公開（デフォルト）/ `understanding-quiz-impl.html` ファイル / セッション内対話出題（`interactive`。HTML を生成せず AskUserQuestion で1問ずつ）
- **enforcement**: advisory。hook による機械的ブロックはしない（合否は自己申告のため）。

### 7.5-1. 材料の読み込み

- 変更差分（`git diff` の結果、または本セッションで Write/Edit した内容）
- `.plugin-workspace/.specs/{nnn}-{feature-name}/implementation-notes.md`（Step 4 で記録したノート）
- `.plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes.md`（事前の意図。存在すれば）

### 7.5-2. 設問の作成

`references/quiz-design.md` の「実装後クイズ（実際の挙動）の出題観点」に沿って、合計 5〜10 問（choice / boolean / order を混ぜる）作る。以下は必須:

- **Deviations を最低1問**（implementation-notes 由来。逸脱が「なし」ならその旨を問う設問でよい）
- **既存コードパス依存で生じた挙動**を優先的に出題（diff をざっと読むだけでは見えない挙動）
- **想定 vs 実際**（hearing-notes と実装のズレ）。`tag` に区分を付けるとバッジ表示される

クイズは回答した時点で1問ずつ正誤と解説が表示される（即時フィードバック）ため、ある設問の解説・選択肢が別の設問の答えを含まないよう設問間を独立させる（quiz-design.md 参照）。

作成後、`references/quiz-design.md` の「設問セルフレビュー」チェックリストで設問を1問ずつ検査し、該当する設問は修正するか捨てる（観点ごとに多めにドラフトして削るのが推奨フロー）。

### 7.5-3. クイズ生成

**`{QUIZ_OUTPUT}` が `interactive` の場合はこの節をスキップし、7.5-3b に進む。**

`test-cases.html` と同様の **DATA スクリプト差し替え方式**。CSS・レンダラ・採点ロジックは固定済み。

1. テンプレート `assets/templates/understanding-quiz-impl.html` を Read する
2. テンプレート先頭の DATA スクリプト（スキーマ説明コメント + `const QUIZ`）を実データで丸ごと置き換える
3. それ以外（`<style>`・レンダラの2スクリプト・HTMLシェル）は1文字も変更しない（自己完結HTML）
4. `<title>` の `{機能名}` を実際の機能名に置換する
5. `{QUIZ_OUTPUT}` に応じて出力する（プレースホルダ `{...}` を残さない）:
   - **`file` の場合**: `.plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-impl.html` に Write する
   - **`artifact` の場合**: 外側のシェルタグ（`<!doctype html>` `<html>` `<head>` `</head>` `<body>` `</body>` `</html>`）を取り除き、`<title>`・`<style>`・body 内容・スクリプトを残した形で `.plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-impl.html` に Write し、Artifact ツールで公開する（Artifact 側がシェルを付与するため。favicon は `📝`）

### 7.5-3b. 対話出題（`{QUIZ_OUTPUT}` が `interactive` の場合）

HTML を生成せず、`references/quiz-design.md` の「対話モード（quiz-output: interactive）の実施方法」に従い、7.5-2 で作成した設問を AskUserQuestion で1問ずつ出題する:

1. 出題前に「push / merge の前に、実装後の理解度クイズを{N}問、1問ずつ出題します」と伝える
2. 1問ずつ AskUserQuestion で出題（選択肢はシャッフル。order 設問は「正しい順序はどれか」の choice に変換）
3. 回答直後に正誤と解説（diff / implementation-notes の根拠箇所つき）を伝えてから次の設問へ。初回の回答で正誤を確定し、出し直さない
4. 全問終了後、スコアと passLine による合否、間違えた設問ごとに読み直すべき diff・ノートの該当箇所を提示する

### 7.5-4. ユーザーへの提示

- `{QUIZ_OUTPUT}` が `artifact` の場合: Artifact の URL を提示し、「実装後の理解度クイズをアーティファクトとして生成しました。push / merge する前に開いて、変更の実際の挙動を確認してください。」と案内する
- `{QUIZ_OUTPUT}` が `file` の場合: `understanding-quiz-impl.html` のファイルパスを提示し、「実装後の理解度クイズを生成しました。push / merge する前に、ブラウザで開いて変更の実際の挙動を確認してください（例: `open .plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-impl.html`）。」と案内する
- `{QUIZ_OUTPUT}` が `interactive` の場合: 7.5-3b の結果（スコア・合否・読み直し箇所）を提示済みのため、ファイル・URL の案内は不要
- 「これは advisory ゲートです。落ちた設問がある＝変更を理解できていない箇所なので、その箇所の diff と implementation-notes を読み直してから push することをおすすめします。」

## Step 8: 完了報告

実装完了後、ユーザーに以下を報告する：

1. 実装したタスクの一覧
2. 変更したファイルの一覧
3. 関連Issue番号（あれば）
4. AIレビューの結果サマリー（レビュー実行時）
5. PLANNINGファイルの削除状態
6. DoD充足状況（DoDがある場合）
7. 実装後理解度クイズ（Artifact の URL または `understanding-quiz-impl.html` のパス）と、push 前に確認するよう案内

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

- 実装開始前のユーザー確認は Step 3.7 を参照（確認なしにコード変更を開始しない）
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
