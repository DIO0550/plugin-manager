# ロードマップ

PLMの実装状況と将来の計画について説明します。

## 実装フェーズ

### Phase 1: 基盤構築 ✅

- [x] Cargoプロジェクト初期化
- [x] CLI引数パーサー（clap）
- [x] 基本的なエラーハンドリング
- [x] GitRepo構造体（raw保持、URL生成）

### Phase 2: Target/Component 実装 ✅

- [x] Target trait 定義
- [x] Component trait 定義
- [x] Codexターゲット実装
- [x] Copilotターゲット実装
- [x] Antigravityターゲット実装
- [x] Gemini CLIターゲット実装
- [x] `plm target` コマンド

### Phase 3: パーサー実装 ✅

- [x] SKILL.md パーサー（YAML frontmatter）
- [x] .agent.md パーサー
- [x] .prompt.md パーサー
- [x] plugin.json パーサー
- [x] Claude Code / Codex / Copilot 各形式のパーサー・変換

### Phase 4: GitHubダウンロード・インストール ✅

- [x] GitHubリポジトリダウンロード
- [x] ZIP展開
- [x] コンポーネント種別の自動検出
- [x] `plm install` コマンド
- [x] 自動展開ロジック

### Phase 5: キャッシュ基盤 ✅

- [x] `CachedPlugin` 構造体定義
- [x] `CachedPlugin` に `marketplace` フィールド追加
- [x] `PluginCache` 読み書き実装
- [x] `git_repo()` メソッド実装
- [x] キャッシュディレクトリ階層化

### Phase 6: マーケットプレイス機能 ✅

- [x] `plm marketplace add/remove/list`
- [x] marketplace.json パーサー
- [x] マーケットプレイスキャッシュ管理
- [x] `find_plugins()` 実装（全マッチ返却）
- [x] `has_conflict()` ヘルパー
- [x] 曖昧性解消 CLI フロー

### Phase 7: 管理機能 ✅

- [x] `plm list` コマンド
- [x] `plm info` コマンド
- [x] `plm uninstall` コマンド（展開先も削除）
- [x] `plm enable/disable` コマンド

### Phase 8: 更新・同期機能 ✅

- [x] `plm update` コマンド
- [x] `plm sync` コマンド
- [x] バージョン/SHA比較ロジック

### Phase 9: 作成・配布機能 🚧

