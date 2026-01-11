//! ターゲット選択TUIダイアログ

use crate::component::ComponentKind;
use crate::error::{PlmError, Result};
use crate::target::Target;
use crate::tui::dialog::{multi_select, SelectItem};

/// ターゲット選択ダイアログを表示
///
/// プラグインのコンポーネント種別に基づき、対応するターゲットをプリセレクトする。
/// サポートしていないターゲットはグレーアウトして無効化する。
pub fn select_targets(
    available_targets: &[&dyn Target],
    plugin_components: &[ComponentKind],
) -> Result<Vec<String>> {
    let mut items: Vec<SelectItem<String>> = available_targets
        .iter()
        .map(|target| {
            // このターゲットがサポートするプラグインコンポーネントを確認
            let supported: Vec<&str> = plugin_components
                .iter()
                .filter(|k| target.supports(**k))
                .map(|k| k.as_str())
                .collect();

            let description = if supported.is_empty() {
                "No matching components".to_string()
            } else {
                supported.join(", ")
            };

            let has_matching = !supported.is_empty();

            SelectItem::new(target.display_name(), target.name().to_string())
                .with_description(description)
                .with_selected(has_matching)
                .with_enabled(has_matching)
        })
        .collect();

    let result = multi_select("Select target(s) to deploy", &mut items)
        .map_err(|e| PlmError::Tui(e.to_string()))?;

    if result.cancelled {
        return Err(PlmError::Cancelled);
    }

    if result.selected.is_empty() {
        return Err(PlmError::Deployment("No targets selected".to_string()));
    }

    Ok(result.selected)
}

#[cfg(test)]
#[path = "target_select_test.rs"]
mod tests;
