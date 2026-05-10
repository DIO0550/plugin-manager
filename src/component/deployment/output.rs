//! デプロイ実行結果の型定義
//!
//! `ComponentDeployment::execute()` の戻り値として使用される。
//! `Result` 系名（`std::result::Result`）との混同を避けるため
//! `~Output` 命名を採用している（`std::process::Output` などの慣用に倣う）。

use crate::component::convert::{AgentConversionResult, ConversionOutcome};
use crate::hooks::converter::{ConversionWarning, SourceFormat};

/// デプロイ結果
#[derive(Debug)]
pub enum DeploymentOutput {
    /// ファイルコピーのみ
    Copied,
    /// Command フォーマット変換が行われた
    CommandConverted(ConversionOutcome),
    /// Agent フォーマット変換が行われた
    AgentConverted(AgentConversionResult),
    /// Hook 変換が行われた
    HookConverted(HookConvertOutput),
}

/// Hook 変換結果
#[derive(Debug)]
pub struct HookConvertOutput {
    pub warnings: Vec<ConversionWarning>,
    pub script_count: usize,
    pub hook_count: usize,
    /// 入力 JSON が Claude Code 形式だったか Copilot 形式（passthrough）だったか。
    /// `(converted from Claude Code format)` サフィックスの判定に使う。
    pub source_format: SourceFormat,
}

impl std::fmt::Display for DeploymentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentOutput::Copied => write!(f, "Copied"),
            DeploymentOutput::CommandConverted(conv) => {
                if conv.converted {
                    write!(
                        f,
                        "Converted: {} → {}",
                        conv.source_format, conv.dest_format
                    )
                } else {
                    write!(f, "Copied (no conversion needed)")
                }
            }
            DeploymentOutput::AgentConverted(conv) => {
                if conv.converted {
                    write!(
                        f,
                        "Agent converted: {} → {}",
                        conv.source_format, conv.dest_format
                    )
                } else {
                    write!(f, "Copied (no agent conversion needed)")
                }
            }
            DeploymentOutput::HookConverted(hr) => {
                write!(
                    f,
                    "Hook converted ({} script{}, {} warning{})",
                    hr.script_count,
                    if hr.script_count == 1 { "" } else { "s" },
                    hr.warnings.len(),
                    if hr.warnings.len() == 1 { "" } else { "s" }
                )
            }
        }
    }
}

#[cfg(test)]
#[path = "output_test.rs"]
mod tests;
