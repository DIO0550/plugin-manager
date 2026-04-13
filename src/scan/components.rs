//! 低レベルコンポーネントスキャン関数
//!
//! 各コンポーネント種別ごとのスキャン実装。
//! `scan_components` から内部的に呼び出される。

use super::constants::{AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX, SKILL_MANIFEST};
use crate::path_ext::PathExt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// スキル名一覧を取得
///
/// 指定されたディレクトリ配下で `SKILL.md` を持つサブディレクトリを列挙し、
/// そのディレクトリ名を返す。ファイル名の判定はファイルシステムのケース感度に
/// 依存しない厳密一致で行う。
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
pub fn list_skill_names(skills_dir: &Path) -> Vec<(String, PathBuf)> {
    if !skills_dir.is_dir() {
        return Vec::new();
    }

    skills_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_dir() && has_exact_skill_manifest(path))
        .filter_map(|path| {
            let name = path.file_name()?.to_str().map(String::from)?;
            Some((name, path))
        })
        .collect()
}

/// サブディレクトリ内に正確に `SKILL.md` という名前のファイルが存在するか判定する。
///
/// `Path::exists()` はファイルシステムのケース感度に依存するため、
/// `read_dir` で実際のファイル名を取得し、`OsStr` レベルで厳密比較する。
/// `read_dir` が失敗した場合（権限エラー等）は `false` を返す。
fn has_exact_skill_manifest(dir: &Path) -> bool {
    let expected = OsStr::new(SKILL_MANIFEST);
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
        .any(|entry| entry.file_name() == expected)
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
/// エージェント名の一覧。
/// 単一ファイルで名前を導出できない場合や、`agents_path` がファイル/ディレクトリの
/// いずれでもない場合は空配列を返す。
pub fn list_agent_names(agents_path: &Path) -> Vec<(String, PathBuf)> {
    if agents_path.is_file() {
        return file_stem_name(agents_path)
            .map(|name| vec![(name, agents_path.to_path_buf())])
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
            let file_name = path.file_name()?.to_str()?;
            let name = if file_name.ends_with(AGENT_SUFFIX) {
                file_name.trim_end_matches(AGENT_SUFFIX).to_string()
            } else if file_name.ends_with(MARKDOWN_SUFFIX) {
                file_name.trim_end_matches(MARKDOWN_SUFFIX).to_string()
            } else {
                return None;
            };
            Some((name, path))
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
pub fn list_command_names(commands_dir: &Path) -> Vec<(String, PathBuf)> {
    if !commands_dir.is_dir() {
        return Vec::new();
    }

    commands_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?;
            let name = if file_name.ends_with(PROMPT_SUFFIX) {
                file_name.trim_end_matches(PROMPT_SUFFIX).to_string()
            } else if file_name.ends_with(MARKDOWN_SUFFIX) {
                file_name.trim_end_matches(MARKDOWN_SUFFIX).to_string()
            } else {
                return None;
            };
            Some((name, path))
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
pub fn list_hook_names(hooks_dir: &Path) -> Vec<(String, PathBuf)> {
    if !hooks_dir.is_dir() {
        return Vec::new();
    }

    hooks_dir
        .read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?;
            let hook_name = file_name
                .rsplit_once('.')
                .map(|(n, _)| n.to_string())
                .unwrap_or_else(|| file_name.to_string());
            Some((hook_name, path))
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
pub fn list_markdown_names(dir: &Path) -> Vec<(String, PathBuf)> {
    if !dir.is_dir() {
        return Vec::new();
    }

    dir.read_dir_entries()
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?;
            if file_name.ends_with(MARKDOWN_SUFFIX) {
                let name = file_name.trim_end_matches(MARKDOWN_SUFFIX).to_string();
                Some((name, path))
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
