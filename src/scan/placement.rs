//! 配置スキャンロジック
//!
//! ターゲットから取得した配置済みアイテム文字列を集約する。
//! ドメイン非依存: 文字列のみに依存。
//!
//! ## 入力形式
//!
//! `target.list_placed()` は以下の形式の文字列を返す:
//! - `<flattened_name>` (フラット 2 階層: Skills, Agents, Commands, Hooks)
//! - `AGENTS.md` / `GEMINI.md` / `copilot-instructions.md` (Instructions)
//!
//! このモジュールは `flattened_name` 集合を返し、Instruction ファイル名は除外する。

use std::collections::HashSet;

/// Instruction として扱う既知のファイル名集合。
const INSTRUCTION_FILE_NAMES: &[&str] = &["AGENTS.md", "copilot-instructions.md", "GEMINI.md"];

/// 配置済みアイテム文字列の中に Instruction ファイル名が含まれているか。
pub fn is_instruction_file(item: &str) -> bool {
    INSTRUCTION_FILE_NAMES.contains(&item)
}

/// 配置済みアイテム文字列のリストから `flattened_name` 集合を抽出する。
///
/// `target.list_placed()` の戻り値をパースし、Instruction ファイルを除外して
/// プラグイン配置の `flattened_name` の重複なし集合を返す。
///
/// # Arguments
///
/// * `placed_items` - `target.list_placed()` の戻り値リスト
pub fn list_placed_components(placed_items: &[String]) -> HashSet<String> {
    placed_items
        .iter()
        .filter(|item| !item.is_empty() && !is_instruction_file(item))
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "placement_test.rs"]
mod tests;
