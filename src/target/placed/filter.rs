//! 配置済みスキャン結果の共通フィルタ
//!
//! 各 env の `filter_component` から抽出した同型アーム。

use crate::target::scanner::ScannedComponent;

/// SKILL.md を含むディレクトリを Skill として認識する（全ターゲット共通）。
pub(crate) fn filter_skill_dir(c: &ScannedComponent) -> Option<String> {
    if c.is_dir && c.path.join("SKILL.md").is_file() {
        Some(c.name.clone())
    } else {
        None
    }
}

/// `ends_with(suffix)` のファイルから suffix を除いた名前を返す。
pub(crate) fn filter_suffix_file(c: &ScannedComponent, suffix: &str) -> Option<String> {
    if !c.is_dir && c.name.ends_with(suffix) {
        Some(c.name.trim_end_matches(suffix).to_string())
    } else {
        None
    }
}

/// プレーン `.md`（`.agent.md` / `.prompt.md` 除外）→ `.md` 除去名（Cursor）。
pub(crate) fn filter_plain_markdown(c: &ScannedComponent) -> Option<String> {
    if !c.is_dir && is_plain_markdown(&c.name) {
        Some(c.name.trim_end_matches(".md").to_string())
    } else {
        None
    }
}

/// ファイル名完全一致 → 固定エイリアス（例: `hooks.json` → `"hooks"`）。
pub(crate) fn filter_exact_file(
    c: &ScannedComponent,
    filename: &str,
    listed_as: &str,
) -> Option<String> {
    if !c.is_dir && c.name == filename {
        Some(listed_as.to_string())
    } else {
        None
    }
}

/// `*.json` ファイル → `.json` 除去名（Copilot Hook）。
pub(crate) fn filter_json_suffix(c: &ScannedComponent) -> Option<String> {
    filter_suffix_file(c, ".json")
}

fn is_plain_markdown(name: &str) -> bool {
    name.ends_with(".md") && !name.ends_with(".agent.md") && !name.ends_with(".prompt.md")
}

#[cfg(test)]
#[path = "filter_test.rs"]
mod tests;
