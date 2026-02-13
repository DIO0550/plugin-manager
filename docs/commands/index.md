# コマンドリファレンス

PLMのコマンド一覧と使い分けガイド。

## コマンド一覧

| コマンド | 説明 |
|----------|------|
| [install](./install.md) | GitHubまたはマーケットプレイスからプラグインをインストール |
| [list](./list.md) | インストール済みコンポーネントの一覧表示 |
| [info](./info.md) | プラグインの詳細情報を表示 |
| [enable](./managed.md) | コンポーネントを有効化（ターゲットへ展開） |
| [disable](./managed.md) | コンポーネントを無効化（ターゲットから除去、キャッシュは保持） |
| [uninstall](./managed.md) | コンポーネントを完全削除（キャッシュ含む） |
| [update](./managed.md) | コンポーネントの更新チェック・適用 |
| [target](./target.md) | ターゲット環境の管理 |
| [marketplace](./marketplace.md) | マーケットプレイスの管理 |
| [managed](./managed.md) | TUI管理画面を起動 |
| [sync](./sync.md) | 環境間のコンポーネント同期 |
| [init](./init.md) | コンポーネントテンプレートの作成 |
| [pack](./pack.md) | コンポーネントのパッケージ化 |
| [import](./import.md) | Claude Code Pluginからのインポート |
| [link](./link.md) | シンボリックリンクの作成 |
| [unlink](./link.md#plm-unlink) | シンボリックリンクの削除 |

## CLI vs TUI の使い分け

| 操作 | CLI直接 | TUI管理画面 |
|------|---------|-------------|
| インストール | `plm install` | Discoverタブ |
| 更新 | `plm update` | ○ |
| 有効化 | `plm enable` | ○ |
| 無効化 | `plm disable` | ○ |
| 削除 | `plm uninstall` | ○ |
| 状態確認 | `plm list` | ○ |
| GitHub参照 | - | ○ "View on GitHub" |
| 詳細表示 | `plm info` | ○ |

## CLIの推奨ユースケース

- **スクリプト/自動化**: CI/CDパイプラインでのインストール
- **単発インストール**: 特定のプラグインを素早くインストール
- **情報取得**: `plm list`や`plm info`での状態確認

## TUIの推奨ユースケース

- **管理作業**: 複数プラグインの有効/無効切替、更新
- **ブラウジング**: マーケットプレイスの探索
- **詳細確認**: プラグインの詳細情報確認とGitHubページへのアクセス

## コマンド体系

```bash
# インストール（直接CLI）
plm install <source>                    # GitHubからインストール
plm install formatter@my-market         # マーケットプレイス経由
plm install owner/repo --target codex   # ターゲット指定
plm install owner/repo --scope personal # スコープ指定

# 管理画面（TUI）
plm managed                             # インタラクティブ管理画面

# マーケットプレイス管理
plm marketplace list
plm marketplace add owner/repo
plm marketplace add owner/repo --name my-market
plm marketplace remove <name>
plm marketplace update

# ターゲット管理
plm target list
plm target add codex
plm target add copilot
plm target remove copilot

# 簡易一覧・情報（非インタラクティブ）
plm list                                # インストール済み一覧
plm list --target codex                 # ターゲット別
plm list --type skill                   # 種別フィルタ
plm info <plugin-name>                  # 詳細情報

# コンポーネント作成・配布
plm init my-skill --type skill          # テンプレート作成
plm init my-agent --type agent
plm pack ./my-component                 # 配布用パッケージ作成

# 環境間同期
plm sync --from codex --to copilot      # コンポーネントをコピー
plm sync --from codex --to copilot --type skill

# Claude Code Plugin からのインポート
plm import owner/claude-plugin --component skills/pdf
plm import owner/claude-plugin --type skill

# シンボリックリンク
plm link CLAUDE.md .github/copilot-instructions.md     # シンボリックリンク作成
plm link --force CLAUDE.md .gemini/GEMINI.md            # 既存を上書き
plm unlink .github/copilot-instructions.md              # シンボリックリンク削除
```
