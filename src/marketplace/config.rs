use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 名前の最大長
const MAX_NAME_LENGTH: usize = 64;

/// マーケットプレイス登録情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceRegistration {
    pub name: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// marketplaces.json のルート構造
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketplacesFile {
    marketplaces: Vec<MarketplaceRegistration>,
}

/// マーケットプレイス設定（marketplaces.json）
pub struct MarketplaceConfig {
    path: PathBuf,
    marketplaces: Vec<MarketplaceRegistration>,
}

impl MarketplaceConfig {
    /// Load from default path (~/.plm/marketplaces.json)
    pub fn load() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
        let path = PathBuf::from(home).join(".plm").join("marketplaces.json");
        Self::load_from(path)
    }

    /// Load marketplace config from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `marketplaces.json` file.
    pub fn load_from(path: PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self {
                path,
                marketplaces: Vec::new(),
            });
        }

        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

        let file: MarketplacesFile =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(Self {
            path,
            marketplaces: file.marketplaces,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let file = MarketplacesFile {
            marketplaces: self.marketplaces.clone(),
        };

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        std::fs::write(&self.path, content).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Add a marketplace registration.
    ///
    /// # Arguments
    ///
    /// * `entry` - The marketplace registration to add.
    pub fn add(&mut self, entry: MarketplaceRegistration) -> Result<(), String> {
        if self.exists(&entry.name) {
            return Err(format!(
                "Marketplace '{}' already exists. Use --name to specify a different name.",
                entry.name
            ));
        }
        self.marketplaces.push(entry);
        Ok(())
    }

    /// Remove a marketplace registration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The marketplace name to remove.
    pub fn remove(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .marketplaces
            .iter()
            .position(|e| e.name == name)
            .ok_or_else(|| format!("Marketplace '{}' not found", name))?;
        self.marketplaces.remove(idx);
        Ok(())
    }

    /// Look up a marketplace registration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The marketplace name to look up.
    pub fn get(&self, name: &str) -> Option<&MarketplaceRegistration> {
        self.marketplaces.iter().find(|e| e.name == name)
    }

    pub fn list(&self) -> &[MarketplaceRegistration] {
        &self.marketplaces
    }

    /// Check whether a marketplace with the given name exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The marketplace name to check.
    pub fn exists(&self, name: &str) -> bool {
        self.marketplaces.iter().any(|e| e.name == name)
    }
}

/// 名前の正規化（小文字化）と検証
///
/// # Arguments
///
/// * `name` - The name to normalize and validate.
pub fn normalize_name(name: &str) -> Result<String, String> {
    let normalized = name.to_lowercase();
    validate_name(&normalized)?;
    Ok(normalized)
}

/// 名前の検証のみ（既に正規化済みの名前に対して使用）
///
/// # Arguments
///
/// * `name` - The already-normalized name to validate.
pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(format!(
            "Name is too long (max {} characters)",
            MAX_NAME_LENGTH
        ));
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '.' && c != '_' && c != '-' {
            return Err(format!(
                "Invalid character '{}' in name. Only [a-z0-9._-] are allowed.",
                c
            ));
        }
    }

    let first = name.chars().next().unwrap();
    let last = name.chars().last().unwrap();

    if first == '.' || first == '-' {
        return Err("Name cannot start with a period or hyphen".to_string());
    }

    if last == '.' || last == '-' {
        return Err("Name cannot end with a period or hyphen".to_string());
    }

    Ok(())
}

/// source_path の正規化
///
/// # Arguments
///
/// * `path` - The raw source path to normalize.
pub fn normalize_source_path(path: &str) -> Result<Option<String>, String> {
    if path.contains('\\') {
        return Err("Backslash is not allowed in path. Use forward slash (/) instead.".to_string());
    }

    if path.contains("..") {
        return Err("Path cannot contain '..' (directory traversal is not allowed)".to_string());
    }

    let mut normalized = path.trim_matches('/');
    if let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized = normalized.trim_matches('/');

    if normalized.is_empty() || normalized == "." {
        return Ok(None);
    }

    Ok(Some(normalized.to_string()))
}

/// 内部表現からユーザー表示用に変換
/// github:owner/repo → owner/repo
///
/// # Arguments
///
/// * `internal` - Internal source string (e.g. `github:owner/repo`).
pub fn to_display_source(internal: &str) -> String {
    internal
        .strip_prefix("github:")
        .unwrap_or(internal)
        .to_string()
}

/// ユーザー入力から内部表現に変換
/// owner/repo → github:owner/repo
///
/// # Arguments
///
/// * `display` - User-facing source string (e.g. `owner/repo`).
pub fn to_internal_source(display: &str) -> String {
    if display.starts_with("github:") {
        display.to_string()
    } else {
        format!("github:{}", display)
    }
}

#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;
