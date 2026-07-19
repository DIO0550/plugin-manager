# コードレビュー統合レポート: {nnn}-{feature-name}

**レビュー日時**: {datetime}
**レビュー回数**: {NNN}
**レビュー対象**: {review-scope}
**参照ドキュメント**（spec文書がない場合は省略）:
- hearing-notes: `{spec-dir}/hearing-notes.md`
- exploration-report: `{spec-dir}/exploration-report.md`
- implementation-plan: `{spec-dir}/implementation-plan.md`
- tasks.md: `{spec-dir}/tasks.md`

---

## インテントサマリー

{hearing-notes に基づく機能の目的を2-3文で要約。この機能がユーザーにどんな価値を提供するか、スコープは何か。}

---

## レビュー結果サマリー

| 分類 | 件数 | 説明 |
|------|------|------|
| 🔴 CRITICAL | {n} | 仕様違反 — 必ず修正 |
| 🟡 WARNING  | {n} | 要確認 — 基本修正 |
| 🔵 INFO     | {n} | 参考情報 — 任意対応 |

---

## 🔴 CRITICAL: 仕様違反

{CRITICAL 指摘がない場合: 「仕様違反は検出されませんでした。」}

### C-{NNN}: {タイトル}

- **次元**: {担当次元名}
- **レビューアー**: {エージェント名}
- **対象**: `{ファイルパス}` L{行番号}
- **根拠**: {ルール根拠（原則名） または 仕様根拠（spec文書名 Section X: "引用"）}
- **問題**: {問題の説明}
- **改善案**: {具体的な修正提案}

---

## 🟡 WARNING: 要確認

{WARNING 指摘がない場合: 「要確認事項はありません。」}

### W-{NNN}: {タイトル}

- **次元**: {担当次元名}
- **レビューアー**: {エージェント名}
- **対象**: `{ファイルパス}` L{行番号}
- **根拠**: {ルール根拠（原則名） または 仕様根拠（spec文書名 Section X: "引用"）}
- **問題**: {問題の説明}
- **改善案**: {具体的な修正提案}

---

## 🔵 INFO: 参考情報

{INFO 指摘がない場合: 「特記事項はありません。」}

### I-{NNN}: {タイトル}

- **レビューアー**: {エージェント名}
- **対象**: `{ファイルパス}` L{行番号}
- **観察**: {観察内容}

---

## DoD充足状況

{DoD セクションが存在しない場合: 「implementation-plan に DoD セクションが存在しないため、スキップしました。」}

| # | DoD項目 | 状態 | 備考 |
|---|---------|------|------|
| 1 | {項目} | ✅ / ❌ | {確認結果} |

---

## レビューソース

| エージェント | 個別レポート | 指摘数 |
|-------------|-------------|--------|
| performance-reviewer | `spec-based-code-review/performance-{NNN}.md` | {n} |
| design-reviewer | `spec-based-code-review/design-{NNN}.md` | {n} |
| spec-alignment-reviewer | `spec-based-code-review/alignment-{NNN}.md` | {n} |
| test-quality-reviewer | `spec-based-code-review/test-quality-{NNN}.md` | {n} |
| comment-quality-reviewer | `spec-based-code-review/comment-quality-{NNN}.md` | {n} |
