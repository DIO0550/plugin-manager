//! Command コンポーネントのフォーマット変換モジュール
//!
//! install/sync コマンドで Command コンポーネントを配置する際に、
//! ソース形式からターゲット形式への自動変換を行う。

use crate::error::{PlmError, Result};
use crate::parser::{ClaudeCodeAgent, ClaudeCodeCommand, TargetType};
use std::fs;
use std::path::Path;

/// コマンドファイルのフォーマット種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFormat {
    /// Claude Code: `.claude/commands/<name>.md`
    ClaudeCode,
    /// Copilot: `.github/prompts/<name>.prompt.md`
    Copilot,
    /// Codex: `~/.codex/prompts/<name>.md`
    Codex,
}

impl std::fmt::Display for CommandFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandFormat::ClaudeCode => write!(f, "ClaudeCode"),
            CommandFormat::Copilot => write!(f, "Copilot"),
            CommandFormat::Codex => write!(f, "Codex"),
        }
    }
}

/// Agent ファイルのフォーマット種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFormat {
    /// Claude Code: `.claude/agents/<name>.md`
    ClaudeCode,
    /// Copilot: `.github/agents/<name>.agent.md`
    Copilot,
    /// Codex: `.codex/agents/<name>.agent.md`
    Codex,
}

impl std::fmt::Display for AgentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentFormat::ClaudeCode => write!(f, "ClaudeCode"),
            AgentFormat::Copilot => write!(f, "Copilot"),
            AgentFormat::Codex => write!(f, "Codex"),
        }
    }
}

/// Command 変換結果（成果レポート）
#[derive(Debug)]
pub struct ConversionOutcome {
    /// 変換が行われたか（false = コピーのみ）
    pub converted: bool,
    /// ソース形式
    pub source_format: CommandFormat,
    /// 出力形式
    pub dest_format: CommandFormat,
}

/// Agent 変換結果
#[derive(Debug)]
pub struct AgentConversionOutcome {
    /// 変換が行われたか（false = コピーのみ）
    pub converted: bool,
    /// ソース形式
    pub source_format: AgentFormat,
    /// 出力形式
    pub dest_format: AgentFormat,
}

/// Command を変換して書き込む
///
/// ## 変換ロジック
///
/// - 同一形式: ファイルをそのままコピー
/// - ClaudeCode → 他形式: パース → 変換 → シリアライズ
/// - 他形式 → 任意: UnsupportedConversion エラー
///
/// ## サポートする変換
///
/// - `ClaudeCode → Copilot`
/// - `ClaudeCode → Codex`
///
/// ## アトミック書き込み
///
/// 変換成功時のみファイルを配置するため、一時ファイル経由で書き込む。
///
/// # Arguments
///
/// * `source_path` - Path to the source command file to read.
/// * `dest_path` - Destination path to write the (possibly converted) command file to.
/// * `source_format` - Format of the source file.
/// * `dest_format` - Desired format of the destination file.
pub fn convert_and_write(
    source_path: &Path,
    dest_path: &Path,
    source_format: CommandFormat,
    dest_format: CommandFormat,
) -> Result<ConversionOutcome> {
    if source_format == dest_format {
        copy_file(source_path, dest_path)?;
        return Ok(ConversionOutcome {
            converted: false,
            source_format,
            dest_format,
        });
    }

    let content = fs::read_to_string(source_path)?;
    let markdown = convert_content(&content, source_format, dest_format)?;
    atomic_write(dest_path, &markdown)?;

    Ok(ConversionOutcome {
        converted: true,
        source_format,
        dest_format,
    })
}

