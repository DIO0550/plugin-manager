//! `ComponentDeployment` で実行する変換種別を表現する enum

use crate::component::convert::{AgentFormat, CommandFormat};
use crate::target::TargetKind;
use std::path::PathBuf;

/// デプロイ時に実行する変換の設定
///
/// 変換が不要な場合 (`Skill`/`Instruction`、フォーマット同一の `Command`/`Agent` 等) は
/// `None` を使う。`Command`/`Agent`/`Hook` の各バリアントは、変換に必要な情報を
/// 型レベルで強制する（旧 Builder の `source_format` / `dest_format` ペア検証を排除）。
#[derive(Debug, Clone)]
pub enum ConversionConfig {
    None,
    Command {
        source: CommandFormat,
        dest: CommandFormat,
    },
    Agent {
        source: AgentFormat,
        dest: AgentFormat,
    },
    Hook {
        target_kind: TargetKind,
        plugin_root: Option<PathBuf>,
    },
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
#[path = "conversion_test.rs"]
mod tests;
