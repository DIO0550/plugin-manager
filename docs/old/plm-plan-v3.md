# plm - Plugin Manager CLI 実装計画 v3

GitHubからAI開発環境向けのプラグインをダウンロードし、複数のAI環境を統一的に管理するRust製CLIツール

> **バージョン**: v3（TUI管理画面対応）
> **前バージョン**: [plm-plan-v2.md](./plm-plan-v2.md)（マーケットプレイス方式）

## v2からの変更点

| 項目 | v2 | v3 |
|------|-----|-----|
| 管理方法 | CLIコマンドのみ | TUI管理画面 + CLI |
| リポジトリ参照 | owner/repo のみ | GitRepo構造体（raw保持、URL生成） |
| 更新操作 | `plm update` コマンド | TUI管理画面から |
| GitHub参照 | なし | "View on GitHub" 機能 |

---

## コマンド設計

### コマンド体系

```bash
# インストール（直接CLI）
plm install <source>                    # GitHubからインストール
plm install formatter@my-market         # マーケットプレイス経由

# 管理画面（TUI）
plm managed                             # インタラクティブ管理画面

# マーケットプレイス管理
plm marketplace list
plm marketplace add owner/repo
plm marketplace remove <name>
plm marketplace update

# ターゲット管理
plm target list
plm target add codex
plm target remove copilot

# 簡易一覧（非インタラクティブ）
plm list                                # インストール済み一覧
plm list --target codex                 # ターゲット別
```

### 使い分け

| 操作 | CLI直接 | TUI管理画面 |
|------|---------|-------------|
| インストール | `plm install` | Discoverタブ |
| 更新 | - | ○ |
| 有効/無効 | - | ○ |
| 削除 | - | ○ |
| 状態確認 | `plm list` | ○ |
| GitHub参照 | - | ○ "View on GitHub" |
| 詳細表示 | - | ○ |

---

## TUI管理画面 (`plm managed`)

### 画面構成

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
│    View on GitHub          ← GitRepo.github_web_url()           │
│    Back to plugin list                                          │
└─────────────────────────────────────────────────────────────────┘
```

### タブ構成

| タブ | 内容 |
|------|------|
| Discover | マーケットプレイスから利用可能なプラグイン検索・インストール |
| Installed | インストール済みプラグイン管理 |
| Marketplaces | 登録済みマーケットプレイス一覧・管理 |
| Errors | エラー・警告一覧 |

### アクション一覧

| アクション | 説明 | 実装 |
|------------|------|------|
| Disable/Enable plugin | プラグインの有効/無効切替 | キャッシュ更新 |
| Mark for update | 更新対象としてマーク | バッチ更新用 |
| Update now | 即座に更新 | GitHub API → キャッシュ更新 |
| Uninstall | プラグイン削除 | ファイル削除 + キャッシュ更新 |
| View on GitHub | リポジトリページを開く | `GitRepo.github_web_url()` |

---

## GitRepo 構造体

### 設計方針

- `raw`: パース前の生の入力文字列を保持（永続化・復元用）
- パース済みフィールド: 実行時の効率用（URL生成等）
- URL生成メソッド: GitHub API/Web URL を一元管理

### 構造体定義

```rust
/// Gitリポジトリ参照（GitHub/GitLab/Bitbucket等で共通利用可能）
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    /// パース前の生の入力文字列
    pub raw: String,
}
```

### コンストラクタ

```rust
impl GitRepo {
    /// 新しいGitRepoを作成
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self;

    /// refを指定してGitRepoを作成
    pub fn with_ref(owner, repo, git_ref) -> Self;

    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self>;
}
```

### URL生成メソッド

```rust
impl GitRepo {
    // GitHub API URLs
    pub fn github_repo_url(&self) -> String;           // リポジトリ情報
    pub fn github_zipball_url(&self, ref) -> String;   // zipダウンロード
    pub fn github_commit_url(&self, ref) -> String;    // コミットSHA取得
    pub fn github_contents_url(&self, path, ref) -> String; // ファイル取得

    // Web URLs
    pub fn github_web_url(&self) -> String;            // ブラウザ用

    // ユーティリティ
    pub fn full_name(&self) -> String;                 // "owner/repo"
    pub fn ref_or_default(&self) -> &str;              // refまたは"HEAD"
}
```

### 使用例

```rust
// パース
let repo = GitRepo::parse("anthropics/claude-plugins@v1.0.0")?;

// URL生成
let api_url = repo.github_repo_url();
// => "https://api.github.com/repos/anthropics/claude-plugins"

