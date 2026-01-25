# アーキテクチャ概要

PLMの内部アーキテクチャについて説明します。

## ディレクトリ構成

```
plm/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs                    # Clap CLI定義
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
│   │   ├── sync.rs               # 環境間同期
│   │   └── import.rs             # Claude Plugin インポート
│   ├── tui/                      # TUI管理画面
│   │   ├── app.rs                # アプリケーション状態
│   │   ├── ui.rs                 # UI描画
│   │   ├── tabs/                 # 各タブ
│   │   │   ├── discover.rs
│   │   │   ├── installed.rs
│   │   │   ├── marketplaces.rs
│   │   │   └── errors.rs
│   │   └── widgets/              # 再利用可能ウィジェット
│   │       └── plugin_select.rs  # プラグイン選択ダイアログ
│   ├── targets/                  # AI環境アダプター
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── codex.rs              # OpenAI Codex
│   │   └── copilot.rs            # VSCode Copilot
│   ├── components/               # コンポーネント種別
│   │   ├── trait.rs              # 共通インターフェース
│   │   ├── skill.rs              # Skills
│   │   ├── agent.rs              # Agents
│   │   ├── prompt.rs             # Prompts
│   │   └── instruction.rs        # Instructions
│   ├── marketplace/              # マーケットプレイス
│   │   ├── registry.rs           # マーケットプレイス登録管理
│   │   └── fetcher.rs            # marketplace.json取得
│   ├── plugin/                   # プラグイン
│   │   ├── manifest.rs           # plugin.json パーサー
│   │   ├── cache.rs              # プラグインキャッシュ管理
│   │   └── deployer.rs           # 自動展開ロジック
│   ├── source/                   # プラグインソース
│   │   ├── trait.rs              # PluginSource トレイト
│   │   └── github.rs             # GitHub実装
│   ├── parser/                   # ファイルパーサー
│   │   ├── skill_md.rs           # SKILL.md パーサー
│   │   ├── agent_md.rs           # .agent.md パーサー
│   │   ├── prompt_md.rs          # .prompt.md パーサー
│   │   └── plugin_json.rs        # plugin.json パーサー
│   └── config.rs                 # 設定管理
├── tests/
└── README.md
```

## モジュール構成方針

Featureベースのモジュール構成を採用。レイヤーベース（domain/, application/, infrastructure/）ではなく、関連する機能を同じモジュール/フォルダにまとめます。

```
src/
├── target/           # Target 関連の全て
│   ├── codex.rs      # Codex ターゲット実装
│   ├── copilot.rs    # Copilot ターゲット実装
│   └── effect.rs     # ターゲット操作の結果
├── plugin/           # Plugin 関連の全て
│   ├── cache.rs      # キャッシュ管理
│   └── manifest.rs   # マニフェスト
└── component/        # Component 関連の全て
    ├── kind.rs       # コンポーネント種別
    └── deployment.rs # デプロイメント
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
