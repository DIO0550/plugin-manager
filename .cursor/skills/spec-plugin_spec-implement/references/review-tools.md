# レビューツールコマンド一覧（コードレビュー用）

全タスク完了後のコードレビュー用コマンド。SKILL.md のレビューステップから参照される。

---

## レビュー結果の保存先

```bash
mkdir -p .plugin-workspace/.specs/{nnn}-{feature-name}/code-review
```

レビュー結果: `.plugin-workspace/.specs/{nnn}-{feature-name}/code-review/review-{NNN}.md`
`{NNN}` は3桁の連番（001, 002, 003...）。再レビュー時はインクリメントする。

---

## コンテキストファイルの組み立て

レビュー実行前に、Writeツールで以下の2ファイルを作成する。

**`.plugin-workspace/.specs/{nnn}-{feature-name}/code-review/context-{NNN}.md`**:

以下の内容を結合して書き出す:
1. `## 実装計画` + implementation-plan.md の内容
2. `## 実装タスク一覧` + tasks.md の全タスク内容
3. `## 変更されたファイル` + `git diff --name-only` の結果
4. `## 変更内容` + `git diff` の結果

**`.plugin-workspace/.specs/{nnn}-{feature-name}/code-review/prompt-{NNN}.txt`**:

```
全タスクの実装をレビューしてください。

【重要】ファイルの作成・編集は一切行わないでください。レビュー結果は標準出力のみで回答してください。

## レビュー対象
.plugin-workspace/.specs/{nnn}-{feature-name}/code-review/context-{NNN}.md を読み込んでレビューしてください。

## レビュー観点
1. 実装計画との整合性: 計画通りに実装されているか
2. コード品質: 可読性、保守性、命名規則は適切か
3. エッジケース: 空データ、エラー、境界値を考慮しているか
4. セキュリティ: インジェクション、XSSなどの脆弱性はないか
5. パフォーマンス: 不要なループ、N+1問題などはないか

問題がなければ「問題なし」と回答してください。
問題があれば具体的な指摘と改善案を提示してください。
```

---

## Codex CLI

```bash
codex exec --cd "$PWD" --dangerously-bypass-approvals-and-sandbox "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/prompt-{NNN}.txt)" > .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/review-{NNN}.md
```

---

## GitHub Copilot CLI

```bash
copilot -p "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/prompt-{NNN}.txt)" < .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/context-{NNN}.md > .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/review-{NNN}.md
```

---

## Claude Code CLI

```bash
claude -p "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/prompt-{NNN}.txt)" .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/context-{NNN}.md > .plugin-workspace/.specs/{nnn}-{feature-name}/code-review/review-{NNN}.md
```

---

## ループ処理

1. 保存したレビュー結果ファイルを読み込み、内容を解析
2. 「問題なし」なら次のステップへ進む
3. 問題があれば:
   - 指摘内容を元にコードを修正
   - 連番をインクリメントして再度レビューを実行・保存
   - **最大5回**までループ
4. 5回超えたらユーザーに相談

---

## 出力構造

```
.plugin-workspace/.specs/{nnn}-{feature-name}/
└── code-review/
    ├── context-001.md
    ├── prompt-001.txt
    ├── review-001.md
    ├── context-002.md
    ├── prompt-002.txt
    ├── review-002.md
    └── ...
```
