//! `InstalledPlugin` / `PluginDetail` 共通の components シリアライズヘルパ。
//!
//! `Vec<Component>` を kind 別にグループ化し
//! `{"skills": [...], "agents": [...], ...}` の形で JSON 出力する。

use crate::component::{Component, ComponentKind};

/// `Vec<Component>` を kind 別にグループ化して JSON serialize する純粋関数。
pub fn serialize_components<S>(components: &[Component], serializer: S) -> Result<S::Ok, S::Error>
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

#[cfg(test)]
#[path = "plugin_component_serde_test.rs"]
mod tests;