- [ ] `plm init` コマンド（テンプレート生成）※CLI定義のみで**未実装スタブ**（[#323](https://github.com/DIO0550/plugin-manager/issues/323)）
- [ ] `plm pack` コマンド（ZIP作成）※CLI定義のみで**未実装スタブ**（[#323](https://github.com/DIO0550/plugin-manager/issues/323)）

### Phase 10: インポート機能 ✅

- [x] Claude Code Plugin構造の解析
- [x] コンポーネント抽出
- [x] `plm import` コマンド

### Phase 11: TUI基盤 ✅

- [x] ratatui 依存追加
- [x] 基本レイアウト（タブ、リスト、詳細）
- [x] キーバインド設計

### Phase 12: TUIタブ実装 🚧

- [x] Installedタブ（プラグイン一覧、詳細、View on GitHub）
- [ ] Discoverタブ（マーケットプレイス検索・インストール）※現状は**プレースホルダのみ**で操作不可（[#325](https://github.com/DIO0550/plugin-manager/issues/325)）
- [x] Marketplacesタブ
- [x] Errorsタブ
- [x] プラグイン選択ダイアログ（同名競合時）

### Phase 13: TUIアクション実装 ✅

- [x] Enable/Disable 実装
- [x] Uninstall 実装
- [x] Update now 実装
- [x] Mark for update（バッチ更新）

### Phase 14: UX改善 ✅

- [x] プログレスバー（indicatif）
- [x] カラー出力（owo-colors）
- [x] テーブル表示（comfy-table）
- [x] エラーメッセージ改善
- [x] ヘルプ・ドキュメント

### Phase 15: リンク機能 ✅

- [x] `plm link` コマンド（シンボリックリンク作成）
- [x] `plm unlink` コマンド（シンボリックリンク削除）

## 将来の拡張

### 追加ターゲット候補

| ターゲット | 説明 |
|------------|------|
| Cursor | .cursor/ ディレクトリ |
| Windsurf | Windsurf IDE |
| Aider | Aider CLI |
| その他 | SKILL.md対応ツール |

### GitLab/Bitbucket対応

```rust
impl GitRepo {
    // 将来追加
    pub fn gitlab_repo_url(&self) -> String;
    pub fn gitlab_web_url(&self) -> String;

    pub fn bitbucket_repo_url(&self) -> String;
    pub fn bitbucket_web_url(&self) -> String;
}
```

### 追加機能候補

| 機能 | 説明 | Issue |
|------|------|-------|
| `plm search` | プラグイン検索 | [#325](https://github.com/DIO0550/plugin-manager/issues/325) |
| 依存関係解決 | プラグイン間の依存関係管理 | [#318](https://github.com/DIO0550/plugin-manager/issues/318) |
| Lockfile | バージョン固定機能 | [#327](https://github.com/DIO0550/plugin-manager/issues/327) |
| `plm dev` | ローカルプラグイン開発支援 | [#324](https://github.com/DIO0550/plugin-manager/issues/324) |
| `plm validate` | プラグインバリデーション | [#326](https://github.com/DIO0550/plugin-manager/issues/326) |
| CI/CD統合 | GitHub Actions対応 | [#322](https://github.com/DIO0550/plugin-manager/issues/322)（非対話環境対応） |
| ホスティング | プラグインのホスティング機能 | - |

### Claude Code Plugin とのギャップ（Issue 管理）

Claude Code のプラグインシステム（`claude plugin` CLI / `/plugin`）との比較で判明した不足仕様・利便性ギャップは、以下の Issue で管理する（2026-07 調査）。

| 分類 | Issue |
|------|-------|
| plugin.json: `mcpServers` / `lspServers` が警告なしで無視される | [#317](https://github.com/DIO0550/plugin-manager/issues/317) |
| plugin.json: 配列型パス・`displayName`・`defaultEnabled`・`dependencies` 等の未対応 | [#318](https://github.com/DIO0550/plugin-manager/issues/318) |
| marketplace.json: source の `ref`/`sha`・`url`・`git-subdir`、`metadata.pluginRoot`、`strict` 未対応 | [#319](https://github.com/DIO0550/plugin-manager/issues/319) |
| `${CLAUDE_PLUGIN_ROOT}` の markdown 本文置換 | [#320](https://github.com/DIO0550/plugin-manager/issues/320) |
| `enable`/`disable`/`update` の `--target` が codex/copilot 限定 | [#321](https://github.com/DIO0550/plugin-manager/issues/321) |
| 非対話環境（CI）対応: デフォルト scope / `--yes` | [#322](https://github.com/DIO0550/plugin-manager/issues/322) |
| `plm init` / `plm pack` の実装 | [#323](https://github.com/DIO0550/plugin-manager/issues/323) |
| ローカルパスからの install / import（開発ループ） | [#324](https://github.com/DIO0550/plugin-manager/issues/324) |
| `plm search` と TUI Discover タブの実装 | [#325](https://github.com/DIO0550/plugin-manager/issues/325) |
| `plm validate` の実装 | [#326](https://github.com/DIO0550/plugin-manager/issues/326) |
| バージョン管理強化（version/semver・marketplace ref 固定・lockfile） | [#327](https://github.com/DIO0550/plugin-manager/issues/327) |
| sync 改善（Hook 対応・`--no-delete`） | [#328](https://github.com/DIO0550/plugin-manager/issues/328) |
| CLI 整合性・出力改善（`marketplace list --json`・shell 補完等） | [#329](https://github.com/DIO0550/plugin-manager/issues/329) |
| インストール前の内訳表示・変換ロスの可視化 | [#330](https://github.com/DIO0550/plugin-manager/issues/330) |
| 名前空間 flatten の prefix 衝突・marketplace `renames` 追従 | [#331](https://github.com/DIO0550/plugin-manager/issues/331) |
| プロジェクト設定ファイルによるチーム共有（`plm apply`） | [#332](https://github.com/DIO0550/plugin-manager/issues/332) |
| `~/.plm/config.toml` の実装 | [#333](https://github.com/DIO0550/plugin-manager/issues/333) |

hooks 関連のギャップは既存 Issue（[#304](https://github.com/DIO0550/plugin-manager/issues/304), [#308](https://github.com/DIO0550/plugin-manager/issues/308), [#309](https://github.com/DIO0550/plugin-manager/issues/309), [#310](https://github.com/DIO0550/plugin-manager/issues/310)）で管理する。

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

### Google Antigravity

- [Getting Started with Google Antigravity](https://codelabs.developers.google.com/getting-started-google-antigravity)
- [Authoring Google Antigravity Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)
- [Build with Google Antigravity](https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/)
