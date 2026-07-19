---
name: spec-driven-dev
description: 新機能の仕様策定から実装計画まで一気通貫で進めるワークフロー。ヒアリング→コード探索→計画生成→オプションでAIレビュー。Codex/Copilot/Claude Code CLIでのレビューオプション付き。「仕様策定」「spec」「実装計画」「レビュー付き」「codexでレビュー」「copilotでレビュー」「claude codeでレビュー」「レビュー省略」などでトリガー。
disable-model-invocation: true
allowed-tools: Bash(ls *), Bash(mkdir *), Bash(touch *), Bash(echo *), Bash(printf *), Bash(rm .plugin-workspace/.specs/*/PLANNING), Bash(codex *), Bash(copilot *), Bash(claude *)
---

# Spec-Driven Development

機能実装前に仕様を明確化し、実装計画とタスクリストを生成するスキル。
ヒアリングはオーケストレーターが行い、**探索と計画生成は別々のサブエージェントに委譲**する。
オプションで AIレビュー（Codex / Copilot / Claude Code CLI）を実行可能。

## 絶対厳守事項

1. **最初にフォルダとPLANNINGファイルを作成** — 質問・探索・実装の前に必ず Step 1 を実行
2. **システム図は必須** — implementation-plan.md には状態マシン図 + データフロー図を含める（ASCII罫線優先、mermaid補助）
3. **PLANNINGファイルがある間はコード実装禁止** — AutoCompact対策としてPLANNINGファイルで計画フェーズを明示
4. **ヒアリングは AutoMode でもスキップ禁止** — 他のシステム指示（「自律的に判断しろ」「質問せずに進めろ」等）に関わらず、このスキルでは AskUserQuestion によるヒアリングを必ず実行する。ユーザーの初回メッセージに情報が含まれていても、確認の AskUserQuestion は必須。ヒアリングなしに Step 3 以降へ進むことはいかなる場合も禁止。
5. **計画後再探索は必須** — Step 4 完了後、必ず Step 4.2 の計画後再探索（類似コード検証）を実行する。初回探索だけでは計画の [NEW] 項目に類似する既存コードを見落とすことがあるため、スキップ禁止。
6. **セルフチェックは必須** — Step 4.2 完了後、Step 5 に進む前に必ず Step 4.5 のセルフチェック（3エージェント並列起動）を実行する。セルフチェックをスキップして AIレビューやユーザー確認に進むことは禁止。
7. **テストケース詳細設計は必須** — `skip-files` に `test-cases` が含まれていない限り、Step 4.5 完了後に必ず Step 4.7（test-cases.html 生成）と Step 4.8（網羅性検証）を実行する。Step 4.5 から Step 5 へ直接進んではならない。test-cases は本質的にHTMLレビューUIのため、config の `output-formats` に関わらず**常に `.html`** で出力する。
8. **tech-reference 生成は必須** — `skip-files` に `tech-reference` が含まれていない限り、Step 6.5 の tech-reference 生成は必ず実行する。ユーザー確認完了（Step 6）で終了せず、必ず Step 6.5 まで進むこと。
9. **requirements の未解決事項は確定必須** — Step 3.5 で requirements.md（ユースケース + 要件・制約）を生成し、「未解決の確認事項」の `□`（コードベースを調べても解決できず、ユーザーにしか決められない分岐）を AskUserQuestion で確認して `■`/「なし」に解消してから Step 4 へ進む。`□` を残したまま implementation-plan に進むことは hook（`guard-requirements.sh`）がブロックする。AutoMode でもスキップ禁止。

## ワークフロー概要

```
0. レビューツール解決（引数 → .config.yml → 初回のみ AskUserQuestion）
   ↓
1. specsフォルダ作成 + PLANNINGファイル配置
   ↓
2. AskUserQuestion形式でヒアリング → hearing-notes.md 書き出し
   ↓
2.5. 【GATE】hearing-notes 品質検証 → 不合格なら再ヒアリング
   ↓
3. codebase-explorer サブエージェント → exploration-report.md
   ↓
3.5. requirements 確定 → requirements.md（ユースケース + 要件・制約 + 未解決の確認事項）
     └ 未解決の確認事項（□）は AskUserQuestion で確認して解消（残ると hook がブロック）
   ↓
4. spec-planner サブエージェント → implementation-plan.md + tasks.md
   ↓
4.2. 計画後再探索（類似コード検証）→ 見落としがあれば計画修正
   ↓
4.5. セルフチェック（計画品質ゲート）→ コード例・設計妥当性・テストパターン検証
   ↓
4.7. テストケース詳細設計 → test-cases.html（テスト網羅性レビュー用、常に .html）
   ↓
4.8. テストケース網羅性検証（test-pattern-checker 詳細モード）
   ↓
5. AIレビュー（オプション）→ 修正ループ
   ↓
6. ユーザーに提示
   ↓
6.5. tech-reference 生成（サブエージェント）→ tech-reference.md
   ↓
6.7. 実装前理解度クイズ生成 → understanding-quiz-plan.html / Artifact / 対話出題（設計意図の自己確認・advisory）
   ↓
7. 実装開始許可後、PLANNINGファイル削除
```

## バリアントパラメータ

| パラメータ | 値 |
|-----------|-----|
| SKILL_NAME | `spec-driven-dev` |
| PLANNING_CONTENT | `${CLAUDE_SESSION_ID}` |
| USE_GUARD | `true` |

## Step 0: 設定解決（レビューツール + 出力形式）

### 0-1. レビューツール解決

使用するレビューツールを以下の優先順で決定する:

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

### 0-2. 出力形式解決

各ファイルの出力形式（`.md` / `.html`）を `.plugin-workspace/.specs/.config.yml` の `output-formats` セクションから読み取る。

```yaml
output-formats:
  hearing-notes: md        # → .md
  exploration-report: html # → .html
  implementation-plan: md  # → .md
  tasks: md                # → .md
  tech-reference: html     # → .html
```

- **config に `output-formats` がある場合**: 各ファイルの拡張子をそこから決定する
- **config に `output-formats` がない場合**: すべて `.md` をデフォルトとする
- **個別キーが欠落している場合**: そのファイルは `.md` をデフォルトとする

以降のステップでは、ファイルごとの拡張子を以下の変数で参照する:
- `{HEARING_NOTES_EXT}` — hearing-notes の拡張子
- `{EXPLORATION_REPORT_EXT}` — exploration-report の拡張子
- `{IMPLEMENTATION_PLAN_EXT}` — implementation-plan の拡張子
- `{TASKS_EXT}` — tasks の拡張子
- `{TECH_REFERENCE_EXT}` — tech-reference の拡張子

**HTML出力時のルール**: `.html` が指定されたファイルを生成する際は:
1. `assets/templates/style.css` を Read してCSSを取得
2. 対応する HTML テンプレート（`assets/templates/{ファイル名}.html`）を Read
3. テンプレートの `<link rel="stylesheet" href="style.css">` を `<style>{CSS内容}</style>` に置換
4. プレースホルダを内容で埋めて自己完結型HTMLとして出力

## Step 1: specsフォルダ + PLANNINGファイル作成

ヒアリング開始前に、specディレクトリ・PLANNINGファイル・ガードファイルを作成する。

**詳細手順は [references/workflow-steps.md](references/workflow-steps.md) の Step 1 を参照。**

## Step 2: ヒアリング → hearing-notes 書き出し

AskUserQuestion で Batch 1-3（スコープ → 技術詳細 → 品質要件）を聴取し、hearing-notes{HEARING_NOTES_EXT} に書き出す。

**詳細手順は [references/workflow-steps.md](references/workflow-steps.md) の Step 2 を参照。**
質問形式の詳細は `references/question-patterns.md` を参照。

## Step 2.5: Reflective Gate（ヒアリング品質検証）

hearing-notes.md 書き出し後、Step 3 に進む前に品質を検証する。不合格の場合は回復フローに入る。

**詳細手順は [references/workflow-steps.md](references/workflow-steps.md) の Step 2.5 を参照。**

## Step 3: コードベース探索

hearing-notes.md から探索ヒント（キーワード5-10個、推定対象パス、探索の重点）を抽出し、codebase-explorer サブエージェントを起動する。

**品質基準**: Readファイル数 ≥ 10、コードスニペット数 ≥ 5、逆引き検索「実施済み」。未達なら補完探索を最大1回実行。

**プロンプトテンプレートと品質検証の詳細は [references/workflow-steps.md](references/workflow-steps.md) の Step 3 を参照。**

探索の5カテゴリ詳細は `references/exploration-perspectives.md` を参照。

## Step 3.5: requirements 確定【必須】

exploration-report.md と hearing-notes.md を統合し、ユースケースと要件・制約を確定した **requirements.md** を**オーケストレーター自身**が生成する（サブエージェントには委譲しない — ユーザーと対話して確定するため）。requirements は hook のパース対象のため、config の `output-formats` に関わらず**常に `.md`** で出力する。

1. テンプレート `assets/templates/requirements.md` を埋め、ユースケース（誰が・どの状況で・何を達成したいか）と機能/非機能要件・制約・設計方針を記述する
2. 「コードベースを調べても解決できず、ユーザーにしか決められない分岐」を **未解決の確認事項** に行頭 `□` で列挙する
3. `□` 項目があれば AskUserQuestion（上限4件/ターン）で確認し、回答を反映して `□` → `■`、または項目を消して「なし」にする
4. 未解決の確認事項が解消（`□` ゼロ）したら Step 4 へ進む

> `□` が残ったまま implementation-plan を書こうとすると `guard-requirements.sh` がブロックする。AutoMode でも `□` の解消を AskUserQuestion でスキップしてはならない。

**詳細は [references/workflow-steps.md](references/workflow-steps.md) の Step 3.5 を参照。**

## Step 4: 実装計画生成

spec-planner サブエージェントを起動し、implementation-plan{IMPLEMENTATION_PLAN_EXT} と tasks{TASKS_EXT} を生成する。**hearing-notes・exploration-report に加え requirements.md（ユースケース・要件・制約）も入力として渡し**、計画が確定済みのユースケースを満たすことを前提条件とする。

**プロンプトテンプレートは [references/workflow-steps.md](references/workflow-steps.md) の Step 4 を参照。**

## Step 4.2: 計画後再探索（類似コード検証）【必須】

計画ができて初めて「何を新規に作ろうとしているか」が具体化するため、その内容を検索キーとして 2 回目の探索を行い、初回探索の見落としを検証する。

1. implementation-plan の変更案から **[NEW] 項目**（新規ファイル・関数・コンポーネント名と責務）を抽出する
2. codebase-explorer を再起動し、各 [NEW] 項目について「類似の既存実装・再利用可能なコードが本当にないか」を逆引きで重点検索する
3. **類似コード発見** → exploration-report を更新し、発見一覧を添えて spec-planner を再起動して計画を修正（既存を再利用する、または再利用しない理由を明記。最大1回）
4. **発見なし** → Step 4.5 へ進む

**プロンプトテンプレートは [references/workflow-steps.md](references/workflow-steps.md) の Step 4.2 を参照。**

## Step 4.5: セルフチェック（3エージェント並列 → オーケストレーター修正）【必須】

> **このステップは必ず実行すること。Step 4 完了後に Step 5 や Step 6 へ直接進んではならない。**

以下の3つのエージェントを Agent tool で **並列起動** する。`subagent_type` に各エージェント名を指定すること。エージェントは評価のみ行い、修正はオーケストレーターが実施。

1. `subagent_type: "plan-format-checker"` — テンプレート構造との適合性（セクション構成・コードブロック形式・テスト構成・プレースホルダ残留）
2. `subagent_type: "design-validity-checker"` — コンポーネント分割・データフロー・依存方向・アーキテクチャ整合性の設計レビュー
3. `subagent_type: "test-pattern-checker"` — テストパターンの網羅性（ファイル構成・シナリオ充足・具体性）の評価

**プロンプトテンプレートと結果処理は [references/workflow-steps.md](references/workflow-steps.md) の Step 4.5 を参照。**

FAIL があればオーケストレーターが修正（最大2回）。WARN はユーザーに提示。

## Step 4.7: テストケース詳細設計【必須】

> **このステップは必ず実行すること（`skip-files` に `test-cases` が含まれる場合のみスキップ可）。Step 4.5 完了後、Step 5 へ直接進んではならない。**

`test-case-designer` サブエージェントを起動し、テスト専用の詳細ドキュメント **test-cases.html** を生成する。implementation-plan の検証計画セクションは**そのまま残し**（要約・戦略レベル）、その詳細版として test-cases.html を作る。

test-cases.html は**マスター詳細型のレビューUI**で、CSS・ヘルパー・レンダラがテンプレートに固定済み。エージェントは先頭の **DATA スクリプト（`FILES` / `PLAN`）だけ**を埋める（HTMLは書かない）。**test-cases は本質的にHTMLレビューUIで .md 相当が無いため、config の `output-formats` に関わらず常に `.html` で出力する。他のファイルと違い `<link>`→style.css 置換は不要**（自己完結済み）。各ケースに ID・優先度・カテゴリ・入力/期待結果の具体値・カバレッジを付与し、`gaps` と `PLAN.trace` で「抜け」を可視化する。目的は **実装前にテストの網羅性を人間がレビューできるゲート**。

```
subagent_type: "test-case-designer"
```

**プロンプトテンプレートは [references/workflow-steps.md](references/workflow-steps.md) の Step 4.7 を参照。**

## Step 4.8: テストケース網羅性検証【必須】

> **Step 4.7 で test-cases.html を生成した場合、必ず実行すること。**

`test-pattern-checker` サブエージェントを**詳細モード**で起動し、test-cases.html の網羅性・具体性を検証する。Step 4.5 の test-pattern-checker（計画のテスト要約を検証）とは対象・観点が異なる。

```
subagent_type: "test-pattern-checker"   # 検証対象に test-cases.html を指定（詳細モード）
```

検証対象は test-cases.html 先頭の DATA スクリプト（`FILES` / `PLAN`）。評価は D1〜D9（データ妥当性・ケースID・カテゴリ妥当性/網羅・優先度・具体性・カバレッジ整合・シナリオ充足・gaps の正直性・要件トレーサビリティ）。FAIL があればオーケストレーターが DATA スクリプトを修正（最大2回、CSS・レンダラは触らない）。

**プロンプトテンプレートと結果処理は [references/workflow-steps.md](references/workflow-steps.md) の Step 4.8 を参照。**

## Step 5: AIレビュー（オプション）

Step 0 で決定した `{REVIEW_TOOL}` を使用する。`none` の場合は Step 6 へスキップ。

ツール選択済みの場合:
1. `plan-review/prompt-{NNN}.txt` を生成
2. [references/review-tools.md](references/review-tools.md) のコマンド構文に従い実行
3. `plan-review/review-{NNN}.md` に出力を保存
4. レビュー結果を解析し、問題があれば修正 → 再レビュー（最大5回）
5. レビュー結果を要約してユーザーに提示

レビュー観点の詳細は `references/review-criteria.md` を参照。

## Step 6: ユーザー確認

生成したファイルをユーザーに提示:

1. **specフォルダパス**: `.plugin-workspace/.specs/{nnn}-{feature-name}/` を明示
2. 生成ファイル一覧（hearing-notes, exploration-report, implementation-plan, tasks — 各ファイルの拡張子は config に従う。`test-cases.html` は常に .html・テスト網羅性レビュー用）
3. implementation-plan.md の内容サマリー
4. tasks.md のタスク一覧
5. **テスト網羅性のレビューには `test-cases.html` をブラウザで開くよう案内する**（例: `open .plugin-workspace/.specs/{nnn}-{feature-name}/test-cases.html`）
6. 「修正が必要な場合はお知らせください」

ユーザーが修正を要求した場合は、フィードバックの明確性を確認する（[references/feedback-clarification.md](references/feedback-clarification.md) 参照）。
曖昧な場合は AskUserQuestion で具体化してから Step 4 に戻る。明確な場合はそのまま Step 4 に戻る。

## Step 6.5: 技術リファレンス生成【必須】

> **このステップは `skip-files` に `tech-reference` が含まれていない限り必ず実行すること。Step 6 のユーザー確認完了でワークフローを終了してはならない。**

`.plugin-workspace/.specs/.config.yml` の `skip-files` に `tech-reference` が含まれている場合のみスキップ可。

ユーザー確認完了後、サブエージェントを起動して tech-reference{TECH_REFERENCE_EXT} を生成する。

implementation-plan に登場するすべての技術（言語・フレームワーク・ライブラリ・ツール・概念）を
初学者向けに解説するドキュメントを生成する。
読者は、言語やライブラリ、作ろうとしているものの初心者であることを前提とする。

**詳細は [references/workflow-steps.md](references/workflow-steps.md) の tech-reference 生成を参照。**

## Step 6.7: 実装前理解度クイズ生成

plan を実装セッションに渡す直前に、**実装者（人間）の設計理解度を測るクイズ**を HTML アーティファクトとして生成する。落ちた設問は「設計がまだ自分の中で固まっていない箇所」を指すため、実装に渡す前に潰すための **advisory ゲート**。

**生成の有無は `.plugin-workspace/.specs/.config.yml` の `skip-files` で決まる**（`/spec-setup` で設定）。`skip-files` に `understanding-quiz` が含まれている場合はこのステップをスキップする。含まれていなければ生成する。

**出力先は `.config.yml` の `quiz-output.plan` で決まる**: `file`（specフォルダ内の HTML ファイル、**未設定時のデフォルト**）、`artifact`（Artifact ツールで公開）、または `interactive`（HTML を生成せず、セッション内で AskUserQuestion により1問ずつ対話出題）。

- **材料**: requirements.md（ユースケース・要件・制約）＋ implementation-plan＋ hearing-notes（どの unknowns をどう解決したかの記録）
- **問う対象**: なぜこの設計にしたか / このデータモデル・型にした理由 / この制約が壊れると何が起きるか / 変わりやすい箇所の意図
- **出力**: 解説＋クイズ（4択・YES/NO・並べ替え。回答した時点で1問ずつ正誤と解説を表示する即時フィードバック方式。初回回答でロックされ答えは変更不可。test-cases.html と同様に自己完結HTMLで、`<link>`→style.css 置換は不要）。`quiz-output.plan` に応じて `understanding-quiz-plan.html` ファイル（デフォルト）/ Artifact 公開 / セッション内対話出題（`interactive`。HTML を生成せず AskUserQuestion で1問ずつ）
- **enforcement**: advisory。hook による機械的ブロックはしない（合否は自己申告のため）。

出題設計の指針は [references/quiz-design.md](references/quiz-design.md) を参照。作成した設問は出題前に同ファイルの「設問セルフレビュー」で検査し、基準を満たさない設問は捨てる。

**詳細手順は [references/workflow-steps.md](references/workflow-steps.md) の Step 6.7 を参照。**

## Step 7: 実装開始（ユーザーによるガード解除）

**詳細は [references/workflow-steps.md](references/workflow-steps.md) のガード解除を参照。**

```
ユーザーへの案内:
  実装を開始するには、以下のコマンドを実行してください:
  rm .plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID} .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
```

## 出力ディレクトリ

```
.plugin-workspace/.specs/
└── {nnn}-{feature-name}/
    ├── PLANNING                 # 計画中は存在、実装開始時に削除
    ├── hearing-notes{EXT}       # ヒアリング結果（オーケストレーター生成）
    ├── exploration-report{EXT}  # 探索レポート（codebase-explorer 生成）
    ├── requirements.md          # ユースケース + 要件・制約（オーケストレーター生成、hook パース対象ゆえ常に .md）
    ├── implementation-plan{EXT} # 実装計画（spec-planner 生成）
    ├── tasks{EXT}               # タスクリスト（spec-planner 生成）
    ├── test-cases.html          # テストケース詳細仕様（網羅性レビュー用、HTML・常に .html）
    ├── tech-reference{EXT}      # 技術リファレンス（初学者向け、サブエージェント生成）
    ├── understanding-quiz-plan.html # 実装前理解度クイズ（設計意図の自己確認、HTML・常に .html。quiz-output.plan: artifact の場合は Artifact 公開、interactive の場合はファイルを作らず対話出題）
    └── plan-review/             # AIレビュー結果（レビュー実行時のみ）
        ├── prompt-001.txt
        ├── review-001.md
        └── ...
```

`{nnn}` は `.plugin-workspace/.specs/` 内の既存フォルダ数に基づく3桁の連番（001, 002, 003...）
`{feature-name}` はケバブケースで命名（例: `001-user-authentication`, `002-block-button`）
