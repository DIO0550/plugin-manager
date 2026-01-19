# ロードマップ

PLMの実装状況と将来の計画について説明します。

## 実装フェーズ

### Phase 1: 基盤構築 ✅

- [x] Cargoプロジェクト初期化
- [x] CLI引数パーサー（clap）
- [x] 基本的なエラーハンドリング
- [x] GitRepo構造体（raw保持、URL生成）

### Phase 2: Target/Component 実装

- [ ] Target trait 定義
- [ ] Component trait 定義
- [ ] Codexターゲット実装
- [ ] Copilotターゲット実装
- [ ] `plm target` コマンド

### Phase 3: パーサー実装

- [ ] SKILL.md パーサー（YAML frontmatter）
- [ ] .agent.md パーサー
- [ ] .prompt.md パーサー
- [ ] plugin.json パーサー

### Phase 4: GitHubダウンロード・インストール

- [ ] GitHubリポジトリダウンロード
- [ ] ZIP展開
- [ ] コンポーネント種別の自動検出
- [ ] `plm install` コマンド
- [ ] 自動展開ロジック

### Phase 5: キャッシュ基盤

- [ ] `CachedPlugin` 構造体定義
- [ ] `CachedPlugin` に `marketplace` フィールド追加
- [ ] `PluginCache` 読み書き実装
- [ ] `git_repo()` メソッド実装
- [ ] キャッシュディレクトリ階層化

### Phase 6: マーケットプレイス機能

- [ ] `plm marketplace add/remove/list`
- [ ] marketplace.json パーサー
- [ ] マーケットプレイスキャッシュ管理
- [ ] `find_plugins()` 実装（全マッチ返却）
- [ ] `has_conflict()` ヘルパー
- [ ] 曖昧性解消 CLI フロー

### Phase 7: 管理機能

- [ ] `plm list` コマンド
- [ ] `plm info` コマンド
- [ ] `plm uninstall` コマンド（展開先も削除）
- [ ] `plm enable/disable` コマンド

### Phase 8: 更新・同期機能

- [ ] `plm update` コマンド
- [ ] `plm sync` コマンド
- [ ] バージョン/SHA比較ロジック

### Phase 9: 作成・配布機能

- [ ] `plm init` コマンド（テンプレート生成）
- [ ] `plm pack` コマンド（ZIP作成）

### Phase 10: インポート機能

- [ ] Claude Code Plugin構造の解析
- [ ] コンポーネント抽出
- [ ] `plm import` コマンド

### Phase 11: TUI基盤

- [ ] ratatui 依存追加
- [ ] 基本レイアウト（タブ、リスト、詳細）
- [ ] キーバインド設計

### Phase 12: TUIタブ実装

- [ ] Installedタブ（プラグイン一覧、詳細、View on GitHub）
- [ ] Discoverタブ（マーケットプレイス検索・インストール）
- [ ] Marketplacesタブ
- [ ] Errorsタブ
- [ ] プラグイン選択ダイアログ（同名競合時）

### Phase 13: TUIアクション実装

- [ ] Enable/Disable 実装
- [ ] Uninstall 実装
- [ ] Update now 実装
- [ ] Mark for update（バッチ更新）

### Phase 14: UX改善

- [ ] プログレスバー（indicatif）
- [ ] カラー出力（owo-colors）
- [ ] テーブル表示（comfy-table）
- [ ] エラーメッセージ改善
- [ ] ヘルプ・ドキュメント

## 将来の拡張

### 追加ターゲット候補

| ターゲット | 説明 |
|------------|------|
| Cursor | .cursor/ ディレクトリ |
| Windsurf | Windsurf IDE |
| Aider | Aider CLI |
| Gemini CLI | Google Gemini CLI |
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

| 機能 | 説明 |
|------|------|
| `plm search` | プラグイン検索 |
| 依存関係解決 | プラグイン間の依存関係管理 |
| Lockfile | バージョン固定機能 |
| `plm dev` | ローカルプラグイン開発支援 |
| `plm validate` | プラグインバリデーション |
| CI/CD統合 | GitHub Actions対応 |
| ホスティング | プラグインのホスティング機能 |

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
