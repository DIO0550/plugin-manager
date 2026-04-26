//! 低レベルコンポーネントスキャン関数
//!
//! 各コンポーネント種別ごとのスキャン実装。
//! `scan_components` から内部的に呼び出される。

use super::constants::{AGENT_SUFFIX, MARKDOWN_SUFFIX, PROMPT_SUFFIX, SKILL_MANIFEST};
use crate::path_ext::PathExt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// スキル名一覧を取得（再帰）
///
/// `skills_dir` 配下を再帰的に走査し、`SKILL.md` を直下に持つディレクトリを
/// すべて採用する。中間ディレクトリ名は戻り値に含めない（例: `bar/foo/SKILL.md`
/// は `("foo", path)` を返す）。
///
/// # Arguments
///
/// * `skills_dir` - スキルディレクトリのパス
///
/// # Returns
/// `(SKILL.md があるディレクトリ名, そのディレクトリの絶対パス)` の一覧。
/// 順序は保証されない。
///
/// # Behavior
/// - `skills_dir` がディレクトリでない場合は空配列を返す
/// - SKILL.md を持つディレクトリを発見したらそれを採用し、配下にはさらに潜らない
///   （Skill 内部の `assets/` 等に SKILL.md がある場合の誤検出を避ける）
/// - UTF-8 変換不可のディレクトリ名は除外
pub fn list_skill_names(skills_dir: &Path) -> Vec<(String, PathBuf)> {
    if !skills_dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    collect_skills_recursive(skills_dir, &mut out);
    out
}

fn collect_skills_recursive(current: &Path, out: &mut Vec<(String, PathBuf)>) {
    for entry in current.read_dir_entries() {
        // symlink は無限ループ防止のため辿らない
        if is_symlink(&entry) {
            continue;
        }
        if !entry.is_dir() {
            continue;
        }
        let Some(entry_name) = entry.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        if has_exact_skill_manifest(&entry) {
            out.push((entry_name, entry));
            continue;
        }
        collect_skills_recursive(&entry, out);
    }
}

/// symlink かどうかを判定する。`symlink_metadata` を使うため symlink 先は辿らない。
/// メタデータ取得に失敗した場合は false（通常エントリ扱い）として呼び出し元の `is_dir`
/// 判定に委ねる。
fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
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

/// エージェント名一覧を取得（再帰）
///
/// 指定されたパスが単一ファイルの場合はそのファイル名（拡張子除去）を返す。
/// ディレクトリの場合は配下を再帰的に走査し、`.agent.md` / `.md` ファイルを
/// 列挙する。中間ディレクトリ名は戻り値に含めない。
///
/// # Arguments
///
/// * `agents_path` - エージェントファイルまたはディレクトリのパス
///
/// # Returns
/// `(エージェント名, ファイルパス)` の一覧。
/// 単一ファイルで名前を導出できない場合や、`agents_path` がファイル/ディレクトリの
/// いずれでもない場合は空配列を返す。
pub fn list_agent_names(agents_path: &Path) -> Vec<(String, PathBuf)> {
    if agents_path.is_file() {
        let Some(file_name) = agents_path.file_name().and_then(|n| n.to_str()) else {
            return Vec::new();
        };
        let Some(name) = file_name
            .strip_suffix(AGENT_SUFFIX)
            .or_else(|| file_name.strip_suffix(MARKDOWN_SUFFIX))
            .map(String::from)
        else {
            // .agent.md / .md 以外の拡張子は対象外（ディレクトリ走査と挙動を揃える）
            return Vec::new();
        };
        return vec![(name, agents_path.to_path_buf())];
    }

    if !agents_path.is_dir() {
        return Vec::new();
    }

    let mut out = Vec::new();
    collect_component_files_recursive(agents_path, AGENT_SUFFIX, true, &mut out);
    out
}

/// `current` 配下を再帰走査し、`primary_suffix` で終わるファイルを採取する。
///
/// - `primary_suffix` で終わるファイルは、その suffix を取り除いた stem を name とする。
/// - `is_root == true` のディレクトリ直下に限り、`.md` で終わるファイルも採取する
///   （既存の Claude Code 互換: 直下 `agents/foo.md` 等）。
///   ネスト先の `.md` は誤検出回避のため対象外とする。
/// - その他のファイルは無視する。
/// - サブディレクトリは再帰的に走査する（descended into のため `is_root = false`）。
fn collect_component_files_recursive(
    current: &Path,
    primary_suffix: &str,
    is_root: bool,
    out: &mut Vec<(String, PathBuf)>,
) {
    for entry in current.read_dir_entries() {
        // symlink は無限ループ防止のため辿らない（symlink 先がファイルでも対象外）
        if is_symlink(&entry) {
            continue;
        }
        if entry.is_file() {
            let Some(file_name) = entry.file_name().and_then(|n| n.to_str()).map(String::from)
            else {
                continue;
            };
            if let Some(stem) = file_name.strip_suffix(primary_suffix) {
                out.push((stem.to_string(), entry));
                continue;
            }
            if is_root {
                if let Some(stem) = file_name.strip_suffix(MARKDOWN_SUFFIX) {
                    out.push((stem.to_string(), entry));
                }
            }
            continue;
        }
        if entry.is_dir() {
            collect_component_files_recursive(&entry, primary_suffix, false, out);
        }
    }
}

/// コマンド名一覧を取得（再帰）
///
/// `commands_dir` 配下を再帰的に走査し、`.prompt.md` / `.md` ファイルを列挙する。
/// 中間ディレクトリ名は戻り値に含めない。
///
/// # Arguments
///
/// * `commands_dir` - コマンドディレクトリのパス
///
/// # Returns
/// `(コマンド名, ファイルパス)` の一覧。
pub fn list_command_names(commands_dir: &Path) -> Vec<(String, PathBuf)> {
    if !commands_dir.is_dir() {
        return Vec::new();
    }

    let mut out = Vec::new();
    collect_component_files_recursive(commands_dir, PROMPT_SUFFIX, true, &mut out);
    out
}

/// フック名一覧を取得
///
/// ディレクトリ内のファイルを列挙し、拡張子を除去した名前を返す。
///
/// # Arguments
///
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
///
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
///
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