/// コンテンツを変換する（内部用）
///
/// ClaudeCode からのみ変換可能。他の形式からの変換は UnsupportedConversion エラー。
///
/// # Arguments
///
/// * `content` - Raw markdown content to convert.
/// * `source_format` - Format of the provided `content`.
/// * `dest_format` - Target format to convert into.
fn convert_content(
    content: &str,
    source_format: CommandFormat,
    dest_format: CommandFormat,
) -> Result<String> {
    if source_format != CommandFormat::ClaudeCode {
        return Err(PlmError::UnsupportedConversion {
            from: source_format.to_string(),
            to: dest_format.to_string(),
        });
    }

    let cmd = ClaudeCodeCommand::parse(content)?;
    let target_type = match dest_format {
        CommandFormat::Copilot => TargetType::Copilot,
        CommandFormat::Codex => TargetType::Codex,
        CommandFormat::ClaudeCode => unreachable!("Same format should be handled by caller"),
    };

    Ok(cmd.to_format(target_type)?.to_markdown())
}

/// Agent を変換して書き込む
///
/// ## 変換ロジック
///
/// - 同一形式: ファイルをそのままコピー
/// - ClaudeCode → 他形式: パース → 変換 → シリアライズ
/// - 他形式 → 任意: UnsupportedConversion エラー
///
/// ## サポートする変換
///
/// - `ClaudeCode → Copilot`
/// - `ClaudeCode → Codex`
///
/// # Arguments
///
/// * `source_path` - Path to the source agent file to read.
/// * `dest_path` - Destination path to write the (possibly converted) agent file to.
/// * `source_format` - Format of the source file.
/// * `dest_format` - Desired format of the destination file.
pub fn convert_agent_and_write(
    source_path: &Path,
    dest_path: &Path,
    source_format: AgentFormat,
    dest_format: AgentFormat,
) -> Result<AgentConversionOutcome> {
    if source_format == dest_format {
        copy_file(source_path, dest_path)?;
        return Ok(AgentConversionOutcome {
            converted: false,
            source_format,
            dest_format,
        });
    }

    let content = fs::read_to_string(source_path)?;
    let markdown = convert_agent_content(&content, source_format, dest_format)?;
    atomic_write(dest_path, &markdown)?;

    Ok(AgentConversionOutcome {
        converted: true,
        source_format,
        dest_format,
    })
}

/// Agent コンテンツを変換する（内部用）
///
/// ClaudeCode からのみ変換可能。他の形式からの変換は UnsupportedConversion エラー。
///
/// # Arguments
///
/// * `content` - Raw markdown content to convert.
/// * `source_format` - Format of the provided `content`.
/// * `dest_format` - Target format to convert into.
fn convert_agent_content(
    content: &str,
    source_format: AgentFormat,
    dest_format: AgentFormat,
) -> Result<String> {
    if source_format != AgentFormat::ClaudeCode {
        return Err(PlmError::UnsupportedConversion {
            from: source_format.to_string(),
            to: dest_format.to_string(),
        });
    }

    let agent = ClaudeCodeAgent::parse(content)?;
    let target_type = match dest_format {
        AgentFormat::Copilot => TargetType::Copilot,
        AgentFormat::Codex => TargetType::Codex,
        AgentFormat::ClaudeCode => unreachable!("Same format should be handled by caller"),
    };

    Ok(agent.to_format(target_type)?.to_markdown())
}

/// ファイルをコピー（親ディレクトリを作成）
///
/// # Arguments
///
/// * `source` - Source file to copy from.
/// * `dest` - Destination path to copy to; parent directories are created as needed.
fn copy_file(source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, dest)?;
    Ok(())
}

/// アトミック書き込み（一時ファイル → rename）
///
/// ターゲットと同一ディレクトリに一時ファイルを作成し、
/// 書き込み成功後に rename でアトミックに移動する。
///
/// # Arguments
///
/// * `dest_path` - Final destination path where the content should end up.
/// * `content` - Content to write atomically.
fn atomic_write(dest_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = dest_path.with_extension(format!(
        "{}.tmp",
        dest_path.extension().unwrap_or_default().to_string_lossy()
    ));

    if let Err(e) = fs::write(&tmp_path, content) {
        return Err(PlmError::Io(e));
    }

    if let Err(e) = fs::rename(&tmp_path, dest_path) {
        // 失敗時は一時ファイルを削除
        let _ = fs::remove_file(&tmp_path);
        return Err(PlmError::Io(e));
    }

    Ok(())
}

#[cfg(test)]
#[path = "convert_test.rs"]
mod tests;
