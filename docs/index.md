# PLM - Plugin Manager CLI

GitHubからAI開発環境向けのプラグインをダウンロードし、複数のAI環境を統一的に管理するRust製CLIツール

## 概要

### 背景

- Claude CodeはPluginとマーケットプレイスでskills, agents, commands, hooksを統合管理
- OpenAI CodexやVSCode CopilotもAgent Skills仕様に対応
- Claude Code以外にはマーケットプレイス機能がない
- GitHubからプラグインコンポーネントをダウンロードして管理する統一CLIが必要

### 目標

- GitHubベースのマーケットプレイスからプラグインをインストール
- プラグイン内のコンポーネントを自動的にCodex/Copilotへ展開
- TUI管理画面で直感的な操作を提供
- 詳細なプラグインメタデータの保持

## 特徴

| 機能 | 説明 |
|------|------|
| マルチターゲット対応 | OpenAI Codex、VSCode Copilotに対応 |
| 統一管理 | 複数環境のプラグインを一元管理 |
| 自動展開 | コンポーネント種別に応じて適切な場所へ配置 |
| マーケットプレイス | GitHubリポジトリをマーケットプレイスとして登録 |
| TUI管理画面 | インタラクティブな管理インターフェース |

## ドキュメント構成

- [Getting Started](./getting-started.md) - 初期設定・クイックスタート
- [Commands](./commands/index.md) - コマンドリファレンス
- [Concepts](./concepts/targets.md) - コンセプト・仕組み
- [Architecture](./architecture/overview.md) - 内部アーキテクチャ
- [Reference](./reference/config.md) - 技術リファレンス
- [Roadmap](./roadmap.md) - 実装状況・ロードマップ

## 対応規格

| 規格 | 説明 | 参照 |
|------|------|------|
| **AGENTS.md** | カスタム指示ファイル（Linux Foundation管轄のオープン標準） | https://agents.md |
| **SKILL.md** | スキル定義（Anthropicがオープン標準として公開、OpenAI/Microsoftが採用） | - |

## 参考リンク

### Agent Skills

- [Agent Skills Specification](https://github.com/anthropics/skills)
- [Skills Marketplace](https://skillsmp.com)

### OpenAI Codex

- [Codex Skills](https://developers.openai.com/codex/skills/)
- [AGENTS.md Guide](https://developers.openai.com/codex/guides/agents-md/)

### VSCode Copilot

- [Custom Instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Custom Agents](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/create-custom-agents)
- [Prompt Files](https://code.visualstudio.com/docs/copilot/customization/overview)

### Claude Code

- [Plugins Documentation](https://code.claude.com/docs/en/plugins)
- [Skills Documentation](https://code.claude.com/docs/en/skills)
- [anthropics/claude-code plugins](https://github.com/anthropics/claude-code/tree/main/plugins)
