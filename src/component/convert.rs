//! Command コンポーネントのフォーマット変換モジュール
//!
//! install/sync コマンドで Command コンポーネントを配置する際に、
//! ソース形式からターゲット形式への自動変換を行う。

use crate::error::{PlmError, Result};
use crate::parser::{ClaudeCodeAgent, ClaudeCodeCommand, TargetType};
use crate::target::TargetKind;
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
pub(crate) fn atomic_write(dest_path: &Path, content: &str) -> Result<()> {
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

/// 指定ターゲットの Skill `SKILL.md` frontmatter で保持すべき top-level フィールド。
///
/// `Some(fields)` を返すターゲットでは、`fields` 以外の top-level フィールドを
/// デプロイ時に除去する。`None` は「制限なし（frontmatter をそのまま保持）」を表す。
///
/// Codex の Skill frontmatter は公式に `name` / `description` / `metadata` のみ対応する。
/// Gemini CLI はさらに限定的で `name` / `description` のみ対応する。いずれもサポート外
/// フィールド（`allowed-tools` / `disable-model-invocation` / `argument-hint` / `model` /
/// `context` 等）が残ると読み込みエラーになりうるため、デプロイ時に取り除く。
/// 特に不正な YAML 値（例: `argument-hint: [a] [b]`）を含むと SKILL.md 全体が
/// パース不能になる。
///
/// Copilot は Codex / Claude Code と共通形式のため制限しない（`None`）。
///
/// # Arguments
///
/// * `target` - 配置先ターゲット種別。
pub fn skill_allowed_fields(target: TargetKind) -> Option<&'static [&'static str]> {
    match target {
        TargetKind::Codex => Some(&["name", "description", "metadata"]),
        TargetKind::GeminiCli => Some(&["name", "description"]),
        TargetKind::Antigravity | TargetKind::Copilot => None,
    }
}

/// `SKILL.md` の内容から、`allowed` に含まれない top-level frontmatter フィールドを除去する。
///
/// frontmatter 全体を YAML として再パースせず、**行ベース**で top-level キーを判定して
/// 除去する。これは、サポート外フィールドが不正な YAML 値を持つ場合（例:
/// `argument-hint: [threshold] [min-lines]` はフローシーケンスとして解釈され壊れる）でも、
/// 該当行を安全に取り除けるようにするためである。
///
/// # 判定ルール
///
/// - 先頭が `---` の行で開始し、以降の `---` 行までを frontmatter とみなす
/// - frontmatter がない、または閉じ `---` がない場合は内容をそのまま返す
/// - インデントなしで `key:` の形を持つ行を top-level キーとみなす
/// - top-level キー行に続くインデント行・空行・コメント行・リスト行は、直前の
///   top-level キーのブロックの一部として扱う（`metadata:` 配下のネストを保持できる）
/// - 本文（閉じ `---` 以降）はバイト単位でそのまま保持する。オフセットは行終端を
///   含めたまま分割（`split_inclusive('\n')`）して算出するため、LF / CRLF いずれの
///   改行コードでも本文を壊さない（保持する frontmatter 行の改行は LF に正規化される）
///
/// # Arguments
///
/// * `content` - `SKILL.md` の全内容。
/// * `allowed` - 保持する top-level フィールド名の一覧。
pub fn strip_skill_frontmatter_fields(content: &str, allowed: &[&str]) -> String {
    let stripped = content.strip_prefix('\u{feff}').unwrap_or(content);
    let bom = &content[..content.len() - stripped.len()];

    // 行終端 (`\n`) を含めたまま分割する。各セグメントのバイト長を合計すれば、
    // LF / CRLF を問わず正確な本文オフセットを算出できる。
    let segments: Vec<&str> = stripped.split_inclusive('\n').collect();

    let first_is_fence = segments.first().map(|s| line_text(s).trim()).unwrap_or("") == "---";
    if !first_is_fence {
        return content.to_string();
    }

    let closing_index = segments
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, seg)| line_text(seg).trim() == "---")
        .map(|(i, _)| i);

    let Some(closing_index) = closing_index else {
        return content.to_string();
    };

    // frontmatter 行（開始/終了 `---` の間）をフィルタリング
    let mut kept_frontmatter: Vec<&str> = Vec::new();
    // 最初の top-level キーより前の行（コメント等）は保持する
    let mut keep_current_block = true;
    for seg in &segments[1..closing_index] {
        let line = line_text(seg);
        if let Some(key) = top_level_frontmatter_key(line) {
            keep_current_block = allowed.contains(&key);
        }
        if keep_current_block {
            kept_frontmatter.push(line);
        }
    }

    // 本文（閉じ `---` セグメントの直後）はバイト単位でそのまま保持する。
    let body_offset: usize = bom.len()
        + segments[..=closing_index]
            .iter()
            .map(|s| s.len())
            .sum::<usize>();
    let body = if body_offset <= content.len() {
        &content[body_offset..]
    } else {
        ""
    };

    let mut result = String::with_capacity(content.len());
    result.push_str(bom);
    result.push_str(line_text(segments[0]));
    result.push('\n');
    for line in kept_frontmatter {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str(line_text(segments[closing_index]));
    result.push('\n');
    result.push_str(body);
    result
}

/// `split_inclusive('\n')` のセグメントから、末尾の改行 (`\r\n` / `\n`) を除いた
/// 行内容を取り出す。
fn line_text(segment: &str) -> &str {
    let without_lf = segment.strip_suffix('\n').unwrap_or(segment);
    without_lf.strip_suffix('\r').unwrap_or(without_lf)
}

/// frontmatter の 1 行から top-level マッピングキーを取り出す。
///
/// インデントなしで `key:` の形を持つ行のみをキーとみなす。インデント行・空行・
/// コメント行（`#`）・リスト行（`-`）は継続行として `None` を返す。
fn top_level_frontmatter_key(line: &str) -> Option<&str> {
    let first = line.as_bytes().first().copied()?;
    if matches!(first, b' ' | b'\t' | b'#' | b'-') {
        return None;
    }
    let colon = line.find(':')?;
    let key = &line[..colon];
    if !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Some(key)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "convert_test.rs"]
mod tests;
