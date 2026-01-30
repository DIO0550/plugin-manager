# コア設計

PLMのコア設計（Traits, 構造体）について説明します。

## Component Trait

コンポーネント種別の共通インターフェース。

```rust
/// コンポーネント種別の共通インターフェース
pub trait Component {
    /// 種別名（"skill", "agent", "prompt", "instruction"）
    fn kind(&self) -> ComponentKind;

    /// ファイル名パターン
    fn file_pattern(&self) -> &str;

    /// メタデータをパース
    fn parse_metadata(&self, content: &str) -> Result<ComponentMetadata>;

    /// バリデーション
    fn validate(&self, path: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Skill,
    Agent,
    Prompt,
    Instruction,
}
```

### 実装例

```rust
pub struct SkillComponent;

impl Component for SkillComponent {
    fn kind(&self) -> ComponentKind {
        ComponentKind::Skill
    }

    fn file_pattern(&self) -> &str {
        "SKILL.md"
    }

    fn parse_metadata(&self, content: &str) -> Result<ComponentMetadata> {
        // YAML frontmatterをパース
        // ...
    }

    fn validate(&self, path: &Path) -> Result<()> {
        // SKILL.mdの存在確認など
        // ...
    }
}
```

## Target Trait

AI環境の共通インターフェース。

```rust
/// AI環境の共通インターフェース
pub trait Target {
    /// ターゲット名（"codex", "copilot"）
    fn name(&self) -> &str;

    /// サポートするコンポーネント種別
    fn supported_components(&self) -> Vec<ComponentKind>;

    /// コンポーネントのインストール先パス
    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf>;

    /// コンポーネントをインストール
    fn install(&self, component: &InstalledComponent, scope: Scope) -> Result<()>;

    /// コンポーネントを削除
    fn uninstall(&self, name: &str, kind: ComponentKind, scope: Scope) -> Result<()>;

    /// インストール済み一覧
    fn list(&self, kind: Option<ComponentKind>, scope: Scope) -> Result<Vec<InstalledComponent>>;
}

pub enum Scope {
    Personal,  // ~/.codex/skills/ など
    Project,   // .codex/skills/ など
}
```

### 実装例

```rust
pub struct CodexTarget;

impl Target for CodexTarget {
    fn name(&self) -> &str {
        "codex"
    }

    fn supported_components(&self) -> Vec<ComponentKind> {
        vec![ComponentKind::Skill, ComponentKind::Agent, ComponentKind::Instruction]
    }

    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf> {
        match (kind, scope) {
            (ComponentKind::Skill, Scope::Personal) => Some(dirs::home_dir()?.join(".codex/skills")),
            (ComponentKind::Skill, Scope::Project) => Some(PathBuf::from(".codex/skills")),
            // ...
        }
    }
    // ...
}

pub struct AntigravityTarget;

impl Target for AntigravityTarget {
    fn name(&self) -> &str {
        "antigravity"
    }

    fn supported_components(&self) -> Vec<ComponentKind> {
        vec![ComponentKind::Skill]  // Antigravity only supports Skills
    }

    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf> {
        match (kind, scope) {
            (ComponentKind::Skill, Scope::Personal) => {
                Some(dirs::home_dir()?.join(".gemini/antigravity/skills"))
            }
            (ComponentKind::Skill, Scope::Project) => {
                Some(PathBuf::from(".agent/skills"))
            }
            _ => None,  // Other components not supported
        }
    }
    // ...
}

pub struct GeminiCliTarget;

impl Target for GeminiCliTarget {
    fn name(&self) -> &str {
        "gemini"
    }

    fn supported_components(&self) -> Vec<ComponentKind> {
        vec![ComponentKind::Skill, ComponentKind::Instruction]
    }

    fn component_path(&self, kind: ComponentKind, scope: Scope) -> Option<PathBuf> {
        match (kind, scope) {
            (ComponentKind::Skill, Scope::Personal) => {
                Some(dirs::home_dir()?.join(".gemini/skills"))
            }
            (ComponentKind::Skill, Scope::Project) => {
                Some(PathBuf::from(".gemini/skills"))
            }
            (ComponentKind::Instruction, Scope::Personal) => {
                Some(dirs::home_dir()?.join(".gemini/GEMINI.md"))
            }
            (ComponentKind::Instruction, Scope::Project) => {
                Some(PathBuf::from("GEMINI.md"))
            }
            _ => None,  // Agents and Prompts not supported
        }
    }
    // ...
}
```

## GitRepo 構造体

Gitリポジトリ参照（GitHub/GitLab/Bitbucket等で共通利用可能）。

```rust
/// Gitリポジトリ参照
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    /// パース前の生の入力文字列
    pub raw: String,
}

impl GitRepo {
    /// 新しいGitRepoを作成
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self;

    /// refを指定してGitRepoを作成
    pub fn with_ref(owner: impl Into<String>, repo: impl Into<String>, git_ref: impl Into<String>) -> Self;

    /// "owner/repo" または "owner/repo@ref" 形式をパース
    pub fn parse(input: &str) -> Result<Self>;

    // GitHub API URLs
    pub fn github_repo_url(&self) -> String;           // リポジトリ情報
    pub fn github_zipball_url(&self, ref_: &str) -> String;   // zipダウンロード
    pub fn github_commit_url(&self, ref_: &str) -> String;    // コミットSHA取得
    pub fn github_contents_url(&self, path: &str, ref_: &str) -> String; // ファイル取得

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
let repo = GitRepo::parse("owner/repo@v1.0.0")?;

// URL生成
let download_url = repo.github_zipball_url("v1.0.0");
let web_url = repo.github_web_url();  // ブラウザで開く

// フルネーム
println!("{}", repo.full_name());  // "owner/repo"
```

## MarketplaceRegistry

マーケットプレイス管理。

```rust
impl MarketplaceRegistry {
    /// 全マッチを返す（競合検出用）
    pub fn find_plugins(&self, query: &str) -> Result<Vec<PluginMatch>>;

    /// 競合検出ヘルパー
    pub fn has_conflict(&self, name: &str) -> Result<bool>;

    /// マーケットプレイスを追加
    pub fn add(&mut self, name: &str, repo: GitRepo) -> Result<()>;

    /// マーケットプレイスを削除
    pub fn remove(&mut self, name: &str) -> Result<()>;

    /// 一覧取得
    pub fn list(&self) -> &[Marketplace];
}

pub struct PluginMatch {
    pub marketplace: String,
    pub plugin: MarketplacePluginEntry,
}
```

## CachedPlugin

キャッシュされたプラグイン情報。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPlugin {
    pub name: String,
    pub source: String,  // GitRepo.raw
    pub version: String,
    pub status: PluginStatus,
    pub marketplace: Option<String>,
    pub installed_at: DateTime<Utc>,
    pub installed_sha: String,
    pub author: Option<Author>,
    pub components: PluginComponents,
    pub deployments: HashMap<String, Deployment>,
}

impl CachedPlugin {
    /// sourceからGitRepoを復元
    pub fn git_repo(&self) -> Result<GitRepo> {
        GitRepo::parse(&self.source)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PluginStatus {
    Enabled,
    Disabled,
}
```

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

## 関連

- [overview](./overview.md) - アーキテクチャ概要
- [cache](./cache.md) - キャッシュアーキテクチャ
