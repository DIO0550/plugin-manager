//! 配置スキャンロジック
//!
//! ターゲットから取得した配置済みアイテム文字列をパースする。
//! ドメイン非依存: 文字列のみに依存。
//!
//! ## 入力形式
//!
//! `target.list_placed()` は以下の形式の文字列を返す:
//! - `"marketplace/plugin/component"` (Skills, Agents)
//! - `"AGENTS.md"` (Instructions)
//!
//! この文字列は常に `/` 区切り（OS のパス区切りではない）。

use std::collections::HashSet;

/// 配置済みアイテム文字列のリストから (marketplace, plugin_name) ペアを抽出
///
/// `target.list_placed()` の戻り値をパースし、プラグイン単位で集約する。
///
/// # Arguments
/// * `placed_items` - `target.list_placed()` の戻り値リスト
///
/// # Returns
/// (marketplace, plugin_name) ペアの集合
pub fn list_placed_plugins(placed_items: &[String]) -> HashSet<(String, String)> {
    placed_items
        .iter()
        .filter_map(|item| parse_placement(item))
        .collect()
}

/// "marketplace/plugin/..." 形式をパース
///
/// # Arguments
/// * `item` - パース対象の文字列
///
/// # Returns
/// * `Some((marketplace, plugin_name))` - 2セグメント以上かつ両方非空の場合
/// * `None` - 以下のいずれかの場合:
///   - 1セグメント以下（スラッシュなし）
///   - marketplace または plugin_name が空文字
///
/// # Edge Cases
/// * `""` → `None`
/// * `"plugin"` → `None`
/// * `"marketplace/plugin"` → `Some(("marketplace", "plugin"))`
/// * `"marketplace/plugin/skill"` → `Some(("marketplace", "plugin"))`
/// * `"/plugin"` → `None`
/// * `"marketplace/"` → `None`
pub fn parse_placement(item: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = item.split('/').collect();
    if parts.len() >= 2 {
        let marketplace = parts[0];
        let plugin = parts[1];
        if marketplace.is_empty() || plugin.is_empty() {
            return None;
        }
        Some((marketplace.to_string(), plugin.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "placement_test.rs"]
mod tests;
