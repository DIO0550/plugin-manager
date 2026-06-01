//! `ComponentDeployment` で実行する変換種別を表現する enum

use crate::component::convert::{AgentFormat, CommandFormat};
use crate::target::TargetKind;
use std::path::PathBuf;

/// デプロイ時に実行する変換の設定
///
/// 変換が不要な場合 (`Instruction`、フォーマット同一の `Command`/`Agent` 等) は
/// `None` を使う。`Command`/`Agent`/`Hook`/`Skill` の各バリアントは、変換に必要な情報を
/// 型レベルで強制する（旧 Builder の `source_format` / `dest_format` ペア検証を排除）。
/// `Skill` はターゲットによっては frontmatter 調整を伴う（調整不要なターゲットでも
/// `target_kind` を持つバリアントを使い、実際に調整が要るかは配置時に判定する）。
#[derive(Debug, Clone, Default)]
pub enum ConversionConfig {
    #[default]
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
    /// Skill デプロイ時の frontmatter 調整。
    ///
    /// ターゲットがサポートしない `SKILL.md` frontmatter フィールドを除去する。
    /// 制限のないターゲットでは何もしない（ディレクトリをそのままコピーする）。
    Skill { target_kind: TargetKind },
}

#[cfg(test)]
#[path = "conversion_test.rs"]
mod tests;
