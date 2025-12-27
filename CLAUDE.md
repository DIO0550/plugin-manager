# CLAUDE.md

このファイルはClaude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

PLM (Plugin Manager CLI) は、複数のAI開発環境（OpenAI Codex、VSCode Copilot、Claude Code）のプラグインを統合管理するRust製CLIツールです。Skills、Agents、Prompts、Instructionsのダウンロード、インストール、同期を行います。

## ビルドコマンド

```bash
# ビルド
cargo build
cargo build --release

# 実行
cargo run -- <command>

# チェック（バイナリ生成なしの高速コンパイル）
cargo check

# フォーマット
cargo fmt

# リント
cargo clippy

# テスト実行
cargo test
cargo test <test_name>  # 単一テスト実行

# 依存関係セキュリティ監査
cargo deny check
```

## アーキテクチャ

### エントリポイント
- `src/main.rs` - tokioランタイムを使用した非同期エントリポイント
- `src/cli.rs` - Clapベースの12コマンドを持つCLI定義

### コマンドディスパッチパターン
コマンドは `src/commands/mod.rs` を経由して各ハンドラモジュールにルーティングされる：
- `target.rs` - ターゲット環境管理（codex/copilot）
- `install.rs` - GitHubからコンポーネントをインストール
- `list.rs`, `info.rs` - インストール済みコンポーネントの照会
- `enable.rs`, `disable.rs`, `uninstall.rs` - コンポーネント状態管理
- `update.rs` - コンポーネント更新
- `init.rs`, `pack.rs` - コンポーネント作成とパッケージング
- `sync.rs` - 環境間同期
- `import.rs` - Claude Code Pluginsからインポート

### 計画中のモジュール構成（docs/plm-plan-v2.md参照）
- `targets/` - Target traitを実装する環境アダプター
- `components/` - Component traitを実装するコンポーネントタイプハンドラー
- `registry/` - `components.json`による状態管理
- `github/` - GitHub API連携
- `parser/` - ファイル形式パーサー（SKILL.md, .agent.md, plugin.json）
- `config.rs` - 設定管理（`~/.plm/config.toml`）

### コア設計パターン

**Target Trait** - 環境差異を抽象化：
| 環境 | Skills | Agents | Prompts | Instructions |
|------------|--------|--------|---------|--------------|
| OpenAI Codex | ○ | × | × | ○ |
| VSCode Copilot | × | ○ | ○ | ○ |

**Component Trait** - コンポーネントタイプを抽象化：
- Skills: YAMLフロントマター付き `SKILL.md`
- Agents: `.agent.md` または `AGENTS.md`
- Prompts: `.prompt.md`
- Instructions: `copilot-instructions.md`

**スコープ** - Personal（`~/.codex/`, `~/.copilot/`）vs Project（`.codex/`, `.github/`）

## 主要依存関係
- `clap` v4 - deriveマクロによるCLIパース
- `tokio` - 非同期ランタイム
- `reqwest` - GitHub API用HTTPクライアント
- `serde`, `serde_json`, `toml`, `serde_yaml` - シリアライゼーション
- `owo-colors`, `indicatif`, `comfy-table` - ターミナルUI

## 仕様ドキュメント

詳細な仕様・実装計画は `docs/` フォルダを参照：
- `docs/plm-plan-v2.md` - 最新の実装計画（設定・レジストリのデータ構造、コンポーネントファイル形式、フェーズ別実装内訳）
- `docs/old/` - 過去のドキュメント
