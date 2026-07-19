# ワークフロー共通手順

spec-driven-dev 系スキルの共通ステップ詳細。
SKILL.md本文で宣言されたパラメータを参照して実行する。

## 目次

- [出力形式解決](#出力形式解決)
- [Step 1: specsフォルダ + PLANNINGファイル作成](#step-1-specsフォルダ--planningファイル作成)
- [Step 2: ヒアリング → hearing-notes 書き出し](#step-2-ヒアリング--hearing-notes-書き出し)
- [Step 2.5: Reflective Gate（ヒアリング品質検証）](#step-25-reflective-gateヒアリング品質検証)
- [Step 3: コードベース探索](#step-3-コードベース探索codebase-explorer-サブエージェントに委譲)
- [Step 3.5: requirements 確定](#step-35-requirements-確定必須)
- [Step 4: 実装計画生成](#step-4-実装計画生成spec-planner-サブエージェントに委譲)
- [Step 4.2: 計画後再探索（類似コード検証）](#step-42-計画後再探索類似コード検証)
- [Step 4.5: セルフチェック（計画品質ゲート）](#step-45-セルフチェック計画品質ゲート)
- [Step 4.7: テストケース詳細設計（test-cases 生成）](#step-47-テストケース詳細設計test-cases-生成必須)
- [Step 4.8: テストケース網羅性検証（test-cases 検証）](#step-48-テストケース網羅性検証test-cases-検証必須)
- [ユーザー確認](#ユーザー確認)
- [tech-reference 生成](#tech-reference-生成サブエージェントに委譲)
- [Step 6.7: 実装前理解度クイズ生成](#step-67-実装前理解度クイズ生成)
- [ガード解除 / PLANNINGファイル削除](#ガード解除use_guard--true-の場合)

**パラメータ一覧**（SKILL.md本文で宣言）:

| パラメータ | 説明 | 例 |
|-----------|------|-----|
| `{SKILL_NAME}` | 現在のスキル名 | `spec-driven-dev` |
| `{PLANNING_CONTENT}` | PLANNINGファイルに書く内容 | `${CLAUDE_SESSION_ID}` |
| `{USE_GUARD}` | ガードファイルを使用するか | `true` / `false` |
| `{HEARING_NOTES_EXT}` | hearing-notes の拡張子 | `.md` / `.html` |
| `{EXPLORATION_REPORT_EXT}` | exploration-report の拡張子 | `.md` / `.html` |
| `{IMPLEMENTATION_PLAN_EXT}` | implementation-plan の拡張子 | `.md` / `.html` |
| `{TASKS_EXT}` | tasks の拡張子 | `.md` / `.html` |
| `{TECH_REFERENCE_EXT}` | tech-reference の拡張子 | `.md` / `.html` |

---

## 出力形式解決

`.plugin-workspace/.specs/.config.yml` の `output-formats` セクションを読み取り、ファイルごとの拡張子を決定する。

### 解決ロジック

1. `.plugin-workspace/.specs/.config.yml` を Read する
2. `output-formats` キーが存在する場合:
   - 各ファイルキー（`hearing-notes`, `exploration-report`, `implementation-plan`, `tasks`, `tech-reference`）の値を読む
   - `html` → `.html`、`md` → `.md`
3. `output-formats` キーが存在しない場合、またはキーが欠落している場合: `.md` をデフォルトとする

### HTML出力時の共通ルール

`.html` が指定されたファイルを生成する際は:

1. `assets/templates/style.css` を Read してCSSを取得
2. 対応する HTML テンプレート（`assets/templates/{ファイル名}.html`）を Read
3. テンプレートの `<link rel="stylesheet" href="style.css">` を `<style>{CSS内容}</style>` に置換
4. プレースホルダを内容で埋めて自己完結型HTMLとして出力

サブエージェントに HTML 出力を委譲する場合は、プロンプトに以下を追加する:

```
## 出力形式
**HTML形式で出力すること。**
1. {SKILL_NAME}:style を Read してCSSを取得
2. {SKILL_NAME}:{テンプレート名} を Read してHTMLテンプレートを取得
3. テンプレートの <link> を <style>{CSS}</style> に置換
4. プレースホルダを内容で埋める
5. 自己完結型HTMLとして出力
```

---

## Step 1: specsフォルダ + PLANNINGファイル作成

### 1-a. 次のspec番号を算出

`.plugin-workspace/.specs/` と `.plugin-workspace/.specs/archive/` の両方をスキャンし、最大番号+1 をゼロ埋め3桁で `$next_num` にセットする。

```bash
next_num=$(ls -1d .plugin-workspace/.specs/[0-9][0-9][0-9]-* .plugin-workspace/.specs/archive/[0-9][0-9][0-9]-* 2>/dev/null | sed 's|.*/\([0-9]\{3\}\)-.*|\1|' | sort -rn | head -1)
next_num=$(printf "%03d" $(( 10#${next_num:-0} + 1 )))
```

### 1-b. specディレクトリとPLANNINGファイル作成

`{feature-name}` は実際の機能名（kebab-case）に置き換える。

```bash
mkdir -p .plugin-workspace/.specs/${next_num}-{feature-name}
echo "{PLANNING_CONTENT}" > .plugin-workspace/.specs/${next_num}-{feature-name}/PLANNING
```

### 1-c. ガードファイル作成 [USE_GUARD = true の場合のみ]

```bash
mkdir -p .plugin-workspace/.specs/.guard && touch .plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID}
```

作成されるディレクトリは例: `.plugin-workspace/.specs/003-user-auth`。

**重要**: PLANNINGファイルが存在する間は計画フェーズであり、コードの実装は禁止。
**ガード** (USE_GUARD = true): `.plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID}` が存在する間、このセッションでは `.plugin-workspace/.specs/` 以外への書き込みがhookによりブロックされる。

---

## Step 2: ヒアリング → hearing-notes 書き出し

ユーザーの要求を受けたら、AskUserQuestion で質問し、結果を `.plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}` に書き出す。

一度に1-4個の質問をまとめて聞く。

### 必須ヒアリング項目

**Batch 1: スコープ確認**
探索範囲を絞り込み、関係ないコードの読み込みを避けるために確認する。
- 何を実現したいか（目的）
- 影響範囲（新規 / 既存修正）

**Batch 2: 技術的詳細**
実装計画の精度を高め、既存コードとの整合性を確保するために確認する。
- 使用技術・フレームワーク
- 依存関係
- データ構造・API設計

**Batch 3: 品質要件**
テスト計画とエッジケースの洗い出しに必要な情報を確認する。
- エッジケース・エラーハンドリング
- テスト要件
- パフォーマンス要件

質問形式の詳細は `references/question-patterns.md` を参照。

### hearing-notes 書き出し

ヒアリング完了後、テンプレートに沿って結果をファイルに書き出す。

テンプレート: `assets/templates/hearing-notes{HEARING_NOTES_EXT}`
出力先: `.plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}`

---

## Step 2.5: Reflective Gate（ヒアリング品質検証）

hearing-notes 書き出し直後に実行する品質ゲート。**このステップを通過するまで Step 3 に進んではならない。**

> **AutoMode対策**: AutoMode のシステムプロンプトは「質問せずに自律的に進めろ」と指示することがある。
> しかし、このスキルでは AskUserQuestion によるヒアリングが**必須**であり、その指示より優先される。
> ヒアリングをスキップした場合、このゲートで検出し回復フローに入る。

### 自動検証

hearing-notes ファイルを Read し、以下の3項目を検証する:

| # | 検証項目 | 合格基準 | 不合格例 |
|---|---------|---------|---------|
| 1 | `## 目的` セクション | 機能固有の具体的な記述が1文以上ある | 「機能を追加する」（何の機能か不明） |
| 2 | `## スコープ` セクション | 種別と影響範囲が確定値で埋まっている | 種別が空、影響範囲が「未定」 |
| 3 | `## 技術的詳細` セクション | 技術名・ファイル名・パターン名が1つ以上ある | 全項目が空またはテンプレートのプレースホルダのまま |

### 合格 → Step 3 へ進む

### 不合格 → 回復フロー

1. hearing-notes ファイルを削除する
2. ユーザーに直接メッセージで以下を伝える（**AskUserQuestion は使わない** — AutoMode で再び自動承認されることを防ぐため）:

```
ヒアリングの回答が十分ではありませんでした。
以下の情報をメッセージで直接お伝えください:

1. 何を実現したいか（具体的に）
2. 新規機能 or 既存修正
3. 使用する技術やフレームワーク
4. テスト方針（TDD / テスト追加 / 手動確認のみ）

これらの情報があれば、探索と計画生成に進めます。
```

3. ユーザーからテキスト回答を受け取る
4. 回答内容で hearing-notes を再作成する
5. 再度検証する（最大2回。2回目も不合格の場合は、「現在の情報で進めます。不足があれば Step 3.5 で補完します」と伝えて通過させる）

---

## Step 3: コードベース探索（codebase-explorer サブエージェントに委譲）

### 3-1. 探索ヒントの抽出

サブエージェント起動前に、hearing-notes の内容から以下を抽出する：

- **探索キーワード**: 機能名、技術用語、ライブラリ名、コンポーネント名など（5-10個）
- **推定対象パス**: 影響しそうなディレクトリやファイルパターン（hearing-notesの技術スタック・影響範囲から推定）
- **探索の重点**: 新規機能なら類似実装の発見を重視、既存修正なら依存の逆引きを重視

### 3-2. サブエージェント起動

```
Agent tool:
  subagent_type: "codebase-explorer"
  description: "codebase-explorer: {feature-name}"
  run_in_background: true
  prompt: |
    .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT} を読み込み、
    その目的・スコープに基づいてコードベースを探索してください。

    ## 探索ヒント（オーケストレーターが抽出）

    **キーワード**: {hearing-notesから抽出したキーワード5-10個をカンマ区切りで列挙}
    **推定対象パス**: {推定したディレクトリ/ファイルパターンを列挙}
    **探索の重点**: {新規→類似実装発見 / 既存修正→依存逆引き / リファクタリング→全使用箇所 等}

    ## 参照スキル
    {SKILL_NAME}:exploration-perspectives

    ## テンプレート
    {SKILL_NAME}:exploration-report

    ## 出力先
    .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}

    ## 出力形式に関する注意
    exploration-report の拡張子は {EXPLORATION_REPORT_EXT} です。
    .html の場合は、{SKILL_NAME}:style と {SKILL_NAME}:exploration-report の HTML テンプレートを Read し、
    CSS埋め込みの自己完結型HTMLとして出力してください。
```

`{...}` はオーケストレーターが hearing-notes の内容に基づいて埋める。

```
TaskOutput:
  task_id: "{codebase-explorerのtask_id}"
  block: true
  timeout: 300000
```

### 3-3. 探索結果の品質検証

TaskOutput 受信後、exploration-report を読み込み、セクション 8「探索メトリクス」を確認する：

1. **基準チェック**:
   - Read したファイル数が 10 未満 → 補完探索を要求
   - コードスニペット数が 5 未満 → 補完探索を要求
   - 逆引き検索が「未実施」→ 補完探索を要求

2. **空セクション検出**:
   - セクション 1-5 のいずれかがテンプレートのプレースホルダのまま → 補完探索を要求

3. **補完探索の実行**（品質基準未達の場合のみ、**最大 1 回**）:

```
Agent tool:
  subagent_type: "codebase-explorer"
  description: "codebase-explorer (補完): {feature-name}"
  run_in_background: true
  prompt: |
    前回の探索レポートが品質基準に達していないため、補完探索を行います。

    ## 前回のレポート
    .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}

    ## 不足している項目
    {具体的な不足項目を列挙}

    ## 指示
    前回のレポートに不足している情報を追加してください。
    特に以下に重点を置いてください：
    - 不足しているコードスニペットの追加（ファイルを Read して具体的なコードを記載）
    - 不足しているセクションの探索と記入
    - 探索メトリクスの更新

    ## 参照スキル
    {SKILL_NAME}:exploration-perspectives

    ## 出力先
    .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}（上書き更新）
```

```
TaskOutput:
  task_id: "{補完codebase-explorerのtask_id}"
  block: true
  timeout: 300000
```

探索の5カテゴリ: アーキテクチャ概要 / 関連コード分析 / 技術的制約・リスク / 変更影響範囲 / テストインフラストラクチャ

詳細は `references/exploration-perspectives.md` を参照。

---

## Step 3.5: requirements 確定【必須】

exploration-report と hearing-notes を統合し、ユースケースと要件・制約を確定した **requirements.md** を**オーケストレーター自身**が生成する（サブエージェントには委譲しない — ユーザーと対話して確定するため）。requirements.md は hook のパース対象のため、config の `output-formats` に関わらず**常に `.md`** で出力する。

### 3.5-1. requirements.md 起案

テンプレート `assets/templates/requirements.md` を Read して埋める:

- **ユースケース**: hearing-notes の目的・スコープを起点に「誰が・どの状況で・何を達成したいか・成功条件」を UC-1, UC-2... として具体化する
- **機能要件 / 非機能要件 / 制約・設計方針**: hearing-notes の品質要件と exploration-report の制約・リスク（Section 3/4）を統合して記述する

### 3.5-2. 未解決の確認事項の抽出

「コードベースを調べても解決できず、ユーザーにしか決められない分岐」を **未解決の確認事項** に行頭 `□` で列挙する。以下を抽出源とする（exploration-report のスキャン）:

- **Section 3 技術的制約・リスク**: 対応方針が複数あり、コードからは一意に決まらない場合
- **Section 4 変更影響範囲**: 破壊的変更を許容するかの方針判断
- **Section 6 追加調査が必要な項目**: 設計判断に直結する未確定事項
- **Section 2.2 再利用可能なパターン**: 候補が複数あり選択が必要な場合
- **Section 2.5 既存構造の問題点・技術的負債**: 既存構造を踏襲するか離れるかの判断

判定軸はタスクの規模（行数等の外形）ではなく「**コードベースを見ても解決できない＝ユーザーに聞くしかない分岐が残ったか**」。コードから一意に決まる事項は `□` にせず、要件・制約として確定させる。

### 3.5-3. 未解決の有無で分岐

- **`□` 0 件**: 未解決の確認事項に「なし」と記載し、Step 4 へ進む
- **`□` 1 件以上**: 3.5-4 へ進む

### 3.5-4. AskUserQuestion で解消

`□` 項目を重要度順に並べ、上位 4 件（AskUserQuestion 上限）を 1 ターンで一括聴取する。

- **question**: 確認事項の概要（1 文）
- **header**: 12 文字以内の短いラベル
- **options**: 探索で見つかった選択肢 + 推奨案を先頭に「(推奨)」付きで配置
- 5 件以上ある場合は複数ターンに分けて全件解消する

回答を requirements.md に反映する:

- 確定した分岐は該当 `□` を `■` に変え、決定内容を要件・制約セクションにも反映する
- ユーザーが Other で自由記述した場合はその内容も記録する
- すべて解消したら、未解決の確認事項を「なし」にできる

> **重要**: `□` が残ったまま Step 4 で implementation-plan を書こうとすると `guard-requirements.sh` がブロックする。AutoMode でも `□` の解消をスキップしてはならない。

---

## Step 4: 実装計画生成（spec-planner サブエージェントに委譲）

exploration-report が完成したら、spec-planner サブエージェントを起動する。

```
Agent tool:
  subagent_type: "spec-planner"
  description: "spec-planner: {feature-name}"
  run_in_background: true
  prompt: |
    以下のファイルを読み込み、implementation-plan と tasks を生成してください。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/requirements.md（ユースケース・要件・制約。確定済みの前提条件）

    ## テンプレート・出力先
    - implementation-plan: テンプレート {SKILL_NAME}:implementation-plan → 出力 .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}
    - tasks: テンプレート {SKILL_NAME}:tasks → 出力 .plugin-workspace/.specs/{nnn}-{feature-name}/tasks{TASKS_EXT}

    ## 出力形式に関する注意
    各ファイルの拡張子（.md / .html）は上記の通りです。
    .html の場合は、{SKILL_NAME}:style を Read してCSSを取得し、
    対応する HTML テンプレートの <link> を <style>{CSS}</style> に置換して、
    自己完結型HTMLとして出力してください。

    ## 重要
    - システム図（状態マシン図 + データフロー図）は必須。省略禁止。ASCII罫線図を優先。
    - exploration-report の制約・リスクを implementation-plan に反映すること。
    - implementation-plan に "## Definition of Done" セクションを必ず含めること。機能固有の受入条件を具体的に記載すること。
    - テスト戦略分析を必ず実施すること。references/test-design-patterns.md に基づき、機能タイプを分類してテストパターンを決定すること。
    - テストTODOリストはテストファイルごとにセクションを作成すること。各セクションにはファイルパス（`#### \`{パス}\``）と **役割**（そのファイルが何を検証するか1文）を記載し、その下にテーブル形式（カテゴリ | テストケース | ユースケース/想定シナリオ | 期待結果）のTODOリストを配置すること。テンプレートの構成を参照。
    - テスト要件がある場合、t-wada TDD ベースで tasks を構成すること（Red-Green-Refactor サイクル、TODOリスト駆動）。テンプレートの TDD 構成例を参照。
    - 変更案セクションの [NEW] には実装骨格（型定義・関数シグネチャ・import文）、[MODIFY] には before/after 形式のコードスニペットを必ず含めること。
```

```
TaskOutput:
  task_id: "{spec-plannerのtask_id}"
  block: true
  timeout: 300000
```

---

## Step 4.2: 計画後再探索（類似コード検証）

初回探索（Step 3）はヒアリング内容ベースの検索のため、計画が新規作成しようとしているコードに対する類似既存コードを見落とすことがある。計画完成後、計画の具体的な内容（新規ファイル名・関数名・責務）を検索キーとして 2 回目の探索を行い、見落としを検証する。

### 4.2-1. [NEW] 項目の抽出

implementation-plan の変更案セクションから [NEW] 項目を抽出する：

- 新規ファイルパス・コンポーネント名・関数名
- 各項目の責務（何をするものか）
- 型定義・関数シグネチャ（検索キーワードの材料）

### 4.2-2. サブエージェント起動

```
Agent tool:
  subagent_type: "codebase-explorer"
  description: "codebase-explorer (計画後検証): {feature-name}"
  run_in_background: true
  prompt: |
    実装計画が完成しました。計画が新規作成を予定しているコードについて、
    類似の既存実装・再利用可能なコードが本当に存在しないかを検証する再探索を行います。
    初回探索で見落とした類似コードの発見が目的です。

    ## 前回のレポート
    .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}

    ## 検証対象（計画の [NEW] 項目）
    {抽出した [NEW] 項目を「名前 / 責務 / 関連キーワード」の形式で列挙}

    ## 指示
    各 [NEW] 項目について、以下を実施してください：
    - 責務・機能名・データ構造から検索キーワードを作り直し、Grep で再検索する（初回と異なる言い回し・同義語も試す）
    - 類似の責務を持つ既存実装・ユーティリティ・ヘルパーがないか確認する（utils/, helpers/, lib/, shared/ 等を重点的に）
    - ヒットしたファイルは必ず Read し、計画の [NEW] 項目と何が重複するかを具体的に比較する

    ## 出力
    - 類似コードを発見した場合: exploration-report の Section 2 に追記して上書き更新し、最終メッセージで「発見一覧（既存コードのパス / 重複する責務 / 再利用可能性）」を報告する
    - 発見がない場合: 最終メッセージで「検証した検索キーワードと対象パス」を添えて「類似コードなし」と報告する
    - exploration-report の Section 8（探索メトリクス）を更新する
```

```
TaskOutput:
  task_id: "{計画後検証codebase-explorerのtask_id}"
  block: true
  timeout: 300000
```

### 4.2-3. 結果による分岐

- **類似コードなし** → Step 4.5 へ進む
- **類似コード発見** → 発見一覧を添えて spec-planner を再起動し、計画を修正する（**最大 1 回**）：

```
Agent tool:
  subagent_type: "spec-planner"
  description: "spec-planner (計画修正): {feature-name}"
  run_in_background: true
  prompt: |
    計画後の再探索で、新規作成を予定していたコードに類似する既存実装が見つかりました。
    implementation-plan と tasks を修正してください。

    ## 発見された類似コード
    {発見一覧（既存コードのパス / 重複する責務 / 再利用可能性）}

    ## 修正方針
    - 再利用できる場合: [NEW] を既存コードの再利用・拡張（[MODIFY]）に変更する
    - 再利用すべきでない場合（既存側に問題がある等）: 計画に「既存コード {X} を再利用しない理由」を明記する

    ## 対象ファイル
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/tasks{TASKS_EXT}
```

修正完了後、Step 4.5 へ進む。再々探索は行わない。

---

## Step 4.5: セルフチェック（3エージェント並列 → オーケストレーター修正）

spec-planner が生成した implementation-plan の品質を、3つの専門サブエージェントで並列評価する。
**エージェントは評価のみ行い、修正はオーケストレーターが実施する。**

| エージェント | 評価観点 | 詳細 |
|------------|---------|------|
| plan-format-checker | フォーマット検証 | セクション構成、コードブロック形式、テストセクション構成、プレースホルダ残留をテンプレートと照合 |
| design-validity-checker | 設計レビュー | コンポーネント分割・責務、データフロー、依存方向、既存アーキテクチャ整合性、エッジケース考慮 |
| test-pattern-checker | テストパターン評価 | ファイル構成、カテゴリ網羅、シナリオ充足、具体性、テスト方針の根拠 |

各エージェントの評価基準の詳細は `agents/` 配下の各エージェントファイルを参照。

### サブエージェント起動（3つ並列）

> **必ず以下の3つの Agent tool を1つのメッセージ内で同時に呼び出すこと。**
> 2つだけ起動して3つ目を省略してはならない。
> 起動後、「起動検証」セクションで3つ全てのレスポンスが得られたことを確認する。

```
Agent tool (並列 1/3):
  subagent_type: "plan-format-checker"
  description: "plan-format-checker: {feature-name}"
  prompt: |
    以下の実装計画がテンプレートの構造に沿っているか検証してください。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}

    ## テンプレート
    - {SKILL_NAME}:implementation-plan
```

```
Agent tool (並列 2/3):
  subagent_type: "design-validity-checker"
  description: "design-validity-checker: {feature-name}"
  prompt: |
    以下の実装計画の設計判断を評価してください。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}
```

```
Agent tool (並列 3/3):
  subagent_type: "test-pattern-checker"
  description: "test-pattern-checker: {feature-name}"
  prompt: |
    以下の実装計画のテストパターン網羅性を評価してください。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}
    - .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}

    ## リファレンス
    - references/test-design-patterns.md
```

### 起動検証

3つの Agent tool の結果を受け取った後、以下を確認する:

- plan-format-checker の結果があるか
- design-validity-checker の結果があるか
- test-pattern-checker の結果があるか

**いずれかのエージェントの結果が欠落している場合、欠落したエージェントのみを再起動する。**

### 結果の処理（オーケストレーター）

3エージェントの評価結果を集約し、以下のフローで処理する。

1. **全エージェントが PASS** → Step 5（AIレビュー）またはユーザー確認へ進む
2. **FAIL / WARN がある場合** → オーケストレーターが以下を実施:
   - 指摘内容を確認し、implementation-plan を直接修正する（**最大2回**）
   - 修正の優先順: コード例の不足 → テストパターンの不足 → 設計の問題
   - design-validity-checker の WARN はユーザー判断に委ねるため、修正対象に含めない
   - 2回修正しても解決しない FAIL、および WARN 項目はユーザーに提示し、対応方針を確認してから次のステップに進む

---

## Step 4.7: テストケース詳細設計（test-cases 生成）【必須】

> **このステップは必ず実行する（`skip-files` に `test-cases` が含まれている場合のみスキップ可）。Step 4.5 のセルフチェック通過後、ユーザー確認に進む前に実行する。**

test-case-designer サブエージェントを起動して **テスト専用の詳細ドキュメント** `test-cases.html` を生成する。

implementation-plan の検証計画セクション（テスト戦略 + テスト表）は**そのまま残す**（要約・戦略レベル）。test-cases.html はその**詳細版**。test-cases.html はマスター詳細型のレビューUIで、CSS・レンダラは固定済み。エージェントは先頭の **DATA スクリプト（`FILES` / `PLAN`）だけ**を埋める（HTMLは書かない）。**test-cases は .md 相当が無いため、config の `output-formats` に関わらず常に `.html` で出力する。** 目的は **実装前にテストの網羅性を人間がレビューできるゲート** を提供すること。

### サブエージェント起動

```
Agent tool:
  subagent_type: "test-case-designer"
  description: "test-case-designer: {feature-name}"
  run_in_background: true
  prompt: |
    implementation-plan の検証計画を起点に、テスト専用ドキュメント test-cases.html を生成してください。
    implementation-plan の検証計画セクションは編集せず（要約として残す）、その詳細版を test-cases.html として作成します。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}  ← 検証計画の起点
    - .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}              ← 要件・受入条件
    - .plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}    ← テストインフラ・対象ファイル

    ## リファレンス
    - references/test-design-patterns.md（機能タイプ分類 §1、タイプ別シナリオ §3、テストインフラ §6）

    ## テンプレート・出力先
    - テンプレート: spec-driven-dev:test-cases
    - 出力: .plugin-workspace/.specs/{nnn}-{feature-name}/test-cases.html （常に .html）

    ## 生成方法（重要）
    test-cases.html はマスター詳細型のレビューUIで、CSS・ヘルパー・レンダラは固定済み。
    1. spec-driven-dev:test-cases を Read する
    2. テンプレート先頭の DATA スクリプト（スキーマ説明コメント + const FILES + const PLAN）を実データで丸ごと置き換える
    3. それ以外（<style>・ヘルパー/レンダラの2スクリプト・HTMLシェル）は1文字も変更しない。<link>→style.css 置換は不要（自己完結済み）
    4. <title> の {機能名} を実際の機能名に置換する
    5. test-cases.html に Write する（プレースホルダ {...} を残さない）

    ## 必須要件（データモデル）
    - FILES[].cases[] の各ケースに id / cat(normal|boundary|error|edge) / prio(high|med|low) / precond / steps / input / expected(具体値) / throws / coverage / ai を付与
    - 各ケースの coverage に k:"関数" を必ず含める（網羅性マトリクスの行になる）
    - 各 file の coverage.fns / coverage.branches を [到達,全体] の数値で記載
    - gaps[] に未カバーの疑いを正直に記載（カバー済なら空配列）
    - PLAN.trace で各要件に最低1ケースを紐づけ、未カバーは status:"gap" で明示
    - test-design-patterns.md §3 の該当タイプのシナリオをすべてチェックし反映
```

```
TaskOutput:
  task_id: "{test-case-designerのtask_id}"
  block: true
  timeout: 240000
```

生成完了後、Step 4.8 へ進む。

---

## Step 4.8: テストケース網羅性検証（test-cases 検証）【必須】

> **このステップは Step 4.7 で test-cases.html を生成した場合に実行する。`skip-files` に `test-cases` が含まれている場合はスキップ。**

test-case-designer が生成した `test-cases.html` の網羅性・具体性を、test-pattern-checker サブエージェント（**詳細モード**）で検証する。エージェントは評価のみ行い、修正はオーケストレーターが実施する。

Step 4.5 の test-pattern-checker は計画のテスト要約（種）を検証する。本ステップはその展開結果である詳細ドキュメントを検証するもので、対象と観点が異なる。

### サブエージェント起動

```
Agent tool:
  subagent_type: "test-pattern-checker"
  description: "test-pattern-checker (詳細): {feature-name}"
  prompt: |
    以下のテストケース詳細ドキュメント（test-cases.html）の網羅性・具体性を検証してください。
    **検証対象は test-cases.html です（詳細モード）。**

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/test-cases.html   ← 検証対象（詳細モード）
    - .plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}

    ## リファレンス
    - references/test-design-patterns.md

    ## 検証観点
    検証対象は test-cases.html 先頭の DATA スクリプト（FILES / PLAN）です。
    詳細モードの D1〜D9（データ妥当性・ケースID・カテゴリ妥当性/網羅・優先度・具体性・カバレッジ整合・シナリオ充足・gaps の正直性・要件トレーサビリティ）を評価すること。
```

```
TaskOutput:
  task_id: "{test-pattern-checker (詳細)のtask_id}"
  block: true
  timeout: 180000
```

### 結果の処理（オーケストレーター）

1. **PASS** → ユーザー確認へ進む
2. **FAIL** → オーケストレーターが test-cases.html の DATA スクリプト（FILES / PLAN）を直接修正する（**最大2回**、CSS・レンダラは触らない）。修正の優先順: データ妥当性/プレースホルダ残留 → カバレッジ整合（k:"関数" 欠落）→ 不足シナリオ・gaps → 具体性。2回修正しても解決しない FAIL はユーザーに提示してから次に進む

---

## ユーザー確認

生成したファイルをユーザーに提示:

1. **specフォルダパス**: `.plugin-workspace/.specs/{nnn}-{feature-name}/` を明示
2. 生成ファイル一覧（各ファイルのフルパス）:
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/hearing-notes{HEARING_NOTES_EXT}`
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/exploration-report{EXPLORATION_REPORT_EXT}`
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}`
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/tasks{TASKS_EXT}`
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/test-cases.html`（テスト網羅性レビュー用・常に .html）
   - `.plugin-workspace/.specs/{nnn}-{feature-name}/tech-reference{TECH_REFERENCE_EXT}`（tech-reference 生成後に追加提示）
3. implementation-plan の内容サマリー
4. tasks のタスク一覧
5. テスト網羅性のレビューには `test-cases.html` をブラウザで開くよう案内する（例: `open .plugin-workspace/.specs/{nnn}-{feature-name}/test-cases.html`）
6. 「修正が必要な場合はお知らせください」

ユーザーが修正を要求した場合:

1. フィードバック明確性チェック（[feedback-clarification.md](feedback-clarification.md) 参照）
2. 曖昧な場合 → AskUserQuestion で具体化してから Step 4 に戻る
3. 明確な場合 → そのまま Step 4（レビュー付きバリアントはレビューループ）に戻る

---

## tech-reference 生成（サブエージェントに委譲）【必須】

> **このステップは `skip-files` に `tech-reference` が含まれていない限り必ず実行すること。ユーザー確認完了でワークフローを終了してはならない。**

`.plugin-workspace/.specs/.config.yml` の `skip-files` に `tech-reference` が含まれている場合のみスキップ可。

ユーザー確認完了後、サブエージェントを起動して tech-reference を生成する。
implementation-plan に登場するすべての技術を、初学者向けに解説するコンパニオンドキュメントを作成する。

### サブエージェント起動

```
Agent tool:
  subagent_type: general-purpose
  description: "tech-reference-writer: {feature-name}"
  run_in_background: true
  prompt: |
    あなたは技術リファレンスライターです。
    implementation-plan を読み込み、そこに登場するすべての技術を
    初学者向けに解説する tech-reference ドキュメントを生成してください。

    読者は、言語やライブラリ、作ろうとしているものの初心者です。
    前提知識ゼロでも理解できる平易な説明を心がけてください。

    ## 入力
    - .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}

    ## テンプレート
    - {SKILL_NAME}:tech-reference（拡張子 {TECH_REFERENCE_EXT} に対応するテンプレートを使用）

    ## 出力先
    - .plugin-workspace/.specs/{nnn}-{feature-name}/tech-reference{TECH_REFERENCE_EXT}

    ## 出力形式に関する注意
    tech-reference の拡張子は {TECH_REFERENCE_EXT} です。
    .html の場合は、{SKILL_NAME}:style を Read してCSSを取得し、
    対応する HTML テンプレートの <link> を <style>{CSS}</style> に置換して、
    自己完結型HTMLとして出力してください。

    ## 執筆スタイル — MDN / ライブラリドキュメント風
    テンプレ的な箇条書き（What/Why/Concepts/Code の繰り返し）ではなく、
    MDN や React 公式ドキュメントのように **流れる散文** で書くこと。
    「実装で詰まった部分をその都度引ける辞書」として機能する粒度で記述する。

    ## 構成ルール
    - 冒頭に **概要セクション** を置き、この機能が何をするか・全体像を2-3段落で説明する
    - 概要に **全体フロー図** を入れる（技術間の関係・データの流れ）
    - 各技術は **1つのミニ記事** として書く（辞書エントリではなく）:
      - 定義から入り、この機能での利用箇所に繋げる散文
      - 仕組みを掘り下げ、メンタルモデルを説明する散文
      - **動作モデルの図** で視覚化
      - **実践的なコード例**（implementation-plan の変更案に沿ったもの）
      - 重要な関数/API は **シグネチャ** + **パラメータ表** で詳細に記述
      - 注意点・ヒントは **メモ / 注意** ブロックで囲む
    - 末尾に **用語集** を置く
    - implementation-plan の変更案に登場するすべての技術をカバーする
    - 外部URLは含めない（Web検索なしで完結させる）
    - 該当する技術がないカテゴリセクションは省略する

    ## 粒度の目安
    - 読者が「この技術って何？」から「このコード、何をしてるか分かった」まで到達できる深さ
    - 専門用語が出たら、その場で噛み砕いて説明を添える
    - 図と文を連携させる（「上の図で◯◯が△△に渡され…」）

    ## 図の描画形式
    - .md 出力の場合: ASCII罫線図を使用する
    - .html 出力の場合: **SVG を使用する**。
      `<div class="svg-diagram"><svg>` または `<figure class="lib-figure">` 内に描画する。
      テンプレートで定義された CSS クラスを活用すること。
      テンプレートの SVG 例を参考にすること

    ## HTML 出力時のスタイル
    .html 出力の場合は、テンプレートのライブラリドキュメント用 CSS クラスを使用する:
    - レイアウト: `lib-doc`, `lib-sidebar`, `lib-main`
    - 見出し: `lib-h1`, `lib-h2`（`.kicker`, `.hash`）, `lib-h3`
    - テキスト: `lib-p`, `lib-intro`, `lib-inline`
    - コード: `lib-code-block`（`.lib-code-lang` + `pre`）
    - シグネチャ: `lib-signature`（`.sig-key`, `.sig-str`, `.sig-type`）
    - パラメータ表: `lib-param-table`（`.param-name`, `.param-tag`, `.param-type`）
    - 注釈: `lib-note`（`lib-note-info` / `lib-note-warn`）
    - 図: `lib-figure`, `lib-figure-inner`, `svg-diagram`
    - 用語集: 通常の `<table>`
    - フッター: `lib-footer`

    ## 品質チェック
    - [ ] 冒頭に概要セクションと全体フロー図があるか
    - [ ] implementation-plan の変更案に登場するすべての技術が記事としてカバーされているか
    - [ ] 各技術セクションが散文（箇条書きの羅列ではない）で書かれているか
    - [ ] 各技術セクションに動作モデルの図があるか（.md → ASCII図、.html → SVG図）
    - [ ] 重要な関数/API にシグネチャとパラメータ表があるか
    - [ ] コード例が implementation-plan の変更案に沿った実践的なものか
    - [ ] 用語集に implementation-plan の専門用語がすべて含まれているか
```

```
TaskOutput:
  task_id: "{tech-reference-writerのtask_id}"
  block: true
  timeout: 180000
```

### 生成後の提示

tech-reference 生成完了後、ユーザーに以下を追加提示する:

- `tech-reference{TECH_REFERENCE_EXT}` のファイルパス
- 「技術リファレンスを生成しました。implementation-plan と合わせてご参照ください。」

---

## Step 6.7: 実装前理解度クイズ生成

plan を実装セッションに渡す直前（ガード解除の前）に、**実装者（人間）の設計理解度を測るクイズ**を
HTML アーティファクトとして生成する。これは「設計がまだ自分の中で固まっているか」を確認し、
unknowns を炙り出す **advisory ゲート**。落ちた設問は実装に渡す前に潰す。

**生成の有無は config で決まる**: `.plugin-workspace/.specs/.config.yml` の `skip-files` に
`understanding-quiz` が含まれていればこのステップをスキップする（`/spec-setup` で設定）。
含まれていなければ生成する。

**出力先も config で決まる**: `.config.yml` の `quiz-output.plan` が `artifact` なら Artifact ツールで公開、
`interactive` なら HTML を生成せずセッション内で AskUserQuestion により1問ずつ対話出題、
`file` または未設定なら specフォルダ内の HTML ファイルとして出力する（**デフォルト: `file`**）。
決定した値を以降 `{QUIZ_OUTPUT}` として参照する。`artifact` でも実行環境に Artifact ツールが
存在しない場合（CLI 等）は `file` にフォールバックする。`interactive` でも対話できない
実行環境（非対話実行）では `file` にフォールバックする。

出題設計の共通指針は `references/quiz-design.md` を参照。

### 6.7-1. 材料の読み込み

以下を材料として読み込む（既存資産を再利用するだけで、新規に材料を用意する必要はない）:

- `requirements.md` — 確定したユースケース・要件・制約（設計判断の前提）
- `implementation-plan{IMPLEMENTATION_PLAN_EXT}` — 設計・データモデル・変更案
- `hearing-notes{HEARING_NOTES_EXT}` — どの unknowns をどう解決したかの記録
  （「なぜこの設計にしたか」の答えの根拠がここにある）

### 6.7-2. 設問の作成

`references/quiz-design.md` の「実装前クイズ（設計意図）の出題観点」に沿って、
以下を問う設問を **合計 5〜10 問**（choice / boolean / order を混ぜる）作る:

- なぜこの設計にしたか（設計判断の理由）
- このデータモデル / 型インターフェースにした理由
- この制約が満たされないと何が壊れるか
- 変更されやすい箇所（data model / 型 / UX フロー）の意図

材料に照らして正解が一意に決まる設問だけを作る。誤答は「ざっと読むと選びそうな別解釈・別設計」にする。
クイズは回答した時点で1問ずつ正誤と解説が表示される（即時フィードバック）ため、
ある設問の解説・選択肢が別の設問の答えを含まないよう、設問間を独立させる（quiz-design.md 参照）。

作成後、`references/quiz-design.md` の「設問セルフレビュー」チェックリストで設問を1問ずつ検査し、
該当する設問は修正するか捨てる（観点ごとに多めにドラフトして削るのが推奨フロー）。

### 6.7-3. クイズ生成

**`{QUIZ_OUTPUT}` が `interactive` の場合はこの節をスキップし、6.7-3b に進む。**

`test-cases.html` と同様の **DATA スクリプト差し替え方式**。CSS・レンダラ・採点ロジックは固定済み。

1. テンプレート `assets/templates/understanding-quiz-plan.html` を Read する
2. テンプレート先頭の DATA スクリプト（スキーマ説明コメント + `const QUIZ`）を実データで丸ごと置き換える
3. それ以外（`<style>`・レンダラの2スクリプト・HTMLシェル）は1文字も変更しない。
   **test-cases.html と同じく自己完結HTMLのため、`<link>`→style.css 置換は不要。**
4. `<title>` の `{機能名}` を実際の機能名に置換する
5. `{QUIZ_OUTPUT}` に応じて出力する（プレースホルダ `{...}` を残さない）:
   - **`file` の場合（デフォルト）**: `.plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-plan.html` に Write する
   - **`artifact` の場合**: 外側のシェルタグ（`<!doctype html>` `<html>` `<head>` `</head>` `<body>` `</body>` `</html>`）を取り除き、
     `<title>`・`<style>`・body 内容・スクリプトを残した形で
     `.plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-plan.html` に Write し、
     Artifact ツールで公開する（Artifact 側がシェルを付与するため。favicon は `📝`）

> **config の `output-formats` に関わらず常に `.html`**（本質的にインタラクティブなレビューUIのため）。

### 6.7-3b. 対話出題（`{QUIZ_OUTPUT}` が `interactive` の場合）

HTML を生成せず、`references/quiz-design.md` の「対話モード（quiz-output: interactive）の実施方法」に
従い、6.7-2 で作成した設問を AskUserQuestion で1問ずつ出題する:

1. 出題前に「これから実装前の理解度クイズを{N}問、1問ずつ出題します」と伝える
2. 1問ずつ AskUserQuestion で出題（選択肢はシャッフル。order 設問は「正しい順序はどれか」の choice に変換）
3. 回答直後に正誤と解説（材料の根拠箇所つき）を伝えてから次の設問へ。初回の回答で正誤を確定し、出し直さない
4. 全問終了後、スコアと passLine による合否、間違えた設問ごとに読み直すべき材料の該当箇所を提示する

### 6.7-4. ユーザーへの提示

- `{QUIZ_OUTPUT}` が `file` の場合: `understanding-quiz-plan.html` のファイルパス
  （例: `open .plugin-workspace/.specs/{nnn}-{feature-name}/understanding-quiz-plan.html`）
- `{QUIZ_OUTPUT}` が `artifact` の場合: Artifact の URL
- `{QUIZ_OUTPUT}` が `interactive` の場合: 6.7-3b の結果（スコア・合否・読み直し箇所）を提示済みのため、
  ファイル・URL の案内は不要
- 「実装前の理解度クイズを生成しました。実装を始める前に、開いて設計意図を確認してください。」（file / artifact の場合）
- 「これは advisory ゲートです。落ちた設問がある＝設計がまだ固まっていない箇所なので、
  その論点を計画・要件で詰め直してから実装に進むことをおすすめします。」

---

## ガード解除（USE_GUARD = true の場合）

計画が完了したら、ユーザーに以下を案内する:

1. ガードファイルの削除（**ユーザーが手動で実行**）
2. PLANNINGファイルの削除

```
ユーザーへの案内:
  実装を開始するには、以下のコマンドを実行してください:
  rm .plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID} .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
```

**注意**: ガードファイルはhookにより自動削除がブロックされる。必ずユーザーが手動で削除すること。
**注意**: ガード解除前に実装コードを書いてはならない。

## PLANNINGファイル削除（USE_GUARD = false の場合）

ユーザーから実装開始の許可を得たら、PLANNINGファイルを削除する。

```bash
rm .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
```

**注意**: PLANNINGファイル削除前に実装コードを書いてはならない。
