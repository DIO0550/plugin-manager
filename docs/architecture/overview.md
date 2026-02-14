# アーキテクチャ概要

PLMの内部アーキテクチャについて説明します。

## ディレクトリ構成

```
plm/
├── Cargo.toml
├── src/
│   ├── main.rs                   # tokio非同期エントリポイント
│   ├── cli.rs                    # Clap CLI定義（16コマンド）
│   ├── commands.rs               # コマンドディスパッチャー
│   ├── commands/
│   │   ├── install.rs            # インストール処理
│   │   ├── uninstall.rs          # 削除処理
│   │   ├── list.rs               # 一覧表示
│   │   ├── info.rs               # 詳細情報
│   │   ├── enable.rs             # 有効化
│   │   ├── disable.rs            # 無効化
│   │   ├── update.rs             # 更新処理
│   │   ├── target.rs             # ターゲット環境管理
│   │   ├── marketplace.rs        # マーケットプレイス管理
│   │   ├── init.rs               # テンプレート作成
│   │   ├── pack.rs               # パッケージ化
│   │   ├── link.rs               # シンボリックリンク作成
│   │   ├── unlink.rs             # シンボリックリンク削除
│   │   ├── sync.rs               # 環境間同期
│   │   ├── import.rs             # Claude Plugin インポート
│   │   └── managed.rs            # TUI管理画面起動
│   ├── target.rs                 # Target trait定義
│   ├── target/                   # AI環境アダプター
│   │   ├── antigravity.rs        # Google Antigravity
│   │   ├── codex.rs              # OpenAI Codex
│   │   ├── copilot.rs            # VSCode Copilot
│   │   ├── gemini_cli.rs         # Gemini CLI
│   │   ├── effect.rs             # ターゲット操作結果
│   │   ├── registry.rs           # ターゲットレジストリ
│   │   └── scanner.rs            # ターゲットスキャン
│   ├── component.rs              # コンポーネントモジュール定義
│   ├── component/                # コンポーネント種別
│   │   ├── kind.rs               # ComponentKind enum
│   │   ├── deployment.rs         # デプロイメント情報
│   │   ├── convert.rs            # コンポーネント変換
│   │   ├── placement.rs          # 配置ロジック
│   │   └── summary.rs            # コンポーネントサマリー
│   ├── plugin.rs                 # プラグインモジュール定義
│   ├── plugin/                   # プラグイン管理
│   │   ├── cache.rs              # プラグインキャッシュ管理
│   │   ├── cached_plugin.rs      # キャッシュ済みプラグイン
│   │   ├── manifest.rs           # plugin.json パーサー
│   │   ├── manifest_resolve.rs   # マニフェスト解決
│   │   ├── meta.rs               # プラグインメタデータ
│   │   ├── update.rs             # 更新ロジック
│   │   └── version.rs            # バージョン管理
│   ├── parser.rs                 # パーサーモジュール定義
│   ├── parser/                   # ファイルパーサー
│   │   ├── claude_code.rs        # Claude Code形式
│   │   ├── claude_code_agent.rs  # Claude Code Agent形式
│   │   ├── codex.rs              # Codex形式
│   │   ├── codex_agent.rs        # Codex Agent形式
│   │   ├── copilot.rs            # Copilot形式
│   │   ├── copilot_agent.rs      # Copilot Agent形式
│   │   ├── convert.rs            # フォーマット変換
│   │   └── frontmatter.rs        # YAML frontmatterパーサー
│   ├── source.rs                 # ソースモジュール定義
│   ├── source/                   # プラグインソース
│   │   ├── github_source.rs      # GitHub実装
│   │   ├── marketplace_source.rs # マーケットプレイス実装
│   │   └── search_source.rs      # 検索実装
│   ├── marketplace.rs            # マーケットプレイスモジュール定義
│   ├── marketplace/              # マーケットプレイス
│   │   ├── config.rs             # マーケットプレイス設定
│   │   ├── fetcher.rs            # marketplace.json取得
│   │   ├── plugin_source_path.rs # プラグインソースパス
│   │   ├── registry.rs           # マーケットプレイスレジストリ
│   │   └── windows_path.rs       # Windowsパス処理
│   ├── sync.rs                   # 同期モジュール定義
│   ├── sync/                     # 環境間同期
│   │   ├── action.rs             # 同期アクション
│   │   ├── destination.rs        # 同期先
│   │   ├── options.rs            # 同期オプション
│   │   ├── placed.rs             # 配置済みコンポーネント
│   │   ├── result.rs             # 同期結果
│   │   └── source.rs             # 同期元
│   ├── scan.rs                   # スキャンモジュール定義
│   ├── scan/                     # コンポーネントスキャン
│   │   ├── components.rs         # コンポーネントスキャン
│   │   ├── constants.rs          # スキャン定数
│   │   └── placement.rs          # 配置スキャン
│   ├── import.rs                 # インポートモジュール定義
│   ├── import/                   # Claude Code Pluginインポート
│   │   └── registry.rs           # インポートレジストリ
│   ├── tui.rs                    # TUIモジュール定義
│   ├── tui/                      # TUI管理画面
│   │   ├── dialog.rs             # ダイアログコンポーネント
│   │   ├── manager.rs            # TUIマネージャー
│   │   │   ├── core/             # コア機能
│   │   │   │   ├── app.rs        # アプリケーション状態
│   │   │   │   ├── common.rs     # 共通ユーティリティ
│   │   │   │   ├── data.rs       # データ構造
│   │   │   │   └── filter.rs     # フィルタリング
│   │   │   └── screens/          # 画面
│   │   │       ├── discover.rs   # マーケットプレイス検索
│   │   │       ├── errors.rs     # エラー一覧
│   │   │       ├── installed/    # インストール済み管理
│   │   │       └── marketplaces/ # マーケットプレイス管理
│   │   ├── scope_select.rs       # スコープ選択ダイアログ
│   │   └── target_select.rs      # ターゲット選択ダイアログ
│   ├── application.rs            # アプリケーションサービス層
│   ├── config.rs                 # 設定管理
│   ├── env.rs                    # 環境検出
│   ├── error.rs                  # エラーハンドリング
│   ├── fs.rs                     # ファイルシステム操作
│   ├── http.rs                   # HTTPクライアント
│   ├── output.rs                 # 出力フォーマット
│   ├── path_ext.rs               # パスユーティリティ
│   └── repo.rs                   # リポジトリ参照
└── README.md
```

