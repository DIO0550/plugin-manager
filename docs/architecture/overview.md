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
│   ├── commands/                 # コマンド群（ライフサイクル/配置/管理の3サブグループ + info/list 既存維持）
│   │   ├── lifecycle.rs          # ライフサイクル系コマンド集約親
│   │   ├── lifecycle/            #
│   │   │   ├── disable.rs            # 無効化
│   │   │   ├── disable_test.rs       # disable.rs テスト
│   │   │   ├── enable.rs             # 有効化
│   │   │   ├── enable_test.rs        # enable.rs テスト
│   │   │   ├── uninstall.rs          # 削除処理
│   │   │   ├── uninstall_test.rs     # uninstall.rs テスト
│   │   │   └── update.rs             # 更新処理
│   │   ├── deploy.rs             # 配置系コマンド集約親
│   │   ├── deploy/               #
│   │   │   ├── install.rs            # インストール処理
│   │   │   ├── install_test.rs       # install.rs テスト
│   │   │   ├── link.rs               # シンボリックリンク作成
│   │   │   ├── link_test.rs          # link.rs テスト
│   │   │   ├── unlink.rs             # シンボリックリンク削除
│   │   │   ├── unlink_test.rs        # unlink.rs テスト
│   │   │   ├── sync.rs               # 環境間同期
│   │   │   ├── sync_test.rs          # sync.rs テスト
│   │   │   ├── import.rs             # Claude Plugin インポート
│   │   │   └── import_test.rs        # import.rs テスト
│   │   ├── manage.rs             # 管理系コマンド集約親
│   │   ├── manage/               #
│   │   │   ├── init.rs               # テンプレート作成
│   │   │   ├── pack.rs               # パッケージ化
│   │   │   ├── target.rs             # ターゲット環境管理
│   │   │   ├── marketplace.rs        # マーケットプレイス管理
│   │   │   └── managed.rs            # TUI管理画面起動
│   │   ├── info.rs               # 詳細情報（既存維持）
│   │   ├── info/                 # 出力フォーマット別実装
│   │   ├── list.rs               # 一覧表示（既存維持）
│   │   └── list/                 # 出力フォーマット別実装
│   ├── target.rs                 # Target trait定義
│   ├── target_test.rs            # target.rs の単体テスト
│   ├── target/                   # AI環境アダプター（環境/配置/コアの3サブグループ）
│   │   ├── env.rs                # 環境実装サブグループ親
│   │   ├── env/                  #
│   │   │   ├── antigravity.rs        # Google Antigravity
│   │   │   ├── antigravity_test.rs   # antigravity.rs テスト
│   │   │   ├── codex.rs              # OpenAI Codex
│   │   │   ├── codex_test.rs         # codex.rs テスト
│   │   │   ├── copilot.rs            # GitHub Copilot
│   │   │   ├── copilot_test.rs       # copilot.rs テスト
│   │   │   ├── gemini_cli.rs         # Gemini CLI
│   │   │   └── gemini_cli_test.rs    # gemini_cli.rs テスト
│   │   ├── placed.rs             # 配置ユーティリティサブグループ親
│   │   ├── placed/               #
│   │   │   ├── placed.rs         # 全ターゲット横断スキャン
│   │   │   ├── placed_common.rs  # Instruction placement 共通ロジック
│   │   │   ├── scanner.rs        # フラット1階層スキャナ
│   │   │   └── scanner_test.rs   # scanner.rs テスト
│   │   ├── core.rs               # コアドメインサブグループ親
│   │   ├── core/                 #
│   │   │   ├── id.rs             # TargetId 値オブジェクト
│   │   │   ├── id_test.rs        # id.rs テスト
│   │   │   ├── paths.rs          # home_dir / base_dir 共通パス計算
│   │   │   ├── paths_test.rs     # paths.rs テスト
│   │   │   ├── registry.rs       # TargetRegistry 状態マシン
│   │   │   └── registry_test.rs  # registry.rs テスト
│   │   ├── effect.rs             # ターゲット操作結果（ルート維持）
│   │   └── effect_test.rs        # effect.rs テスト
│   ├── component.rs              # コンポーネントモジュール定義
│   ├── component/                # コンポーネント種別（model サブグループ + ルート）
│   │   ├── model.rs              # 値オブジェクト群サブグループ親
│   │   ├── model/                #
│   │   │   ├── kind.rs                  # ComponentKind / Component / Scope
│   │   │   ├── kind_test.rs             # kind.rs テスト
│   │   │   ├── placement.rs             # ComponentRef / PlacementContext / PlacementLocation
│   │   │   ├── placement_test.rs        # placement.rs テスト
│   │   │   ├── scoped_path.rs           # ScopedPath
│   │   │   ├── scoped_path_test.rs      # scoped_path.rs テスト
│   │   │   ├── file_operation.rs        # FileOperation
│   │   │   └── file_operation_test.rs   # file_operation.rs テスト
│   │   ├── convert.rs            # コンポーネント変換（ルート維持）
│   │   ├── convert_test.rs       # convert.rs テスト
│   │   ├── deployment.rs         # デプロイメント情報サブグループ親（ルート維持）
│   │   ├── deployment_test.rs    # deployment.rs テスト
│   │   └── deployment/           # デプロイメント実装
│   ├── plugin.rs                 # プラグインモジュール定義
│   ├── plugin/                   # プラグイン管理（4 サブグループ）
│   │   ├── cache.rs              # cache サブグループ親
│   │   ├── cache/                # キャッシュ管理サブグループ
│   │   │   ├── cache.rs          # PackageCache / PackageCacheAccess
│   │   │   ├── cached_package.rs # CachedPackage
│   │   │   └── cleanup.rs        # cleanup_legacy_hierarchy / cleanup_plugin_directories
│   │   ├── content.rs            # content サブグループ親
│   │   ├── content/              # コンテンツサブグループ
│   │   │   ├── installed.rs      # InstalledPlugin
│   │   │   ├── loader.rs         # load_plugin
│   │   │   ├── marketplace_content.rs # MarketplaceContent
│   │   │   └── plugin_content.rs # Plugin
│   │   ├── lifecycle.rs          # lifecycle サブグループ親
│   │   ├── lifecycle/            # ライフサイクルサブグループ
│   │   │   ├── action.rs         # PluginAction
│   │   │   ├── intent.rs         # PluginIntent
│   │   │   └── update.rs         # update_plugin / update_all_plugins
│   │   ├── meta.rs               # meta サブグループ親
│   │   └── meta/                 # メタデータサブグループ
│   │       ├── manifest.rs       # plugin.json パーサー
│   │       ├── manifest_resolve.rs # マニフェスト解決
│   │       ├── meta.rs           # プラグインメタデータ
│   │       └── version.rs        # バージョン管理
│   ├── parser.rs                 # パーサーモジュール定義
│   ├── parser/                   # ファイルパーサー（環境別サブフォルダ）
│   │   ├── claude_code.rs        # Claude Code サブグループ親（command + agent）
│   │   ├── claude_code/          #
│   │   │   ├── command.rs        # Claude Code Command形式
│   │   │   └── agent.rs          # Claude Code Agent形式
│   │   ├── codex.rs              # Codex サブグループ親（prompt + agent）
│   │   ├── codex/                #
│   │   │   ├── prompt.rs         # Codex Prompt形式
│   │   │   └── agent.rs          # Codex Agent形式
│   │   ├── copilot.rs            # Copilot サブグループ親（prompt + agent）
│   │   ├── copilot/              #
│   │   │   ├── prompt.rs         # Copilot Prompt形式
│   │   │   └── agent.rs          # Copilot Agent形式
│   │   ├── convert.rs            # フォーマット変換（cross-cutting, ルート維持）
│   │   └── frontmatter.rs        # YAML frontmatterパーサー（cross-cutting, ルート維持）
│   ├── hooks.rs                  # フック変換モジュール定義
│   ├── hooks/                    # フック設定変換（converter / model + 既存 event/tool 維持）
│   │   ├── converter.rs          # converter サブグループ親
│   │   ├── converter/            # フック変換器サブグループ
│   │   │   ├── codex.rs          # Codex 用変換層
│   │   │   ├── converter.rs      # ポリモルフィック変換エンジン
│   │   │   └── copilot.rs        # Copilot CLI 用変換層
│   │   ├── event/                # （既存維持）イベント名マップ
│   │   ├── model.rs              # model サブグループ親
│   │   ├── model/                # フック値オブジェクトサブグループ
│   │   │   ├── hook_definition.rs # CommandHook / HttpHook / StubHook / HookDefinition
│   │   │   ├── name.rs           # HookName
│   │   │   └── script_path.rs    # resolve_script_path
│   │   └── tool/                 # （既存維持）ツール名マップ
│   ├── source.rs                 # ソースモジュール定義
│   ├── source/                   # プラグインソース
│   │   ├── github_source.rs      # GitHub実装
│   │   ├── marketplace_source.rs # マーケットプレイス実装
│   │   └── search_source.rs      # 検索実装
│   ├── marketplace.rs            # マーケットプレイスモジュール定義
│   ├── marketplace/              # マーケットプレイス（path サブグループ + ルート 3 ファイル）
│   │   ├── config.rs             # マーケットプレイス設定（ルート維持）
│   │   ├── download.rs           # marketplace.json取得（ルート維持）
│   │   ├── path.rs               # path サブグループ親
│   │   ├── path/                 # パスユーティリティサブグループ
│   │   │   ├── plugin_source_path.rs # プラグインソースパス
│   │   │   └── windows_path.rs   # Windowsパス処理
│   │   └── registry.rs           # マーケットプレイスレジストリ（ルート維持）
│   ├── sync.rs                   # 同期モジュール定義（オーケストレータはルート維持）
│   ├── sync/                     # 環境間同期（endpoint / model サブグループ）
│   │   ├── endpoint.rs           # endpoint サブグループ親
│   │   ├── endpoint/             # 同期両端 Target ラッパーサブグループ
│   │   │   ├── destination.rs    # 同期先
│   │   │   └── source.rs         # 同期元
│   │   ├── model.rs              # model サブグループ親
│   │   └── model/                # 同期値オブジェクトサブグループ
│   │       ├── action.rs         # 同期アクション
│   │       ├── options.rs        # 同期オプション
│   │       ├── placed.rs         # 配置済みコンポーネント
│   │       └── result.rs         # 同期結果
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
├── target/           # Target 関連の全て（env/placed/core サブグループ）
│   ├── env/          # 環境実装サブグループ
│   │   ├── antigravity.rs # Antigravity ターゲット実装
│   │   ├── codex.rs      # Codex ターゲット実装
│   │   ├── copilot.rs    # Copilot ターゲット実装
│   │   └── gemini_cli.rs # Gemini CLI ターゲット実装
│   ├── placed/       # 配置ユーティリティサブグループ
│   │   ├── placed.rs        # 全ターゲット横断スキャン
│   │   ├── placed_common.rs # Instruction placement 共通ロジック
│   │   └── scanner.rs       # ターゲットスキャン
│   ├── core/         # コアドメインサブグループ
│   │   ├── id.rs        # TargetId 値オブジェクト
│   │   ├── paths.rs     # home_dir / base_dir 共通パス計算
│   │   └── registry.rs  # TargetRegistry 状態マシン
│   └── effect.rs     # ターゲット操作の結果（ルート維持）
├── plugin/           # Plugin 関連の全て（4 サブグループ）
│   ├── cache/        # キャッシュ管理サブグループ
│   ├── content/      # コンテンツサブグループ
│   ├── lifecycle/    # ライフサイクルサブグループ
│   └── meta/         # メタデータサブグループ
├── marketplace/     # Marketplace 関連の全て（path サブグループ + ルート 3 ファイル）
│   ├── config.rs     # マーケットプレイス設定（ルート維持）
│   ├── download.rs   # marketplace.json取得（ルート維持）
│   ├── path/         # パスユーティリティサブグループ
│   └── registry.rs   # マーケットプレイスレジストリ（ルート維持）
├── hooks/           # Hooks 関連の全て（converter / model + 既存 event/tool）
│   ├── converter/   # フック変換器サブグループ
│   ├── event/       # （既存）イベント名マップサブグループ
│   ├── model/       # フック値オブジェクトサブグループ
│   └── tool/        # （既存）ツール名マップサブグループ
├── sync/            # 同期関連の全て（endpoint / model サブグループ + ルートロジック）
│   ├── endpoint/    # 同期両端 Target ラッパーサブグループ
│   └── model/       # 同期値オブジェクトサブグループ
└── component/        # Component 関連の全て（model サブグループ + ルート）
    ├── model/        # 値オブジェクト群サブグループ
    │   ├── kind.rs           # ComponentKind / Component / Scope
    │   ├── placement.rs      # ComponentRef / PlacementContext / PlacementLocation
    │   ├── scoped_path.rs    # ScopedPath
    │   └── file_operation.rs # FileOperation
    ├── convert.rs    # コンポーネント変換（ルート維持）
    └── deployment/   # デプロイメント（ルート維持）
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
