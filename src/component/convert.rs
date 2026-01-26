//! Command コンポーネントのフォーマット変換モジュール
//!
//! install/sync コマンドで Command コンポーネントを配置する際に、
//! ソース形式からターゲット形式への自動変換を行う。

use crate::error::{PlmError, Result};
use crate::parser::{ClaudeCodeCommand, CodexPrompt, CopilotPrompt};
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

/// 変換結果
#[derive(Debug)]
pub struct ConversionResult {
    /// 変換が行われたか（false = コピーのみ）
    pub converted: bool,
    /// ソース形式
    pub source_format: CommandFormat,
    /// 出力形式
    pub dest_format: CommandFormat,
}

/// Command を変換して書き込む
///
/// ## 変換ロジック
///
/// - 同一形式: ファイルをそのままコピー
/// - 異なる形式: パース → 変換 → シリアライズ
///
/// ## 多段変換
///
/// Copilot ↔ Codex の変換は ClaudeCode を経由する（メモリ上で変換）：
/// - Copilot → ClaudeCode → Codex
/// - Codex → ClaudeCode → Copilot
///
/// ## アトミック書き込み
///
/// 変換成功時のみファイルを配置するため、一時ファイル経由で書き込む。
pub fn convert_and_write(
    source_path: &Path,
    dest_path: &Path,
    source_format: CommandFormat,
    dest_format: CommandFormat,
) -> Result<ConversionResult> {
    // 同一形式ならコピーのみ
    if source_format == dest_format {
        copy_file(source_path, dest_path)?;
        return Ok(ConversionResult {
            converted: false,
            source_format,
            dest_format,
        });
    }

    // 1. ソース読み込み
    let content = fs::read_to_string(source_path)?;

    // 2. パース & 変換 & シリアライズ
    let markdown = convert_content(&content, source_format, dest_format)?;

    // 3. アトミック書き込み
    atomic_write(dest_path, &markdown)?;

    Ok(ConversionResult {
        converted: true,
        source_format,
        dest_format,
    })
}

/// コンテンツを変換する（内部用）
fn convert_content(
    content: &str,
    source_format: CommandFormat,
    dest_format: CommandFormat,
) -> Result<String> {
    use CommandFormat::*;

    let markdown = match (source_format, dest_format) {
        // ClaudeCode → 他形式
        (ClaudeCode, Copilot) => {
            let cmd = ClaudeCodeCommand::parse(content)?;
            CopilotPrompt::from(&cmd).to_markdown()
        }
        (ClaudeCode, Codex) => {
            let cmd = ClaudeCodeCommand::parse(content)?;
            CodexPrompt::from(&cmd).to_markdown()
        }

        // Copilot → 他形式
        (Copilot, ClaudeCode) => {
            let prompt = CopilotPrompt::parse(content)?;
            ClaudeCodeCommand::from(&prompt).to_markdown()
        }
        (Copilot, Codex) => {
            // 多段変換: Copilot → ClaudeCode → Codex
            let prompt = CopilotPrompt::parse(content)?;
            let cmd = ClaudeCodeCommand::from(&prompt);
            CodexPrompt::from(&cmd).to_markdown()
        }

        // Codex → 他形式
        (Codex, ClaudeCode) => {
            let prompt = CodexPrompt::parse(content)?;
            ClaudeCodeCommand::from(&prompt).to_markdown()
        }
        (Codex, Copilot) => {
            // 多段変換: Codex → ClaudeCode → Copilot
            let prompt = CodexPrompt::parse(content)?;
            let cmd = ClaudeCodeCommand::from(&prompt);
            CopilotPrompt::from(&cmd).to_markdown()
        }

        // 同一形式（上で処理済み）
        (ClaudeCode, ClaudeCode) | (Copilot, Copilot) | (Codex, Codex) => {
            unreachable!("Same format should be handled by caller")
        }
    };

    Ok(markdown)
}

/// ファイルをコピー（親ディレクトリを作成）
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
fn atomic_write(dest_path: &Path, content: &str) -> Result<()> {
    // 親ディレクトリを作成
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 一時ファイル名: <filename>.tmp
    let tmp_path = dest_path.with_extension(format!(
        "{}.tmp",
        dest_path.extension().unwrap_or_default().to_string_lossy()
    ));

    // 一時ファイルに書き込み
    if let Err(e) = fs::write(&tmp_path, content) {
        return Err(PlmError::Io(e));
    }

    // アトミックに移動
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
