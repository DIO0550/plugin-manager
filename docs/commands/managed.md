# plm managed

TUI（ターミナルユーザーインターフェース）管理画面を起動します。

## 基本構文

```bash
plm managed
```

## 画面構成

```
┌─────────────────────────────────────────────────────────────────┐
│  Discover    [Installed]    Marketplaces    Errors  (tab)       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  cc-plugin @ DIO0550-marketplace                                │
│                                                                 │
│  Scope: user                                                    │
│  Version: 1.0.1                                                 │
│  プラグイン                                                      │
│                                                                 │
│  Author: DIO0550                                                │
│  Status: Enabled                                                │
│                                                                 │
│  Installed components:                                          │
│  • Commands: commit, review-test-code, fix-all-issues, ...      │
│  • Agents: git-commit-agent, tidy-first-reviewer, ...           │
│  • Hooks: PreToolUse                                            │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  > Disable plugin                                               │
│    Mark for update                                              │
│    Update now                                                   │
│    Uninstall                                                    │
│    View on GitHub                                               │
│    Back to plugin list                                          │
└─────────────────────────────────────────────────────────────────┘
```

## タブ構成

| タブ | 内容 |
|------|------|
| **Discover** | マーケットプレイスから利用可能なプラグインを検索・インストール |
| **Installed** | インストール済みプラグインの管理 |
| **Marketplaces** | 登録済みマーケットプレイスの一覧・管理 |
| **Errors** | エラー・警告の一覧 |

## キーバインド

| キー | 操作 |
|------|------|
| `Tab` / `Shift+Tab` | タブ切り替え |
| `↑` / `↓` / `j` / `k` | リスト内移動 |
| `Enter` | 選択/アクション実行 |
| `Space` | チェックボックス切り替え |
| `q` | 終了 |
| `?` | ヘルプ表示 |

## アクション一覧

| アクション | 説明 | 備考 |
|------------|------|------|
| Disable/Enable plugin | プラグインの有効/無効切替 | キャッシュとデプロイ先を更新 |
| Mark for update | 更新対象としてマーク | バッチ更新用 |
| Update now | 即座に更新を実行 | GitHub APIで最新を取得 |
| Uninstall | プラグインを削除 | ファイルとキャッシュを削除 |
| View on GitHub | リポジトリページをブラウザで開く | `GitRepo.github_web_url()`を使用 |

## Discoverタブ

マーケットプレイスからプラグインを検索・インストール:

- 登録済みマーケットプレイスのプラグイン一覧を表示
- プラグインを選択してインストール
- ターゲット・スコープを選択

## Installedタブ

インストール済みプラグインの管理:

- プラグイン一覧表示
- 詳細情報の確認
- 有効/無効の切替
- 更新・削除

## 同名プラグインの選択

複数のマーケットプレイスに同名プラグインがある場合、選択ダイアログを表示:

```
┌─────────────────────────────────────────────────────────────┐
│  Multiple plugins found: formatter                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  > [ ] formatter@company-tools                              │
│        v1.0.0 - Code formatting tool                        │
│                                                             │
│    [ ] formatter@anthropic                                  │
│        v2.0.0 - Advanced formatter with AI                  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [Enter] Select   [Esc] Cancel                              │
└─────────────────────────────────────────────────────────────┘
```

## 関連

- [architecture/tui](../architecture/tui.md) - TUIアーキテクチャ
- [commands/index](./index.md) - CLI vs TUIの使い分け
