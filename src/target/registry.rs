//! ターゲットレジストリ（状態マシン）
//!
//! ターゲット設定の永続化を状態マシンパターンで管理する。
//!
//! ## 状態遷移図
//!
//! ```text
//!                    ┌─────────────┐
//!                    │   Initial   │
//!                    └──────┬──────┘
//!                           │ new() / with_path()
//!                           ▼
//!                    ┌─────────────┐
//!              ┌────▶│    Idle     │◀────┐
//!              │     └──────┬──────┘     │
//!              │            │            │
//!              │    load()  │  list()    │
//!              │            ▼            │
//!              │     ┌─────────────┐     │
//!              │     │   Loaded    │     │
//!              │     └──────┬──────┘     │
//!              │            │            │
//!              │  add() / remove()       │
//!              │            ▼            │
//!              │     ┌─────────────┐     │
//!              │     │  Modified   │     │
//!              │     └──────┬──────┘     │
//!              │            │            │
//!              │       save()            │
//!              │            │            │
//!              └────────────┴────────────┘
//! ```

use crate::error::{PlmError, Result};
use crate::target::TargetKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// ターゲット設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetsConfig {
    pub targets: Vec<TargetKind>,
}

impl Default for TargetsConfig {
    fn default() -> Self {
        Self {
            targets: vec![TargetKind::Codex, TargetKind::Copilot],
        }
    }
}

impl TargetsConfig {
    /// 正規化（重複排除 + ソート）
    pub fn normalize(&mut self) {
        self.targets.sort();
        self.targets.dedup();
    }
}

/// 追加操作の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddResult {
    Added,
    AlreadyExists,
}

/// 削除操作の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveResult {
    Removed,
    NotFound,
}

/// 状態マシンの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Loaded,
    Modified,
}

/// ターゲットレジストリ（状態マシン）
///
/// 永続化ファイル `~/.plm/targets.json` を管理する。
/// 状態遷移を追跡し、不整合な操作を防ぐ。
pub struct TargetRegistry {
    config_path: PathBuf,
    state: State,
    config: Option<TargetsConfig>,
}

impl TargetRegistry {
    /// 新しいレジストリを作成（デフォルトパス）
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            PlmError::TargetRegistry("HOME environment variable not set".to_string())
        })?;
        let config_path = PathBuf::from(home).join(".plm").join("targets.json");
        Ok(Self {
            config_path,
            state: State::Idle,
            config: None,
        })
    }

    /// カスタムパスで作成（テスト用）
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
            state: State::Idle,
            config: None,
        }
    }

    /// 設定を読み込み（Idle → Loaded）
    pub fn load(&mut self) -> Result<&TargetsConfig> {
        let mut config = match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                serde_json::from_str(&content).map_err(|e| {
                    PlmError::TargetRegistry(format!(
                        "Failed to parse targets.json: {}",
                        e
                    ))
                })?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => TargetsConfig::default(),
            Err(e) => return Err(PlmError::Io(e)),
        };

        config.normalize();
        self.config = Some(config);
        self.state = State::Loaded;

        Ok(self.config.as_ref().unwrap())
    }

    /// 設定を保存（Modified → Idle）
    fn save(&mut self) -> Result<()> {
        let config = self.config.as_ref().ok_or_else(|| {
            PlmError::TargetRegistry("No config loaded".to_string())
        })?;

        // 親ディレクトリを作成
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 同じディレクトリに一時ファイルを作成
        let parent = self.config_path.parent().unwrap_or(Path::new("."));
        let mut temp_file = NamedTempFile::new_in(parent).map_err(|e| {
            PlmError::TargetRegistry(format!("Failed to create temp file: {}", e))
        })?;

        // JSONを書き込み
        let content = serde_json::to_string_pretty(config)?;
        temp_file.write_all(content.as_bytes())?;

        // アトミックに置換
        temp_file.persist(&self.config_path).map_err(|e| {
            PlmError::TargetRegistry(format!("Failed to persist config: {}", e))
        })?;

        self.state = State::Idle;
        Ok(())
    }

    /// ターゲット一覧を取得
    pub fn list(&mut self) -> Result<Vec<TargetKind>> {
        if self.state == State::Idle {
            self.load()?;
        }
        Ok(self.config.as_ref().unwrap().targets.clone())
    }

    /// ターゲットを追加（load → modify → normalize → save）
    pub fn add(&mut self, target: TargetKind) -> Result<AddResult> {
        if self.state == State::Idle {
            self.load()?;
        }

        let config = self.config.as_mut().unwrap();

        if config.targets.contains(&target) {
            return Ok(AddResult::AlreadyExists);
        }

        config.targets.push(target);
        config.normalize();
        self.state = State::Modified;

        self.save()?;
        Ok(AddResult::Added)
    }

    /// ターゲットを削除（load → modify → normalize → save）
    pub fn remove(&mut self, target: TargetKind) -> Result<RemoveResult> {
        if self.state == State::Idle {
            self.load()?;
        }

        let config = self.config.as_mut().unwrap();
        let original_len = config.targets.len();

        config.targets.retain(|t| *t != target);

        if config.targets.len() == original_len {
            return Ok(RemoveResult::NotFound);
        }

        config.normalize();
        self.state = State::Modified;

        self.save()?;
        Ok(RemoveResult::Removed)
    }

    /// 現在の状態を取得（デバッグ用）
    #[cfg(test)]
    pub fn current_state(&self) -> &'static str {
        match self.state {
            State::Idle => "Idle",
            State::Loaded => "Loaded",
            State::Modified => "Modified",
        }
    }
}

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;
