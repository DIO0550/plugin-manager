# コア設計

PLMのコア設計（Traits, 構造体）について説明します。

## Component Trait

コンポーネント種別の共通インターフェース。

```rust
/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    /// スキル（SKILL.md形式）
    Skill,
    /// エージェント（.agent.md形式）
    Agent,
    /// コマンド（.prompt.md形式）
    Command,
    /// インストラクション（AGENTS.md, copilot-instructions.md形式）
    Instruction,
    /// フック（任意のスクリプト）
    Hook,
}

/// プラグイン内のコンポーネント
pub struct Component {
    pub kind: ComponentKind,
    pub name: String,
    pub path: PathBuf,
}
```

## Target Trait

AI環境の共通インターフェース。

```rust
/// ターゲット環境の抽象化trait
pub trait Target: Send + Sync {
    /// ターゲット識別子（"codex", "copilot", "antigravity", "gemini"）
    fn name(&self) -> &'static str;

    /// 表示名
    fn display_name(&self) -> &'static str;

    /// ターゲット種別
    fn kind(&self) -> TargetKind;

    /// サポートするコンポーネント種別
    fn supported_components(&self) -> &[ComponentKind];

    /// 指定コンポーネント種別をサポートするか
    fn supports(&self, kind: ComponentKind) -> bool;

    /// 配置先ロケーションを取得
    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation>;

    /// コンポーネントを削除
    fn remove(&self, context: &PlacementContext) -> Result<()>;

    /// 配置済みコンポーネント一覧を取得
    fn list_placed(&self, kind: ComponentKind, scope: Scope, project_root: &Path) -> Result<Vec<String>>;
}

pub enum Scope {
    Personal,  // ~/.codex/skills/ など
    Project,   // .codex/skills/ など
}
```

### 実装例（擬似コード）

> **Note**: 以下は設計意図を示す擬似コードです。
> 実装の詳細は `src/target/` 配下の各ファイルを参照してください。

```rust
pub struct CodexTarget;

impl Target for CodexTarget {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[ComponentKind::Skill, ComponentKind::Agent, ComponentKind::Instruction]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        // PlacementContextに基づいて配置先を決定
        // ...
    }
    // ...
}

pub struct AntigravityTarget;

impl Target for AntigravityTarget {
    fn name(&self) -> &'static str {
        "antigravity"
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[ComponentKind::Skill]  // Antigravity only supports Skills
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        // Skills のみサポート、Personal/Project で配置先が異なる
        // ...
    }
    // ...
}

pub struct GeminiCliTarget;

impl Target for GeminiCliTarget {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &[ComponentKind::Skill, ComponentKind::Instruction]
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        // Skills と Instructions をサポート
        // ...
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
