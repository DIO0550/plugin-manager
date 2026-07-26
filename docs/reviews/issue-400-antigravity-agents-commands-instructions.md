# Issue #400 調査レビュー: Antigravity Agents / Commands / Instructions

| 項目 | 内容 |
|------|------|
| Issue | [#400 [research/antigravity] Agents / Commands / Instructions の公式サポート調査と PLM 対応方針の決定](https://github.com/DIO0550/plugin-manager/issues/400) |
| レビュー日 | 2026-07-26 |
| 対象ブランチ | `main`（`3856475`） |
| 種別 | 調査（research）→ 方針決定 |
| 関連 | Hooks [#309](https://github.com/DIO0550/plugin-manager/issues/309)、Gemini レガシー化 [#402](https://github.com/DIO0550/plugin-manager/issues/402) |

## サマリー

Antigravity は **Skills 専用ではない**。公式ドキュメント上、Agents（Custom Subagents）、Commands 相当（Workflows）、Instructions 相当（Rules / `GEMINI.md` / `AGENTS.md`）はいずれも **ファイルベースで公式サポートされている**。

**方針: 3 コンポーネントとも PLM 実装対象。** 個別の `[feature/antigravity]` Issue を起票して実装する。現状の「Skills のみ」「Instructions は別途設定」という docs / コードの前提は **時代遅れ** であり、本調査をもって更新する。

| コンポーネント | 公式サポート | PLM 現状 | 方針 |
|----------------|--------------|----------|------|
| Agents | ✅ `agent.md`（Custom Subagents） | ❌ `SUPPORTED = [Skill]` | **実装する** → feature Issue 起票 |
| Commands | ✅ Workflows（`/name` スラッシュ） | ❌ | **実装する**（Commands → Workflows マッピング）→ feature Issue 起票 |
| Instructions | ✅ Rules + `GEMINI.md` / `AGENTS.md` | ❌ | **実装する** → feature Issue 起票 |
| Hooks | ✅（別調査済み） | ❌ | [#309](https://github.com/DIO0550/plugin-manager/issues/309) で追跡（本 Issue 対象外） |

## 調査方法

1. 公式ドキュメント（`antigravity.google/docs/*`）を取得・照合
2. Changelog で `AGENTS.md` / `agent.md` 追加時期を確認
3. 配置パスが公式に未記載の Workflows について、IDE 実機確認系の公開検証（Mete Atamel 2026-07-13）でクロスチェック
4. PLM コード（`src/target/env/antigravity.rs`）と `docs/concepts/targets.md` の現状記載を突合

## 現状（PLM）

```29:32:src/target/env/antigravity.rs
const SUPPORTED: &[ComponentKind] = &[ComponentKind::Skill];

const CAPABILITIES: &[(ComponentKind, ScopeSupport)] =
    &[(ComponentKind::Skill, ScopeSupport::Both)];
```

| 種別 | Personal | Project |
|------|----------|---------|
| Skills | `~/.gemini/antigravity/skills/` | `.agent/skills/` |

`docs/concepts/targets.md` は「Skills専用」「Instructionsは別途設定で管理」と記載。README 対応表も Skills のみ「対応」。

## 調査結果

### 1. Agents（Custom Subagents）— 公式サポートあり

**出典:** [Subagents](https://antigravity.google/docs/subagents)、[/agents コマンド](https://antigravity.google/docs/cli/commands/agents)、[Changelog](https://antigravity.google/changelog)（`agent.md` 追加）

| 項目 | 内容 |
|------|------|
| 形式 | Markdown。推奨ファイル名 `agent.md`（YAML frontmatter + 本文が system prompt） |
| Personal | `~/.gemini/config/agents/<name>/agent.md` |
| Project | `.agents/agents/<name>/agent.md`（または `.agents/agents/<name>.md`） |
| 自動読み込み | ✅ ディレクトリ走査で発見 |
| Claude Code 互換 | **低い** — PLM の `*.agent.md` とはファイル名・配置階層・frontmatter キーが異なる |

#### Frontmatter（公式）

| フィールド | 必須 | 備考 |
|------------|------|------|
| `name` | ✅ | 一意 ID |
| `description` | ✅ | 委譲判断用 |
| `tools` | | 許可ツール一覧（誤記で hang する既知問題あり） |
| `mainAgent` / `subagent` | | 既定 `true` |
| `model` | | `inherit` / `flash` / `pro` |
| `commandExecutionPolicy` | | `off` / `auto` / `eager` / `sandbox` |
| `mcpServers` / `skills` / `plugins` | | 任意 |

#### PLM への示唆

- `ComponentKind::Agent` として扱うのが妥当（Cursor と同様、サブエージェント定義を Agents にマップ）
- 配置時は **`*.agent.md` → `<name>/agent.md`（または `<name>.md`）へのリネーム** が必要
- frontmatter 変換（`tools` 名の写像、`model` 既定など）は feature Issue で詳細設計
- Personal / Project ともファイル配置のみ（追加設定不要）

### 2. Commands（Workflows）— 公式サポートあり

**出典:** [Rules & Workflows](https://antigravity.google/docs/rules-workflows)、[IDE Workflows](https://antigravity.google/docs/ide/workflows)

公式は「Markdown の Workflow。`/workflow-name` で呼び出し」と明記。**配置パスは公式ページに明示されていない**ため、IDE Customizations UI が書き出す実パスを公開検証した二次ソースで確定する:

**パス出典（クロスチェック）:** [Where does Antigravity look for Rules and Workflows? (Mete Atamel, 2026-07-13)](https://atamel.dev/posts/2026/07-13_where_agy_rules_workflows/) — AGY / AGY CLI / AGY IDE 共通

| 項目 | 内容 |
|------|------|
| 形式 | Markdown（title / description / 手順）。上限 12,000 文字/ファイル |
| Personal | `~/.gemini/config/global_workflows/<name>.md` |
| Project | `.agents/workflows/<name>.md` |
| 呼び出し | `/<filename-stem>` スラッシュコマンド |
| 自動読み込み | ✅（IDE・AGY は `/` 一覧表示。CLI は読み込みはあるが `/` トリガ UI が弱い） |

#### PLM への示唆

- PLM の `ComponentKind::Command` を **Antigravity Workflows にマップ**する（Copilot Prompt / Cursor Commands と同レイヤ）
- Claude Code `.prompt.md` → Workflow Markdown への変換が必要（frontmatter `description` + 本文を手順化）
- 実装前に IDE で `global_workflows` / `.agents/workflows` の実在を再確認することを feature Issue の受け入れ条件に含める（公式 docs 未記載リスク）

### 3. Instructions（Rules / context files）— 公式サポートあり

**出典:** [Rules](https://antigravity.google/docs/rules-workflows)、[CLI Best Practices](https://antigravity.google/docs/cli/best-practices)、[Gemini CLI → Antigravity CLI 移行](https://antigravity.google/docs/cli/gcli-migration)、Changelog（「AGENTS.md in addition to GEMINI.md」）

Antigravity の「指示」は複数レイヤがある:

| レイヤ | Personal | Project | 備考 |
|--------|----------|---------|------|
| Global Rules | `~/.gemini/GEMINI.md`（単一ファイル） | — | 公式 Rules ドキュメント |
| Workspace Rules | — | `.agents/rules/*.md` | 活性化: Manual / Always On / Model Decision / Glob。`.agent/rules` 後方互換 |
| Root context | — | ワークスペース直下の `GEMINI.md` / `AGENTS.md` | CLI best practices・移行ガイド・Changelog で明記 |

#### PLM への示唆（推奨方針）

| スコープ | 推奨配置 | 理由 |
|----------|----------|------|
| Personal | `~/.gemini/GEMINI.md` | 公式 Global Rules。Gemini CLI ターゲットと **同一パスを共有**（#402 レガシー並立時に注意） |
| Project | ルート `AGENTS.md`（第一候補） | Codex / Cursor と共有可能。公式も `AGENTS.md` を読む |
| Project 代替 | `.agents/rules/<name>.md` | 複数ルール・活性化モードが必要ならこちら。PLM Instruction 1 ファイルモデルとは別設計 |

**Phase 1 推奨:** Gemini CLI と同様に Personal=`GEMINI.md`、Project=`AGENTS.md`（または `GEMINI.md`）の **単一ファイル Instruction** から始める。`.agents/rules/` の複数ルール配置は Phase 2（別 Issue）でよい。

> #402 本文の「Antigravity Instructions = 非対応（IDE 設定）」は本調査により **誤り**。ドキュメント更新時は #402 側も本結論に合わせる。

### 4. 付随発見: Skills 配置パスのドリフト（本 Issue 対象外だが重要）

公式 Skills ドキュメントの正は:

| スコープ | 公式（現行） | PLM 現状 |
|----------|--------------|----------|
| Personal | `~/.gemini/config/skills/`（3 flavour 共通で認識） | `~/.gemini/antigravity/skills/` |
| Project | `.agents/skills/`（`.agent/skills` は後方互換） | `.agent/skills/` |

`~/.gemini/antigravity/skills/` は AGY / AGY IDE では今も読めるが、**AGY CLI では読まれない**（[検証記事](https://atamel.dev/posts/2026/07-01_where_agy_agent_skills/)）。#402（gemini レガシー化）および別途 `Skills パス移行` feature で扱うことを推奨。本 Issue の Agents/Commands/Instructions 実装とは分離する。

## 方針決定

Issue 本文の提案ツリーに対する結論:

| 提案分岐 | 判定 |
|----------|------|
| **公式サポートあり → feature Issue を個別起票** | **採用**（Agents / Commands / Instructions の 3 本） |
| 公式サポートなし / IDE 設定のみ → 対象外と明記 | 不採用（公式ファイル配置が確認できた） |
| 将来追加予定 → Issue を open 維持 | 不採用（既に公式サポート済み。本 research Issue は docs 反映後クローズ可） |

### 起票推奨 feature Issue（案）

実装 Issue の作成はリポジトリ管理者が行う（本エージェントの `gh` は read-only）。以下を起票テンプレートとして使う。

#### A. `[feature/antigravity] Agents（Custom Subagents / agent.md）対応`

- `supported_components` に `Agent` 追加
- 配置: Personal `~/.gemini/config/agents/<name>/agent.md`、Project `.agents/agents/<name>/agent.md`
- Claude Code / Codex `.agent.md` → Antigravity frontmatter + `agent.md` 変換
- テスト: placement / list / convert

#### B. `[feature/antigravity] Commands（Workflows）対応`

- `Command` → Workflow Markdown マッピング
- 配置: Personal `~/.gemini/config/global_workflows/<name>.md`、Project `.agents/workflows/<name>.md`
- 受け入れ条件: IDE でパス実在を再確認してからマージ
- CLI での `/` 起動制限を docs に注記

#### C. `[feature/antigravity] Instructions（GEMINI.md / AGENTS.md）対応`

- Phase 1: Personal `~/.gemini/GEMINI.md`、Project ルート `AGENTS.md`（または `GEMINI.md`）
- Codex / Cursor / Gemini CLI との共有パス衝突は既存 Instruction 方針に追随
- Phase 2（任意）: `.agents/rules/` 複数ルール

### Epic 化の是非

Cursor Epic #356 と同型の `[epic/antigravity] 非 Skill コンポーネント対応` で A/B/C + Skills パス移行 + #309 Hooks を束ねると追跡しやすい。必須ではない。

## docs 更新（本 PR）

1. `docs/concepts/targets.md` — Antigravity セクションを公式仕様ベースに書き換え、対応表の脚注を「公式サポートあり・PLM 未実装（#400）」に変更
2. `docs/concepts/components.md` — サポート表の脚注を同期
3. `docs/roadmap.md` — 公式リンクを現行 docs に更新
4. README 対応表は **実装完了まで Skills のみ「対応」のまま**（未実装を「対応」にしない）。脚注または Issue リンクで調査完了を示す程度にとどめる

## クローズ条件（#400）

- [x] Agents / Commands / Instructions の公式有無と配置パス・形式を整理した（本ドキュメント）
- [x] PLM 方針を決定した（3 種とも実装対象）
- [x] `docs/concepts/targets.md` を更新（本 PR）
- [ ] feature Issue A/B/C を起票（メンテナ作業）
- [ ] 起票後に #400 をクローズ（または Epic へロールアップ）

## 参考リンク

- [Skills](https://antigravity.google/docs/skills)
- [Rules & Workflows](https://antigravity.google/docs/rules-workflows)
- [Subagents](https://antigravity.google/docs/subagents)
- [CLI Best Practices（GEMINI.md / AGENTS.md）](https://antigravity.google/docs/cli/best-practices)
- [gcli-migration](https://antigravity.google/docs/cli/gcli-migration)
- [Rules/Workflows パス検証（Atamel）](https://atamel.dev/posts/2026/07-13_where_agy_rules_workflows/)
- [Skills パス検証（Atamel）](https://atamel.dev/posts/2026/07-01_where_agy_agent_skills/)
