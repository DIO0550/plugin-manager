//! 低レベルコンポーネントスキャン関数
//!
//! 各コンポーネント種別ごとのスキャン実装。
//! `scan_components` から内部的に呼び出される。

use super::constants::{AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX, SKILL_MANIFEST};
use crate::path_ext::PathExt;
use std::path::Path;

/// スキル名一覧を取得
///
/// 指定されたディレクトリ配下で `SKILL.md` を持つサブディレクトリを列挙し、
/// そのディレクトリ名を返す。
///
/// # Arguments
/// * `skills_dir` - スキルディレクトリのパス
///
/// # Returns
/// スキル名（ディレクトリ名）の一覧。順序は保証されない。
///
/// # Behavior
/// - `skills_dir` がディレクトリでない場合は空配列を返す
/// - サブディレクトリのうち `SKILL.md` が存在するものだけを抽出
/// - UTF-8 変換不可のディレクトリ名は除外
pub fn list_skill_names(skills_dir: &Path) -> Vec<String> {
    if !skills_dir.is_dir() {
        return Vec::new();
    }

    skills_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_dir() && path.join(SKILL_MANIFEST).exists())
        .filter_map(|path| path.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect()
}

/// エージェント名一覧を取得
///
/// 指定されたパスが単一ファイルの場合はそのファイル名（拡張子除去）を返す。
/// ディレクトリの場合は .agent.md / .md ファイルを列挙する。
///
/// # Arguments
/// * `agents_path` - エージェントファイルまたはディレクトリのパス
///
/// # Returns
/// エージェント名の一覧。単一ファイルの場合は None を返す可能性がある
/// （呼び出し側でフォールバック処理が必要）。
pub fn list_agent_names(agents_path: &Path) -> Vec<String> {
    // 単一ファイルの場合
    if agents_path.is_file() {
        return file_stem_name(agents_path)
            .map(|name| vec![name])
            .unwrap_or_default();
    }

    if !agents_path.is_dir() {
        return Vec::new();
    }

    agents_path
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(AGENT_SUFFIX) {
                Some(name.trim_end_matches(AGENT_SUFFIX).to_string())
            } else if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// コマンド名一覧を取得
///
/// ディレクトリ内の .prompt.md / .md ファイルを列挙する。
///
/// # Arguments
/// * `commands_dir` - コマンドディレクトリのパス
///
/// # Returns
/// コマンド名の一覧。
pub fn list_command_names(commands_dir: &Path) -> Vec<String> {
    if !commands_dir.is_dir() {
        return Vec::new();
    }

    commands_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(PROMPT_SUFFIX) {
                Some(name.trim_end_matches(PROMPT_SUFFIX).to_string())
            } else if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// フック名一覧を取得
///
/// ディレクトリ内のファイルを列挙し、拡張子を除去した名前を返す。
///
/// # Arguments
/// * `hooks_dir` - フックディレクトリのパス
///
/// # Returns
/// フック名の一覧。
pub fn list_hook_names(hooks_dir: &Path) -> Vec<String> {
    if !hooks_dir.is_dir() {
        return Vec::new();
    }

    hooks_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            // 拡張子を除去（複数ドットの場合は最後のみ）
            let hook_name = name
                .rsplit_once('.')
                .map(|(n, _)| n.to_string())
                .unwrap_or_else(|| name.to_string());
            Some(hook_name)
        })
        .collect()
}

/// Markdown ファイル名一覧を取得
///
/// ディレクトリ内の .md ファイルを列挙し、拡張子を除去した名前を返す。
///
/// # Arguments
/// * `dir` - 対象ディレクトリのパス
///
/// # Returns
/// Markdown ファイル名（拡張子除去済み）の一覧。
pub fn list_markdown_names(dir: &Path) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }

    dir.read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            if name.ends_with(MARKDOWN_SUFFIX) {
                Some(name.trim_end_matches(MARKDOWN_SUFFIX).to_string())
            } else {
                None
            }
        })
        .collect()
}

/// パスからファイル名（拡張子除去）を取得
///
/// # Arguments
/// * `path` - ファイルパス
///
/// # Returns
/// ファイル名（拡張子除去済み）。UTF-8変換不可の場合は None。
pub fn file_stem_name(path: &Path) -> Option<String> {
    path.file_stem().and_then(|s| s.to_str()).map(String::from)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptests;
