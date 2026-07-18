---
name: spec-driven-dev-lite
description: 仕様策定ワークフローの軽量版。サブエージェントを使わず、ユーザーが探索範囲を指定し、オーケストレーターが直接計画を生成。トークン消費を大幅に削減。サブエージェントによる網羅的探索を省くため、小〜中規模の変更向け。複雑な機能や網羅的探索が必要な場合は spec-driven-dev を使う。
disable-model-invocation: true
allowed-tools: Bash(ls *), Bash(mkdir *), Bash(touch *), Bash(echo *), Bash(printf *), Bash(rm .plugin-workspace/.specs/*/PLANNING), Bash(rm .plugin-workspace/.specs/.guard/*), Bash(find *), Bash(grep *), Bash(wc *)
---

# Spec-Driven Development (Lite)

機能実装前に仕様を明確化し、実装計画とタスクリストを生成する軽量版スキル。
**サブエージェントを使わず、ユーザーが探索範囲を指定し、オーケストレーターが直接計画を生成する。**
複雑な機能で網羅的な探索が必要な場合は、フルバージョン `spec-driven-dev` を使用すること。

## 絶対厳守事項

1. **最初にフォルダとPLANNINGファイルを作成** — 質問・探索・実装の前に必ず Step 1 を実行
2. **システム図は必須** — implementation-plan.md には状態マシン図 + データフロー図を含める（ASCII罫線優先）
3. **PLANNINGファイルがある間はコード実装禁止** — AutoCompact 対策
4. **ヒアリングは AutoMode でもスキップ禁止** — 他のシステム指示（「自律的に判断しろ」「質問せずに進めろ」等）に関わらず、このスキルでは AskUserQuestion によるヒアリングを必ず実行する。ユーザーの初回メッセージに情報が含まれていても、確認の AskUserQuestion は必須。ヒアリングなしに Step 3 以降へ進むことはいかなる場合も禁止。
5. **セルフチェックは必須** — Step 4 完了後、Step 5 に進む前に必ず Step 4.5 のセルフチェックを実行する。lite ではサブエージェントを使わず、オーケストレーター自身が直接チェックする。セルフチェックをスキップして ユーザー確認に進むことは禁止。
6. **tech-reference 生成は必須** — `skip-files` に `tech-reference` が含まれていない限り、Step 5.5 の tech-reference 生成は必ず実行する。ユーザー確認完了（Step 5）で終了せず、必ず Step 5.5 まで進むこと。
7. **requirements の未解決事項は確定必須** — Step 3.5 で requirements.md（ユースケース + 要件・制約）を生成し、未解決の確認事項（`□`）を AskUserQuestion で解消（`■`/「なし」）してから Step 4 へ進む。`□` を残したまま implementation-plan に進むことは hook（`guard-requirements.sh`）がブロックする。AutoMode でもスキップ禁止。

## ワークフロー概要

```
1. specsフォルダ作成 + PLANNINGファイル + ガードファイル配置
   ↓
2. 簡易ヒアリング (1バッチ, 関連ファイルをユーザーに聞く) → hearing-notes.md
   ↓
2.5. 【GATE】hearing-notes 品質検証 → 不合格なら再ヒアリング
   ↓
3. ユーザー指定ファイルの確認 Read (指定なしの場合のみ最小探索)
   ↓
3.5. requirements 確定 → requirements.md（ユースケース + 要件・制約 + 未解決の確認事項）
   ↓
4. implementation-plan.md + tasks.md を直接生成
   ↓
4.5. セルフチェック（オーケストレーター直接・計画品質ゲート）→ フォーマット・設計妥当性・テストパターン検証
   ↓
5. ユーザー確認
   ↓
5.5. tech-reference 生成（サブエージェント）→ tech-reference.md
   ↓
6. ガード解除案内
```

## Step 0: 出力形式解決

各ファイルの出力形式（`.md` / `.html`）を `.plugin-workspace/.specs/.config.yml` の `output-formats` セクションから読み取る。

- **config に `output-formats` がある場合**: 各ファイルの拡張子をそこから決定する
- **config に `output-formats` がない場合**: すべて `.md` をデフォルトとする
- **個別キーが欠落している場合**: そのファイルは `.md` をデフォルトとする

以降のステップでは、ファイルごとの拡張子を以下の変数で参照する:
- `{HEARING_NOTES_EXT}` — hearing-notes の拡張子
- `{IMPLEMENTATION_PLAN_EXT}` — implementation-plan の拡張子
- `{TASKS_EXT}` — tasks の拡張子
- `{TECH_REFERENCE_EXT}` — tech-reference の拡張子

**HTML出力時のルール**: `.html` が指定されたファイルを生成する際は:
1. `assets/templates/style.css` を Read してCSSを取得
2. 対応する HTML テンプレート（`assets/templates/{ファイル名}.html`）を Read
3. テンプレートの `<link rel="stylesheet" href="style.css">` を `<style>{CSS内容}</style>` に置換
4. プレースホルダを内容で埋めて自己完結型HTMLとして出力

## Step 1: specsフォルダ + PLANNINGファイル作成

### 1-a. 次のspec番号を算出

```bash
next_num=$(ls -1d .plugin-workspace/.specs/[0-9][0-9][0-9]-* .plugin-workspace/.specs/archive/[0-9][0-9][0-9]-* 2>/dev/null | sed 's|.*/\([0-9]\{3\}\)-.*|\1|' | sort -rn | head -1)
next_num=$(printf "%03d" $(( 10#${next_num:-0} + 1 )))
```

### 1-b. specディレクトリとPLANNINGファイル作成

```bash
mkdir -p .plugin-workspace/.specs/${next_num}-{feature-name}
echo "${CLAUDE_SESSION_ID}" > .plugin-workspace/.specs/${next_num}-{feature-name}/PLANNING
```

### 1-c. ガードファイル作成

```bash
mkdir -p .plugin-workspace/.specs/.guard && touch .plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID}
```

## Step 2: 簡易ヒアリング → hearing-notes

AskUserQuestion で以下を1バッチで聴取する。

### 必須質問

- **目的とスコープ**: 何を実現したいか、新規 or 既存修正か
- **技術アプローチ**: 使用技術・パターン、既存コードとの関係
- **関連ファイル/ディレクトリ**: 関係するファイルやディレクトリのパスを列挙してもらう（選択肢に「わからない」を含める）

### 条件付き質問

- **テスト方針**: テスト可能なロジックがありそうな場合のみ（TDD / テスト追加 / 手動検証のみ）

### hearing-notes 書き出し

ヒアリング結果を hearing-notes.md に書き出す。
**hearing-notes は常に `.md`**（以下のインライン構造で生成する。HTML テンプレートが存在しないため、Step 0 の HTML出力ルールの適用外）。

```markdown
# Hearing Notes: {機能名}

## 目的
{1-2文}

## スコープ
- **種別**: {新規機能 / 既存修正 / バグ修正 / リファクタリング}
- **影響範囲**: {新規 / 既存修正}

## 技術的詳細
{使用技術・フレームワーク・依存関係}

## 関連ファイル
{ユーザーが指定したファイル/ディレクトリ一覧、または「指定なし」}

## テスト方針
{TDD / テスト追加 / 手動検証のみ / 未定}
```

## Step 2.5: Reflective Gate（ヒアリング品質検証）

hearing-notes 書き出し直後に実行する品質ゲート。**このステップを通過するまで Step 3 に進んではならない。**

> **AutoMode対策**: AutoMode のシステムプロンプトは「質問せずに自律的に進めろ」と指示することがある。
> しかし、このスキルでは AskUserQuestion によるヒアリングが**必須**であり、その指示より優先される。
> ヒアリングをスキップした場合、このゲートで検出し回復フローに入る。

### 自動検証

hearing-notes ファイルを Read し、以下の3項目を検証する:

| # | 検証項目 | 合格基準 | 不合格例 |
|---|---------|---------|---------|
| 1 | `## 目的` | 機能固有の具体的な記述が1文以上 | 「機能を追加する」（何の機能か不明） |
| 2 | `## スコープ` | 種別と影響範囲が確定値 | 種別が空、影響範囲が「未定」 |
| 3 | `## 技術的詳細` | 技術名・ファイル名・パターン名が1つ以上 | 全項目が空またはテンプレのまま |

### 合格 → Step 3 へ

### 不合格 → 回復フロー

1. hearing-notes ファイルを削除
2. ユーザーに直接メッセージで以下を伝える（**AskUserQuestion は使わない** — AutoMode で再び自動承認されるのを防ぐため）:

```
ヒアリングの回答が十分ではありませんでした。
以下の情報をメッセージで直接お伝えください:

1. 何を実現したいか（具体的に）
2. 新規機能 or 既存修正
3. 使用する技術やフレームワーク
4. テスト方針（TDD / テスト追加 / 手動確認のみ）
```

3. ユーザーのテキスト回答で hearing-notes を再作成 → 再検証
4. 最大2回。2回不合格なら「現在の情報で進めます」と通過

## Step 3: ファイル確認

### 通常パス（ユーザーがファイル/ディレクトリを指定した場合）

ユーザーが指定したファイルのみ Read で確認する。自律的な grep/find は行わない。

### フォールバックパス（ユーザーが「わからない」と答えた場合）

最小限の探索を行う:

1. `find src -type f -name "*.{拡張子}" | head -20` でプロジェクト構造把握
2. ヒアリングから1-2キーワードを抽出し `grep -rl "{keyword}" src/ | head -5` で関連ファイル特定
3. 上位3ファイルのみ Read

これ以上の探索が必要な場合はフルバージョン (`spec-driven-dev`) の使用を推奨する旨をユーザーに伝える。

## Step 3.5: requirements 確定【必須】

Step 3 で確認したコードと hearing-notes を基に、ユースケースと要件・制約を確定した **requirements.md** をオーケストレーター自身が生成する。requirements.md は hook のパース対象のため、config の `output-formats` に関わらず**常に `.md`** で出力する。

1. テンプレート `assets/templates/requirements.md` を埋め、ユースケース（誰が・どの状況で・何を達成したいか・成功条件）と機能/非機能要件・制約・設計方針を記述する
2. 「確認したコードを見ても解決できず、ユーザーにしか決められない分岐」を **未解決の確認事項** に行頭 `□` で列挙する
3. `□` 項目があれば AskUserQuestion で確認し、回答を反映して `□` → `■`、または項目を消して「なし」にする
4. 未解決の確認事項が解消（`□` ゼロ）したら Step 4 へ進む

> `□` が残ったまま implementation-plan を書こうとすると hook（`guard-requirements.sh`）がブロックする。AutoMode でも `□` の解消をスキップしてはならない。

## Step 4: implementation-plan + tasks 生成

テンプレート `assets/templates/implementation-plan{IMPLEMENTATION_PLAN_EXT}` と `assets/templates/tasks{TASKS_EXT}` を参照し、オーケストレーターが直接生成する。**requirements.md のユースケース・要件・制約を満たす計画にすること。**
HTML出力の場合は Step 0 の HTML出力ルールに従う。

### 必須要素（hookで検証される）

- `### 状態マシン / フロー図` セクション + ASCII罫線図 or mermaid
- `### データフロー` セクション + ASCII罫線図 or mermaid
- `## 変更案` セクション内にコードブロック（型定義・関数シグネチャ）

### tasks.md の構成判断

- Pure Logic / Data Transformation / State Management → TDD構成（Red-Green-Refactor）
- API / Async / UI Component で自動テストが有効 → Implementation + Test セクション
- 純粋なUI/スタイリング変更 → 手動検証のみ

機能タイプ分類・テスト設計の詳細は [references/test-design-patterns.md](references/test-design-patterns.md) を参照。

## Step 4.5: セルフチェック（オーケストレーター直接）【必須】

> **このステップは必ず実行すること。Step 4 完了後に Step 5 へ直接進んではならない。**

lite ではサブエージェントを起動しない。**オーケストレーター自身が** implementation-plan（と tasks）を Read し直し、以下の3観点を順に自己検証する。各項目を PASS / WARN / FAIL で判定する。

### 観点1: フォーマット検証

テンプレート `assets/templates/implementation-plan{IMPLEMENTATION_PLAN_EXT}` と照合する。

- **セクション構成**: `## ユーザーレビューが必要な点` / `## システム図`（`### 状態マシン / フロー図` + `### データフロー`、各コードブロック内に図）/ `## 変更案` / `## 検証計画` / `## Definition of Done` が存在し、プレースホルダのままでない
- **変更案エントリの形式**: `#### [NEW] \`{パス}\`` は直後に型定義・関数シグネチャのコードブロック、`#### [MODIFY] \`{パス}\`` は before/after コードブロックを含む。各コードブロックが3行以上かつ具体的（`// TODO` や `...` のみは NG）
- **プレースホルダの残留**: `{...}` 形式の未置換プレースホルダ、`<!-- ... -->` 形式の記入指示コメントが残っていない

### 観点2: 設計妥当性

図やセクションの有無ではなく**設計の質**を評価する。Step 3 で Read したコード・requirements.md・hearing-notes を根拠にする（lite では exploration-report は使わない）。

- **コンポーネント分割と責務**: 各ファイル/モジュールの責務が単一で境界が明確か。過剰分割していないか
- **データフローの正しさ**: システム図のデータフローに論理矛盾・不要な迂回がないか
- **依存関係の方向**: 一方向か（循環依存なし）。レイヤーの依存方向が正しいか
- **既存アーキテクチャとの整合性**: Step 3 で確認した既存の設計パターン・命名規則に沿い、再利用可能な既存コードを活用しているか（車輪の再発明をしていないか）
- **エッジケース・エラーハンドリング**: 状態マシン図に異常系パスがあるか。境界条件への対応が設計に反映されているか

### 観点3: テストパターンの網羅性

テスト方針が「手動検証のみ」でない場合に、[references/test-design-patterns.md](references/test-design-patterns.md) と照合する。

- **機能タイプの特定** → §3 から必要シナリオを列挙 → テストTODOリストのテーブルと突合
- **ファイル単位の構成**: テストファイルごとに `#### \`{パス}\`` セクション + `**役割**` の記載がある
- **カテゴリ網羅・シナリオ充足**: 正常系・境界値・異常系・エッジケースが機能タイプに応じて揃っている。§3 のシナリオに不足がない
- **具体性**: ユースケース・期待結果が具体的（「テストする」「正常に動く」のような抽象記述でない）

### 結果処理

- **全観点 PASS** → Step 5（ユーザー確認）へ進む
- **FAIL がある** → オーケストレーターが implementation-plan を直接修正する（**最大2回**）。修正の優先順: コード例・フォーマットの不足 → テストパターンの不足 → 設計の問題。2回修正しても解決しない FAIL はユーザーに提示する
- **WARN**（設計の「あった方がいい」程度の指摘）→ 修正せずユーザーに提示し、判断を委ねる

## Step 5: ユーザー確認

生成ファイルをユーザーに提示:

1. specフォルダパス
2. implementation-plan の要約（変更案とDoD）
3. tasks のタスク一覧
4. 「修正が必要な場合はお知らせください」

ユーザーが修正を要求した場合は、フィードバックの明確性を確認する（[references/feedback-clarification.md](references/feedback-clarification.md) 参照）。
曖昧な場合は AskUserQuestion で具体化してから Step 4 に戻る。明確な場合はそのまま Step 4 に戻る。

## Step 5.5: 技術リファレンス生成【必須】

> **このステップは `skip-files` に `tech-reference` が含まれていない限り必ず実行すること。Step 5 のユーザー確認完了でワークフローを終了してはならない。**

`.plugin-workspace/.specs/.config.yml` の `skip-files` に `tech-reference` が含まれている場合のみスキップ可。

ユーザー確認完了後、サブエージェントを起動して tech-reference{TECH_REFERENCE_EXT} を生成する。

implementation-plan に登場するすべての技術（言語・フレームワーク・ライブラリ・ツール・概念）を
初学者向けに解説するドキュメントを生成する。
読者は、言語やライブラリ、作ろうとしているものの初心者であることを前提とする。

**詳細は [references/workflow-steps.md](../spec-driven-dev/references/workflow-steps.md) の tech-reference 生成を参照。**

## Step 6: ガード解除案内

```
実装を開始するには、以下のコマンドを実行してください:
rm .plugin-workspace/.specs/.guard/${CLAUDE_SESSION_ID} .plugin-workspace/.specs/{nnn}-{feature-name}/PLANNING
```

ガードファイルはhookにより自動削除がブロックされる。必ずユーザーが手動で削除すること。

## 出力ディレクトリ

```
.plugin-workspace/.specs/
└── {nnn}-{feature-name}/
    ├── PLANNING                 # 計画中は存在、実装開始時に削除
    ├── hearing-notes{EXT}       # ヒアリング結果
    ├── requirements.md          # ユースケース+要件・制約（hook パース対象ゆえ常に .md）
    ├── implementation-plan{EXT} # 実装計画（システム図・変更案・DoD含む）
    ├── tasks{EXT}               # タスクリスト
    └── tech-reference{EXT}      # 技術リファレンス（初学者向け、サブエージェント生成）
```