let web_url = repo.github_web_url();
// => "https://github.com/anthropics/claude-plugins"

// 永続化（rawを使用）
cache.source = repo.raw.clone();  // "anthropics/claude-plugins@v1.0.0"

// 復元
let repo = GitRepo::parse(&cache.source)?;
```

---

## キャッシュアーキテクチャ

### 全体構成

```
┌─────────────────────────────────────────────────────────────────┐
│                        plm managed (TUI)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        PluginCache                              │
│                    (~/.plm/plugins.json)                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ CachedPlugin                                               │ │
│  │  - name: String                                            │ │
│  │  - source: String (GitRepo.raw)  ──┐                       │ │
│  │  - version: String                 │                       │ │
│  │  - status: Enabled/Disabled        │                       │ │
│  │  - installed_sha: String           │                       │ │
│  │  - components: [...]               │                       │ │
│  │  - deployments: {...}              │                       │ │
│  └────────────────────────────────────│───────────────────────┘ │
└───────────────────────────────────────│─────────────────────────┘
                                        │
                                        ▼ GitRepo::parse()
                              ┌─────────────────────┐
                              │      GitRepo        │
                              │  - owner            │
                              │  - repo             │
                              │  - git_ref          │
                              │  - raw              │
                              │                     │
                              │  github_web_url()   │──→ ブラウザで開く
                              │  github_*_url()     │──→ API呼び出し
                              └─────────────────────┘
                                        │
                                        ▼
                              ┌─────────────────────┐
                              │    GitHub API       │
                              │  - 更新チェック     │
                              │  - ダウンロード     │
                              └─────────────────────┘
```

### キャッシュの役割

| 役割 | 説明 |
|------|------|
| オフライン表示 | TUI起動時にネットワーク不要で一覧表示 |
| 状態管理 | Enabled/Disabled、バージョン情報 |
| 更新検知 | installed_sha と最新を比較 |
| 永続化 | `source` (raw) からいつでも `GitRepo` を復元可能 |

### データ構造

#### plugins.json

```json
{
  "version": 1,
  "plugins": [
    {
      "name": "code-formatter",
      "source": "company/claude-plugins@v2.1.0",
      "version": "2.1.0",
      "status": "enabled",
      "marketplace": "company-tools",
      "installed_at": "2025-01-15T10:30:00Z",
      "installed_sha": "abc123def456",
      "author": {
        "name": "Dev Team",
        "email": "dev@company.com"
      },
      "components": {
        "skills": ["code-formatter"],
        "agents": ["formatter-agent"],
        "commands": ["format"],
        "hooks": []
      },
      "deployments": {
        "codex": {
          "scope": "personal",
          "enabled": true,
          "paths": ["~/.codex/skills/code-formatter"]
        },
        "copilot": {
          "scope": "project",
          "enabled": true,
          "paths": [".github/skills/code-formatter"]
        }
      }
    }
  ]
}
```

#### Rust構造体

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPlugin {
    pub name: String,
    pub source: String,              // GitRepo.raw を保存
    pub version: String,
    pub status: PluginStatus,
    pub marketplace: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub installed_sha: String,
    pub author: Option<Author>,
    pub components: PluginComponents,
    pub deployments: HashMap<String, Deployment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}

impl CachedPlugin {
    /// sourceからGitRepoを復元
    pub fn git_repo(&self) -> Result<GitRepo> {
        GitRepo::parse(&self.source)
    }
}
```

---

## ターゲット環境の設定読み込み仕様

各AI開発環境が実際にどのパスから設定ファイルを読み込むかの仕様。

### OpenAI Codex CLI

