# ターゲット環境

PLMがサポートするAI開発環境（ターゲット）について説明します。

## 対応ターゲット

| ターゲット | 説明 |
|------------|------|
| **codex** | OpenAI Codex CLI |
| **copilot** | VSCode GitHub Copilot |
| **antigravity** | Google Antigravity IDE |
| **gemini** | Gemini CLI（ターミナルベースAIエージェント） |

## サポートするコンポーネント

| コンポーネント | Codex | Copilot | Antigravity | Gemini CLI |
|----------------|-------|---------|-------------|------------|
| Skills | ✅ | ✅ | ✅ | ✅ |
| Agents | ✅ | ✅ | ❌ | ❌ |
| Commands | ❌ | ✅ | ❌ | ❌ |
| Instructions | ✅ | ✅ | ❌* | ✅** |
| Hooks | ❌ | ❌ | ❌ | ❌ |

> *AntigravityはSkills専用の設計で、Instructionsは別途設定で管理します。
> **Gemini CLIは`GEMINI.md`による階層的な指示システムを持ちます。

## OpenAI Codex

### 読み込みパスと優先順位

公式ドキュメント: [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global (override) | `~/.codex/AGENTS.override.md` | ✅ | 最優先 |
| Global | `~/.codex/AGENTS.md` | ✅ | Personal対応 |
| Project | `./AGENTS.override.md` | ✅ | ディレクトリ毎 |
| Project | `./AGENTS.md` | ✅ | ディレクトリ毎 |
| Skills (Global) | `~/.codex/skills/` | ✅ | Personal |
| Skills (Project) | `./.codex/skills/` | ✅ | Project |

### 読み込み順序

1. **Global scope**: `~/.codex/` (または `$CODEX_HOME`) をチェック
   - `AGENTS.override.md` があればそれを使用、なければ `AGENTS.md`
2. **Project scope**: リポジトリルートから現在ディレクトリまで走査
   - 各ディレクトリで `AGENTS.override.md` → `AGENTS.md` → fallback の順
3. **マージ**: ルートから現在ディレクトリに向かって連結（上限: `project_doc_max_bytes` = 32KiB）

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.codex/skills/<marketplace>/<plugin>/<skill>/` | `.codex/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `*.agent.md` | `~/.codex/agents/<marketplace>/<plugin>/` | `.codex/agents/<marketplace>/<plugin>/` |
| Instructions | `AGENTS.md` | `~/.codex/AGENTS.md` | `AGENTS.md` |

## VSCode GitHub Copilot

### 読み込みパスと優先順位

公式ドキュメント: [Use custom instructions in VS Code](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Project | `.github/copilot-instructions.md` | ✅ | メインの指示ファイル |
| Project | `.github/instructions/*.instructions.md` | ❌ | 手動指定が必要 |
| User | VSCode設定の `file` プロパティ | ✅ | 設定で外部ファイル参照 |
| Prompts | `.github/prompts/*.prompt.md` | ❌ | 手動呼び出し |

### 重要な制約

- **Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込まない**
- Personal スコープは VSCode 設定経由で外部ファイルを参照する形式
- Issue: [Global files outside workspace の要望](https://github.com/microsoft/vscode-copilot-release/issues/3129)

### VSCode設定での外部ファイル参照

```json
// settings.json (User または Workspace)
{
  "github.copilot.chat.codeGeneration.instructions": [
    {
      "file": "/path/to/personal-instructions.md"
    }
  ],
  "github.copilot.chat.codeGeneration.useInstructionFiles": true
}
```

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | - | `.github/skills/<marketplace>/<plugin>/<skill>/` |
| Agents | `*.agent.md` | `~/.copilot/agents/<marketplace>/<plugin>/` | `.github/agents/<marketplace>/<plugin>/` |
| Prompts | `*.prompt.md` | - | `.github/prompts/<marketplace>/<plugin>/` |
| Instructions | `AGENTS.md` | - | `AGENTS.md` |
| Instructions | `copilot-instructions.md` | - | `.github/copilot-instructions.md` |

## Google Antigravity

### 概要

Google AntigravityはGemini 3 Pro搭載のエージェント指向IDE。2026年1月13日にAnthropicのAgent Skills open standard（SKILL.md形式）を正式採用。

公式ドキュメント:
- [Getting Started with Google Antigravity](https://codelabs.developers.google.com/getting-started-google-antigravity)
- [Authoring Google Antigravity Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)

### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global | `~/.gemini/antigravity/skills/` | ✅ | Personal対応 |
| Workspace | `<workspace-root>/.agent/skills/` | ✅ | Project対応 |

### 重要な特徴

- **ディレクトリベースのSkillsパッケージ**: 各Skillは独立したディレクトリとして管理
- **Progressive Disclosure**: Skillは必要時のみコンテキストにロードされる（コンテキスト肥大化を防止）
- **SKILL.md形式**: Anthropic発祥のAgent Skills open standardを採用

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.gemini/antigravity/skills/<marketplace>/<plugin>/<skill>/` | `.agent/skills/<marketplace>/<plugin>/<skill>/` |

### 制約事項

- **Skills専用**: Agents、Prompts、Instructionsは別のシステムで管理
- Skillsはタスク終了後にコンテキストから解放される（エフェメラル）

## Gemini CLI

### 概要

Gemini CLIはGoogleのターミナルベースAIエージェントツール。v0.23.0（2026年1月7日）でAgent Skills（実験的機能）が追加された。Claude Code Skillsと同じ`SKILL.md`形式を採用しており、既存のSkillsをそのまま再利用可能。

公式ドキュメント:
- [Agent Skills | Gemini CLI](https://geminicli.com/docs/cli/skills/)
- [Getting Started with Agent Skills](https://geminicli.com/docs/cli/tutorials/skills-getting-started/)

### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Workspace | `.gemini/skills/` | ✅ | プロジェクト固有、VCS管理推奨 |
| User | `~/.gemini/skills/` | ✅ | 個人用、全ワークスペースで利用可能 |
| Extension | 拡張機能に同梱 | ✅ | 拡張機能パッケージ内 |
| Instructions (Global) | `~/.gemini/GEMINI.md` | ✅ | 全プロジェクト共通の指示 |
| Instructions (Project) | `./GEMINI.md` | ✅ | 親ディレクトリまで走査 |

### 優先順位

同名Skillが複数スコープに存在する場合: Workspace > User > Extension

### Skills のアクティベーション

Gemini CLI SkillsはProgressive Disclosure方式を採用:

1. **Discovery**: セッション開始時にSkillの名前と説明のみをシステムプロンプトに注入
2. **Activation**: タスクにマッチするSkillを検出すると `activate_skill` ツールを呼び出す
3. **Consent**: ユーザーにSkill名・目的・ディレクトリパスを表示して確認を求める
4. **Injection**: `SKILL.md` の本文とフォルダ構造を会話に追加
5. **Execution**: 専門知識がアクティブな状態でタスクを実行

### 管理コマンド

**セッション内** (`/skills`):
- `/skills list` - 発見されたSkill一覧
- `/skills disable <name>` - Skillを無効化
- `/skills enable <name>` - Skillを再有効化
- `/skills reload` - Skill検出を再実行

**ターミナル** (`gemini skills`):
- `gemini skills list` - 全Skill表示
- `gemini skills install <source>` - Skill追加（Gitリポジトリ、ローカルパス、`.skill`ファイル対応）
- `gemini skills uninstall <name>` - Skill削除
- `gemini skills enable/disable <name>` - 有効/無効切替

### Instructions システム（GEMINI.md）

Gemini CLIは `GEMINI.md` ファイルによる階層的な指示システムを持つ:

- **Global**: `~/.gemini/GEMINI.md` - 全プロジェクト共通の指示
- **Project**: カレントディレクトリからプロジェクトルート（`.git`フォルダ）まで走査し、各ディレクトリの `GEMINI.md` を連結
- **ファイル名設定**: `.gemini/settings.json` で `contextFileName` を変更可能（例: `"contextFileName": "AGENTS.md"`）
- **モジュラーインポート**: `@file.md` 構文で他ファイルの内容をインポート可能

### コンポーネント配置場所

| 種別 | ファイル形式 | Personal | Project |
|------|-------------|----------|---------|
| Skills | `SKILL.md` | `~/.gemini/skills/<marketplace>/<plugin>/<skill>/` | `.gemini/skills/<marketplace>/<plugin>/<skill>/` |
| Instructions | `GEMINI.md` | `~/.gemini/GEMINI.md` | `GEMINI.md` |

### 制約事項

- **実験的機能**: `/settings` で Agent Skills を `true` に設定して有効化が必要
- **Agents非対応**: `.agent.md` 形式はサポートしない
- **Prompts非対応**: `.prompt.md` 形式はサポートしない

## PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | 不要（自動読み込み） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |
| Antigravity | `~/.gemini/antigravity/` に配置 | 不要（自動読み込み） |
| Gemini CLI | `~/.gemini/skills/` に配置 | 不要（自動読み込み、要Settings有効化） |

## 将来の拡張候補

- Cursor（.cursor/）
- Windsurf
- Aider
- その他SKILL.md対応ツール

## 関連

- [concepts/components](./components.md) - コンポーネント種別
- [concepts/scopes](./scopes.md) - Personal/Projectスコープ
- [commands/target](../commands/target.md) - ターゲット管理コマンド
