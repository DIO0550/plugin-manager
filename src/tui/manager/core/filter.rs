//! フィルタロジック
//!
//! プラグイン一覧をフィルタテキストで絞り込む共通モジュール。

use crate::application::InstalledPlugin;

/// フィルタテキストでプラグインを絞り込む
///
/// - `filter_text` が空なら全件返却
/// - case-insensitive な部分一致で `name` と `marketplace` を検索
///
/// # Arguments
///
/// * `plugins` - 絞り込み対象のインストール済みプラグイン一覧
/// * `filter_text` - 検索クエリ文字列。空なら全件をそのまま返す
pub fn filter_plugins<'a>(
    plugins: &'a [InstalledPlugin],
    filter_text: &str,
) -> Vec<&'a InstalledPlugin> {
    if filter_text.is_empty() {
        return plugins.iter().collect();
    }

    let query = filter_text.to_lowercase();
    plugins
        .iter()
        .filter(|p| {
            p.name().to_lowercase().contains(&query)
                || p.marketplace()
                    .is_some_and(|m| m.to_lowercase().contains(&query))
        })
        .collect()
}