公式ドキュメント: [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

#### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Global (override) | `~/.codex/AGENTS.override.md` | ✅ | 最優先 |
| Global | `~/.codex/AGENTS.md` | ✅ | Personal対応 |
| Project | `./AGENTS.override.md` | ✅ | ディレクトリ毎 |
| Project | `./AGENTS.md` | ✅ | ディレクトリ毎 |
| Skills (Global) | `~/.codex/skills/` | ✅ | Personal |
| Skills (Project) | `./.codex/skills/` | ✅ | Project |

#### 読み込み順序

1. **Global scope**: `~/.codex/` (または `$CODEX_HOME`) をチェック
   - `AGENTS.override.md` があればそれを使用、なければ `AGENTS.md`
2. **Project scope**: リポジトリルートから現在ディレクトリまで走査
   - 各ディレクトリで `AGENTS.override.md` → `AGENTS.md` → fallback の順
3. **マージ**: ルートから現在ディレクトリに向かって連結（上限: `project_doc_max_bytes` = 32KiB）

#### 設定オプション

```toml
# ~/.codex/config.toml
project_doc_fallback_filenames = ["TEAM_GUIDE.md", ".agents.md"]
project_doc_max_bytes = 65536
```

### VSCode GitHub Copilot

公式ドキュメント: [Use custom instructions in VS Code](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)

#### 読み込みパスと優先順位

| スコープ | パス | 自動読み込み | 備考 |
|---------|------|--------------|------|
| Project | `.github/copilot-instructions.md` | ✅ | メインの指示ファイル |
| Project | `.github/instructions/*.instructions.md` | ❌ | 手動指定が必要 |
| User | VSCode設定の `file` プロパティ | ✅ | 設定で外部ファイル参照 |
| Prompts | `.github/prompts/*.prompt.md` | ❌ | 手動呼び出し |

#### 重要な制約

- **Copilotはグローバルファイル（`~/.copilot/`等）を直接読み込まない**
- Personal スコープは VSCode 設定経由で外部ファイルを参照する形式
- Issue: [Global files outside workspace の要望](https://github.com/microsoft/vscode-copilot-release/issues/3129)

#### VSCode設定での外部ファイル参照

```json
// settings.json (User または Workspace)
{
  "github.copilot.chat.codeGeneration.instructions": [
    {
      "file": "/path/to/personal-instructions.md"
    }
  ]
}
```

#### 必要な設定

```json
{
  "github.copilot.chat.codeGeneration.useInstructionFiles": true
}
```

### PLMでの対応方針

| ターゲット | Personal インストール | 追加アクション |
|-----------|----------------------|----------------|
| Codex | `~/.codex/` に配置 | 不要（自動読み込み） |
| Copilot | ファイル配置 + VSCode設定追記 | `settings.json` への参照追加が必要 |

#### インストールフロー（案）

```
$ plm install some-skill --target copilot

? インストール先を選択:
  > Project (.github/)           ← 追加設定なし
    Personal (~/.plm/copilot/)   ← VSCode設定に自動追記

# Personal選択時の追加処理
1. ~/.plm/copilot/skills/some-skill.md に配置
2. ~/.config/Code/User/settings.json に参照を追加
```

---

## 処理フロー

### インストールフロー

```
1. plm install owner/repo@v1.0.0
2. GitRepo::parse("owner/repo@v1.0.0")
3. repo.github_zipball_url("v1.0.0") でダウンロード
4. ~/.plm/cache/plugins/<name>/ に展開
5. plugin.json パース
6. ターゲットへ自動展開
7. CachedPlugin作成（source = repo.raw）
8. plugins.json に保存
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

---

## 実装フェーズ

### Phase 1: GitRepo リファクタリング ✅

- [x] `raw` フィールド追加
- [x] URL生成メソッド追加
- [x] fetcher.rs の URL 組み立てを GitRepo に委譲

### Phase 2: キャッシュ基盤

- [ ] `CachedPlugin` 構造体定義
- [ ] `PluginCache` 読み書き実装
- [ ] `git_repo()` メソッド実装

### Phase 3: TUI基盤

- [ ] ratatui 依存追加
- [ ] 基本レイアウト（タブ、リスト、詳細）
- [ ] キーバインド設計

### Phase 4: Installedタブ

- [ ] プラグイン一覧表示
- [ ] 詳細表示
- [ ] "View on GitHub" 実装
- [ ] Enable/Disable 実装

### Phase 5: アクション実装

- [ ] Uninstall 実装
- [ ] Update now 実装
- [ ] Mark for update（バッチ更新）

### Phase 6: Discoverタブ

- [ ] マーケットプレイスからプラグイン検索
- [ ] インストールフロー統合

### Phase 7: その他タブ

- [ ] Marketplacesタブ
- [ ] Errorsタブ

---

## 技術選定

### TUIライブラリ

| ライブラリ | 選定理由 |
|------------|----------|
| **ratatui** | Rust製TUIのデファクト、活発なメンテナンス |
| crossterm | クロスプラットフォームターミナル操作 |

### ブラウザ起動

```rust
fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/c", "start", url]).spawn()?;

    Ok(())
}
```

---

## 将来の拡張

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

### 追加ターゲット

- Cursor（.cursor/）
- Windsurf
- Aider
- Gemini CLI
