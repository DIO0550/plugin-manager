# レビューツールコマンド一覧

plan-review 用のレビューツールコマンド。SKILL.md のレビューステップから参照される。

---

## プロンプトファイルの組み立て

レビュー実行前に、Writeツールで `.plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/prompt-{NNN}.txt` にレビュー指示文を書き出す。
`{NNN}` は3桁の連番（001, 002, 003...）。再レビュー時はインクリメントする。

**prompt-{NNN}.txt の内容**:

```
以下の実装計画をレビューしてください。

【重要】ファイルの作成・編集は一切行わないでください。レビュー結果は標準出力のみで回答してください。

レビュー観点:
1. 仕様の曖昧さ・抜け漏れはないか
2. 実装可能性に問題はないか
3. エッジケースは考慮されているか
4. ファイル構成は妥当か
5. 全体アーキテクチャとの整合性はあるか
6. テスト戦略・テストケースの網羅性は適切か

問題がなければ「問題なし」と回答してください。
問題があれば具体的な指摘と改善案を提示してください。
```

バリアントごとの補足（下記参照）があればプロンプト末尾に追記する。

---

## Codex CLI

```bash
codex exec --cd "$PWD" --dangerously-bypass-approvals-and-sandbox "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/prompt-{NNN}.txt)" > .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/review-{NNN}.md
```

**プロンプト補足** — prompt-{NNN}.txt の末尾に追記:
```
レビュー対象: .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT}
```

---

## GitHub Copilot CLI

```bash
copilot -p "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/prompt-{NNN}.txt)" < .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT} > .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/review-{NNN}.md
```

**プロンプト補足** — prompt-{NNN}.txt の末尾に追記:
```
以下は標準入力で渡される implementation-plan{IMPLEMENTATION_PLAN_EXT} の内容です:
```

---

## Claude Code CLI

```bash
claude -p "$(cat .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/prompt-{NNN}.txt)" .plugin-workspace/.specs/{nnn}-{feature-name}/implementation-plan{IMPLEMENTATION_PLAN_EXT} > .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/review-{NNN}.md
```

**プロンプト補足**: 不要（ファイルが引数で直接渡されるため）

---

## レビュー結果の保存先

```bash
mkdir -p .plugin-workspace/.specs/{nnn}-{feature-name}/plan-review
```

レビュー結果: `.plugin-workspace/.specs/{nnn}-{feature-name}/plan-review/review-{NNN}.md`

---

## ループ処理

1. 保存したレビュー結果ファイルを読み込み、内容を解析
2. 「問題なし」なら次のステップ（ユーザー確認）へ進む
3. 問題があれば:
   - 指摘内容を元に implementation-plan{IMPLEMENTATION_PLAN_EXT} を修正
   - **反映履歴は書かない**: 設計内容そのものだけを修正する
   - 連番をインクリメントして再度レビューを実行・保存
   - **最大5回**までループ
4. 5回超えたらユーザーに相談

レビュー観点の詳細は `references/review-criteria.md` を参照。

---

## 出力構造

```
.plugin-workspace/.specs/{nnn}-{feature-name}/
└── plan-review/
    ├── prompt-001.txt
    ├── review-001.md
    ├── prompt-002.txt
    ├── review-002.md
    └── ...
```
