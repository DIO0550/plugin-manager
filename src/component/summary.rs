//! コンポーネントのサマリ情報
//!
//! TUI等での表示に使用する軽量な型を提供する。

use crate::component::{Component, ComponentKind};

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

/// Vec<Component> を kind 別にグループ化して JSON serialize する純粋関数。
///
/// `{"skills": ["name1", ...], "agents": [...], ...}` の形を出力する。
/// `PluginSummary` では `#[serde(flatten)]` と併用して top-level に展開、
/// `PluginDetail` では nested `components` キーとして出力する。
#[allow(clippy::ptr_arg)]
pub fn serialize_components<S>(
    components: &Vec<Component>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(5))?;
    for (kind, key) in [
        (ComponentKind::Skill, "skills"),
        (ComponentKind::Agent, "agents"),
        (ComponentKind::Command, "commands"),
        (ComponentKind::Instruction, "instructions"),
        (ComponentKind::Hook, "hooks"),
    ] {
        let names: Vec<&str> = components
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.as_str())
            .collect();
        map.serialize_entry(key, &names)?;
    }
    map.end()
}

/// コンポーネント名（表示用）
#[derive(Debug, Clone)]
pub struct ComponentName {
    pub name: String,
}

#[cfg(test)]
#[path = "summary_test.rs"]
mod tests;
