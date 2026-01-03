# PLM - Plugin Manager CLI

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

AIコーディングアシスタント（OpenAI Codex、VSCode Copilot）のプラグインを統合管理するRust製CLIツールです。Claude Code Pluginsをインポートして他の環境にデプロイできます。Skills、Agents、Prompts、Instructionsのダウンロード、インストール、同期をシームレスに行えます。

[English README](README.md)

## 特徴

- **マルチ環境対応**: OpenAI CodexとVSCode Copilotにプラグインを単一ツールでデプロイ
- **Claude Code Pluginインポート**: 既存のClaude Code Pluginsをインポートして他の環境で使用
- **コンポーネントタイプ**: Skills、Agents、Prompts、Instructionsに対応
- **マーケットプレイス連携**: マーケットプレイスからプラグインを検索・インストール
- **スコープ管理**: 個人レベル（`~/.codex/`、`~/.copilot/`）またはプロジェクトレベル（`.codex/`、`.github/`）でインストール
- **TUIインターフェース**: 対話型ターミナルUIでプラグインを管理
- **環境間同期**: 異なる環境間でプラグインを同期

## インストール

### ソースからビルド

```bash
git clone https://github.com/your-org/plugin-manager.git
cd plugin-manager
cargo build --release
```

バイナリは `target/release/plm` に生成されます。

### 必要要件

- Rust 2021 エディション以降
- Git（プラグインダウンロード用）

## クイックスタート

```bash
# GitHubからプラグインをインストール
plm install owner/repo

# ターゲットとスコープを指定してインストール
plm install owner/repo --target codex --scope personal

# インストール済みプラグイン一覧
plm list

# TUIでプラグイン管理
plm managed
```

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `target` | ターゲット環境の管理（codex/copilot） |
| `marketplace` | マーケットプレイスの管理 |
| `install` | マーケットプレイスまたはGitHubからプラグインをインストール |
| `list` | インストール済みコンポーネント一覧 |
| `info` | コンポーネントの詳細表示 |
| `enable` | コンポーネントを有効化 |
| `disable` | コンポーネントを無効化 |
| `uninstall` | コンポーネントを削除 |
| `update` | コンポーネントを更新 |
| `init` | テンプレート生成 |
| `pack` | 配布用パッケージ作成 |
| `sync` | 環境間同期 |
| `import` | Claude Code Pluginからインポート |
| `managed` | TUIでプラグイン管理 |

## 使用例

### ターゲット管理

```bash
# 設定済みターゲット一覧
plm target list

# ターゲット環境を追加
plm target add codex

# ターゲット環境を削除
plm target remove copilot
```

### マーケットプレイス管理

```bash
# 登録済みマーケットプレイス一覧
plm marketplace list

# マーケットプレイスを追加
plm marketplace add owner/marketplace-repo

# マーケットプレイスのキャッシュを更新
plm marketplace update
```

### プラグインのインストール

```bash
# GitHubリポジトリからインストール
plm install owner/repo

# 特定のコンポーネントタイプのみインストール
plm install owner/repo --type skills --type agents

# 特定のターゲットにインストール
plm install owner/repo --target codex --target copilot

# プロジェクトスコープでインストール
plm install owner/repo --scope project

# マーケットプレイスからインストール
plm install plugin-name@marketplace-name

# 強制再ダウンロード（キャッシュ無視）
plm install owner/repo --force
```

### コンポーネント管理

```bash
# インストール済みコンポーネント一覧
plm list

# コンポーネントの詳細表示
plm info component-name

# コンポーネントの有効化/無効化
plm enable component-name
plm disable component-name

# コンポーネントの削除
plm uninstall component-name

# コンポーネントの更新
plm update
```

### プラグインの作成

```bash
# 新規プラグインテンプレートを生成
plm init

# 配布用パッケージを作成
plm pack
```

## 対応環境

| 環境 | Skills | Agents | Prompts | Instructions |
|------|:------:|:------:|:-------:|:------------:|
| OpenAI Codex | 対応 | - | - | 対応 |
| VSCode Copilot | - | 対応 | 対応 | 対応 |

## 設定

PLMの設定は `~/.plm/config.toml` に保存されます。

### コンポーネントレジストリ

インストール済みコンポーネントは各スコープの `components.json` で追跡されます。

## 開発

```bash
# ビルド
cargo build

# テスト実行
cargo test

# チェック（バイナリ生成なしの高速コンパイル）
cargo check

# フォーマット
cargo fmt

# リント
cargo clippy
```

## サードパーティライセンス

`cargo-about` を使用してサードパーティライセンス一覧を生成できます：

```bash
cargo install --locked cargo-about
cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs
```

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。
