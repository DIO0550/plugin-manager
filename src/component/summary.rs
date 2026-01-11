//! コンポーネントのサマリ情報
//!
//! TUI等での表示に使用する軽量な型を提供する。

use crate::component::ComponentKind;

/// コンポーネント種別ごとの件数
#[derive(Debug, Clone)]
pub struct ComponentTypeCount {
    /// コンポーネント種別
    pub kind: ComponentKind,
    /// 件数
    pub count: usize,
}

impl ComponentTypeCount {
    /// 表示用タイトルを取得（複数形）
    pub fn title(&self) -> &'static str {
        match self.kind {
            ComponentKind::Skill => "Skills",
            ComponentKind::Agent => "Agents",
            ComponentKind::Command => "Commands",
            ComponentKind::Instruction => "Instructions",
            ComponentKind::Hook => "Hooks",
        }
    }
}

/// コンポーネント名（表示用）
#[derive(Debug, Clone)]
pub struct ComponentName {
    pub name: String,
}

#[cfg(test)]
#[path = "summary_test.rs"]
mod tests;
