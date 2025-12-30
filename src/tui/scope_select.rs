//! スコープ選択TUIダイアログ

use crate::error::{PlmError, Result};
use crate::target::Scope;
use crate::tui::dialog::{single_select, SelectItem};

/// スコープ選択ダイアログを表示
///
/// Personal（ユーザーレベル）またはProject（プロジェクトレベル）を選択する。
/// デフォルトはProjectが選択されている。
pub fn select_scope() -> Result<Scope> {
    let items = vec![
        SelectItem::new(Scope::Personal.display_name(), Scope::Personal)
            .with_description(Scope::Personal.description()),
        SelectItem::new(Scope::Project.display_name(), Scope::Project)
            .with_description(Scope::Project.description())
            .with_selected(true), // Default to Project
    ];

    let result =
        single_select("Select scope", &items).map_err(|e| PlmError::Tui(e.to_string()))?;

    if result.cancelled {
        return Err(PlmError::Cancelled);
    }

    result
        .selected
        .ok_or_else(|| PlmError::Deployment("No scope selected".to_string()))
}

#[cfg(test)]
mod tests {
    // TUIテストは実際のターミナルが必要なためスキップ
}