## モジュール構成方針

Featureベースのモジュール構成を採用。レイヤーベース（domain/, application/, infrastructure/）ではなく、関連する機能を同じモジュール/フォルダにまとめます。

```
src/
├── target/           # Target 関連の全て
│   ├── antigravity.rs # Antigravity ターゲット実装
│   ├── codex.rs      # Codex ターゲット実装
│   ├── copilot.rs    # Copilot ターゲット実装
│   ├── gemini_cli.rs # Gemini CLI ターゲット実装
│   ├── effect.rs     # ターゲット操作の結果
│   ├── registry.rs   # ターゲットレジストリ
│   └── scanner.rs    # ターゲットスキャン
├── plugin/           # Plugin 関連の全て
│   ├── cache.rs      # キャッシュ管理
│   ├── cached_plugin.rs # キャッシュ済みプラグイン
│   ├── manifest.rs   # マニフェスト
│   ├── update.rs     # 更新ロジック
│   └── version.rs    # バージョン管理
└── component/        # Component 関連の全て
    ├── kind.rs       # コンポーネント種別
    ├── deployment.rs # デプロイメント
    ├── convert.rs    # コンポーネント変換
    └── placement.rs  # 配置ロジック
```

## 依存クレート

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# 非同期
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.12", features = ["json", "stream"] }

# シリアライズ
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
serde_yaml = "0.9"

# ファイル操作
zip = "2"
dirs = "5"
walkdir = "2"
glob = "0.3"

# TUI
ratatui = "0.29"
crossterm = "0.28"

# ターミナルUI
owo-colors = "4"
indicatif = "0.17"
comfy-table = "7"

# その他
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
regex = "1"
```

## 処理フロー

### インストールフロー

```
1. plm install owner/repo@v1.0.0
2. GitRepo::parse("owner/repo@v1.0.0")
3. repo.github_zipball_url("v1.0.0") でダウンロード
4. ~/.plm/cache/plugins/<marketplace>/<name>/ に展開
5. plugin.json パース
6. デプロイ先の競合チェック
7. ターゲットへ自動展開
8. CachedPlugin作成
9. plugins.json に保存
```

### TUI表示フロー

```
1. plm managed
2. PluginCache::load() で plugins.json 読み込み
3. 一覧表示（ネットワーク不要）
4. 選択時: CachedPlugin.git_repo() で GitRepo 復元
5. "View on GitHub": repo.github_web_url() でブラウザ起動
```

### 更新フロー

```
1. TUIで "Update now" 選択
2. CachedPlugin.git_repo() で GitRepo 復元
3. repo.github_commit_url("HEAD") で最新SHA取得
4. installed_sha と比較
5. 差分あれば repo.github_zipball_url() でダウンロード
6. 再展開
7. CachedPlugin更新、plugins.json 保存
```

## 関連

- [core-design](./core-design.md) - コア設計（Traits, 構造体）
- [cache](./cache.md) - キャッシュアーキテクチャ
- [tui](./tui.md) - TUI設計
- [file-formats](./file-formats.md) - ファイルフォーマット仕様・変換マッピング
