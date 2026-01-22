//! インポートレジストリ（状態マシン）
//!
//! インポート履歴の永続化を状態マシンパターンで管理する。
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
//!              │     record()            │
//!              │            ▼            │
//!              │     ┌─────────────┐     │
//!              │     │  Modified   │     │
//!              │     └──────┬──────┘     │
//!              │            │            │
//!              │       save()            │
//!              │            │            │
//!              └────────────┴────────────┘
//! ```

use crate::component::ComponentKind;
use crate::error::{PlmError, Result};
use crate::target::{Scope, TargetKind};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// インポート履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecord {
    /// インポート元リポジトリ（owner/repo 形式）
    pub source_repo: String,
    /// コンポーネント種別
    pub kind: ComponentKind,
    /// コンポーネント名
    pub name: String,
    /// 配置先ターゲット
    pub target: TargetKind,
    /// 配置先スコープ
    pub scope: Scope,
    /// 配置先パス
    pub path: PathBuf,
    /// インポート日時（RFC3339形式）
    pub imported_at: String,
    /// Git ref（ブランチ/タグ）
    pub git_ref: String,
    /// コミットSHA
    pub commit_sha: String,
}

/// インポート履歴設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportsConfig {
    pub imports: Vec<ImportRecord>,
}

/// 状態マシンの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Loaded,
    Modified,
}

/// インポートレジストリ（状態マシン）
///
/// 永続化ファイル `~/.plm/imports.json` を管理する。
/// 状態遷移を追跡し、不整合な操作を防ぐ。
pub struct ImportRegistry {
    pub(crate) config_path: PathBuf,
    state: State,
    config: Option<ImportsConfig>,
}

impl ImportRegistry {
    /// 新しいレジストリを作成（デフォルトパス: ~/.plm/imports.json）
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            PlmError::ImportRegistry("HOME environment variable not set".to_string())
        })?;
        let config_path = PathBuf::from(home).join(".plm").join("imports.json");
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
    pub fn load(&mut self) -> Result<&ImportsConfig> {
        let config = match fs::read_to_string(&self.config_path) {
            Ok(content) => serde_json::from_str(&content).map_err(|e| {
                PlmError::ImportRegistry(format!("Failed to parse imports.json: {}", e))
            })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ImportsConfig::default(),
            Err(e) => return Err(PlmError::Io(e)),
        };

        self.config = Some(config);
        self.state = State::Loaded;

        Ok(self.config.as_ref().unwrap())
    }

    /// 設定を保存（Modified → Idle）
    fn save(&mut self) -> Result<()> {
        let config = self.config.as_ref().ok_or_else(|| {
            PlmError::ImportRegistry("No config loaded".to_string())
        })?;

        // 親ディレクトリを作成
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 同じディレクトリに一時ファイルを作成
        let parent = self
            .config_path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let mut temp_file = NamedTempFile::new_in(parent).map_err(|e| {
            PlmError::ImportRegistry(format!("Failed to create temp file: {}", e))
        })?;

        // JSONを書き込み
        let content = serde_json::to_string_pretty(config)?;
        temp_file.write_all(content.as_bytes())?;

        // アトミックに置換
        temp_file.persist(&self.config_path).map_err(|e| {
            PlmError::ImportRegistry(format!("Failed to persist config: {}", e))
        })?;

        self.state = State::Idle;
        Ok(())
    }

    /// インポート履歴一覧を取得
    pub fn list(&mut self) -> Result<Vec<ImportRecord>> {
        if self.state == State::Idle {
            self.load()?;
        }
        Ok(self.config.as_ref().unwrap().imports.clone())
    }

    /// インポートを記録（Loaded → Modified → save → Idle）
    pub fn record(&mut self, record: ImportRecord) -> Result<()> {
        if self.state == State::Idle {
            self.load()?;
        }

        let config = self.config.as_mut().unwrap();
        config.imports.push(record);
        self.state = State::Modified;

        self.save()?;
        Ok(())
    }

    /// 特定ソースからのインポート履歴を取得
    pub fn list_by_source(&mut self, source_repo: &str) -> Result<Vec<ImportRecord>> {
        if self.state == State::Idle {
            self.load()?;
        }

        let records = self
            .config
            .as_ref()
            .unwrap()
            .imports
            .iter()
            .filter(|r| r.source_repo == source_repo)
            .cloned()
            .collect();

        Ok(records)
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
