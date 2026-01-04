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

### 計画中のモジュール構成（docs/plm-plan-v3.md参照）
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
| VSCode Copilot | ○ | ○ | ○ | ○ |

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
- `ratatui`, `crossterm` - TUI（管理画面用）

## Rust コーディング規約

### モジュール構成（Rust 2018+ スタイル）

`mod.rs` は使用しない。Rust 2018 エディション以降の新しいモジュールスタイルを使用する：

```
# ✗ 古いスタイル（使用禁止）
src/
├── source/
│   ├── mod.rs        # ← これは使わない
│   └── github.rs

# ✓ 新しいスタイル（推奨）
src/
├── source.rs         # mod source の定義
├── source/
│   └── github.rs     # source::github
```

例：`source` モジュールを作る場合
```rust
// src/source.rs （モジュールのルート）
mod github;
pub use github::GitHubSource;

// src/source/github.rs （サブモジュール）
pub struct GitHubSource { ... }
```

### モジュール構成方針（Feature ベース）

レイヤーベース（domain/, application/, infrastructure/）ではなく、
**Feature ベース**のモジュール構成を採用する。

関連する機能は同じモジュール/フォルダにまとめる：

```
src/
├── target/           # Target 関連の全て
│   ├── codex.rs      # Codex ターゲット実装
│   ├── copilot.rs    # Copilot ターゲット実装
│   └── effect.rs     # ターゲット操作の結果（値オブジェクト）
├── plugin/           # Plugin 関連の全て
│   ├── cache.rs      # キャッシュ管理
│   └── manifest.rs   # マニフェスト
└── component/        # Component 関連の全て
    ├── kind.rs       # コンポーネント種別
    └── deployment.rs # デプロイメント
```

**原則:**
- 機能（Feature）単位でモジュールを分ける
- 値オブジェクト、エンティティ、サービスは関連する Feature に配置
- レイヤー分離よりも凝集度を優先

## 仕様ドキュメント

詳細な仕様・実装計画は `docs/` フォルダを参照：
- `docs/plm-plan-v3.md` - 最新の実装計画（TUI管理画面、GitRepo設計、キャッシュアーキテクチャ）
- `docs/old/` - 過去のドキュメント

※ 仕様ドキュメントのバージョンを上げる際は、古いバージョンを `docs/old/` に移動すること

## コミットメッセージ規約
コミットメッセージは、英語にする
Conventional Commits に従い、絵文字プレフィックスを使用する：

```
<emoji> <type>(<scope>): <description>
```

### Type 一覧

| 絵文字 | type | 説明 |
|--------|------|------|
| ✨ | `feat` | 新機能 |
| 🐛 | `fix` | バグ修正 |
| 📚 | `docs` | ドキュメントのみの変更 |
| 🎨 | `style` | フォーマット変更（動作に影響しない） |
| ♻️ | `refactor` | リファクタリング（機能追加・バグ修正なし） |
| 🧪 | `test` | テストの追加・修正 |
| 🔧 | `chore` | ビルドプロセス・補助ツールの変更 |
| ⚡ | `perf` | パフォーマンス改善 |
| 👷 | `ci` | CI設定の変更 |

### 例

```
✨ feat(install): add GitHub repository caching
🐛 fix(parser): handle empty YAML frontmatter
📚 docs: update README with new commands
♻️ refactor(targets): migrate to trait-based design
```

### コミットの分割

関連性のある変更ごとにコミットを分ける：

- **機能単位**: 1つの機能・修正は1つのコミット
- **ドキュメント**: コード変更とドキュメント変更は分ける
- **依存関係**: Cargo.toml の変更は関連するコード変更と一緒にコミット可
- **リファクタリング**: 機能追加とリファクタリングは分ける

```bash
# ✗ 悪い例: 無関係な変更を1つにまとめる
git commit -m "feat: add feature X, fix bug Y, update docs"

# ✓ 良い例: 関連性で分ける
git commit -m "✨ feat(install): add feature X"
git commit -m "🐛 fix(parser): fix bug Y"
git commit -m "📚 docs: update installation guide"
```
