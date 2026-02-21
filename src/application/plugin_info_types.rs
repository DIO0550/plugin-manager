//! プラグイン詳細情報の型定義
//!
//! PluginDetail, AuthorInfo, PluginSource, ComponentInfo のDTO群。

use serde::Serialize;

/// プラグイン詳細情報（DTO）
#[derive(Debug, Clone, Serialize)]
pub struct PluginDetail {
    /// プラグイン名
    pub name: String,
    /// バージョン
    pub version: String,
    /// 説明文
    pub description: Option<String>,

    /// 作者情報（未設定の場合は None → JSON/YAMLでフィールド省略）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorInfo>,

    /// インストール日時（RFC3339形式）
    #[serde(rename = "installedAt")]
    pub installed_at: Option<String>,
    /// ソース情報
    pub source: PluginSource,

    /// コンポーネント一覧
    pub components: ComponentInfo,

    /// 有効状態
    pub enabled: bool,
    /// キャッシュパス（絶対パス）
    #[serde(rename = "cachePath")]
    pub cache_path: String,
}

/// 作者情報
#[derive(Debug, Clone, Serialize)]
pub struct AuthorInfo {
    /// 作者名
    pub name: String,
    /// メールアドレス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// プラグインソース情報
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PluginSource {
    /// GitHub からインストール
    GitHub { repository: String },
    /// マーケットプレイスからインストール
    Marketplace { name: String },
}

/// コンポーネント一覧
#[derive(Debug, Clone, Serialize)]
pub struct ComponentInfo {
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub commands: Vec<String>,
    pub instructions: Vec<String>,
    pub hooks: Vec<String>,
}
