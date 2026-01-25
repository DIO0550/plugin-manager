# CLAUDE.md

このファイルはClaude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

PLM (Plugin Manager CLI) は、複数のAI開発環境（OpenAI Codex、VSCode Copilot、Google Antigravity、Claude Code）のプラグインを統合管理するRust製CLIツールです。Skills、Agents、Prompts、Instructionsのダウンロード、インストール、同期を行います。

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
- `target.rs` - ターゲット環境管理（codex/copilot/antigravity）
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
- `parser/` - ファイル形式パーサー・変換（詳細は `docs/architecture/file-formats.md` 参照）
- `config.rs` - 設定管理（`~/.plm/config.toml`）

### コア設計パターン

**Target Trait** - 環境差異を抽象化：
| 環境 | Skills | Agents | Prompts | Instructions |
|------------|--------|--------|---------|--------------|
| OpenAI Codex | ○ | × | × | ○ |
| VSCode Copilot | ○ | ○ | ○ | ○ |
| Google Antigravity | ○ | × | × | × |

**Component Trait** - コンポーネントタイプを抽象化：
- Skills: YAMLフロントマター付き `SKILL.md`
- Agents: `.agent.md` または `AGENTS.md`
- Prompts: `.prompt.md`
- Instructions: `copilot-instructions.md`

**スコープ** - Personal（`~/.codex/`, `~/.copilot/`, `~/.gemini/antigravity/`）vs Project（`.codex/`, `.github/`, `.agent/`）

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
│   ├── antigravity.rs # Antigravity ターゲット実装
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

### テストコードの分離

テストコードは本体コードと同じファイルに書かない。`foo.rs` のテストは `foo_test.rs` に分離する。

## 開発プロセス

### TDD（テスト駆動開発）

新機能の実装やバグ修正は、TDDの **Red → Green → Refactor** サイクルで進める。

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│    ┌───────┐      ┌───────┐      ┌──────────┐     │
│    │  Red  │ ───→ │ Green │ ───→ │ Refactor │     │
│    └───────┘      └───────┘      └──────────┘     │
│        ↑                              │            │
│        └──────────────────────────────┘            │
│                                                     │
└─────────────────────────────────────────────────────┘
```

1. **Red（レッド）**: まず失敗するテストを書く
   - 実装したい振る舞いをテストコードで表現する
   - テストを実行し、**失敗することを確認**する（これが重要）
   - 失敗を確認せずに実装に進まない

2. **Green（グリーン）**: テストを通す最小限のコードを書く
   - テストをパスさせることだけに集中する
   - 完璧なコードを書こうとしない
   - "仮実装"や"明白な実装"で素早くグリーンにする

3. **Refactor（リファクタ）**: テストが通る状態を保ちながらコードを改善する
   - 重複を除去する
   - 命名を改善する
   - 設計を洗練させる
   - **テストが通り続けることを常に確認**する

**原則:**
- 小さなステップで進める（一度に大きな変更をしない）
- 各ステップでテストを実行する
- Redを確認せずにGreenに進まない（テストが正しく失敗することを確認する）
- リファクタリング中は機能追加しない

## 仕様ドキュメント

詳細な仕様・実装計画は `docs/` フォルダを参照：
- `docs/architecture/file-formats.md` - ファイルフォーマット仕様・変換マッピング
- `docs/old/` - 過去のドキュメント

※ 仕様ドキュメントのバージョンを上げる際は、古いバージョンを `docs/old/` に移動すること

## コミットメッセージ規約

コミットメッセージは英語で書く。
