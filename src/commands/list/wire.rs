//! JSON 出力用の Wire フォーマット定義
//!
//! `InstalledPlugin` を serde serialize 用の表現に変換する。
//! 素の `list --json` と `list --outdated --json` で構造を共有し、
//! キー名の意図しないドリフトを防ぐ。

use crate::component::{Component, ComponentKind};
use crate::plugin::{InstalledPlugin, UpgradeState};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub(super) struct Wire<'a> {
    pub(super) name: &'a str,
    pub(super) version: &'a str,
    pub(super) install_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) marketplace: Option<&'a str>,
    pub(super) enabled: bool,
    pub(super) components: ComponentsWire<'a>,
}

impl<'a> Wire<'a> {
    /// Builds a `Wire` borrowing fields from an installed plugin.
    ///
    /// # Arguments
    ///
    /// * `plugin` - Installed plugin to borrow from.
    pub(super) fn from_installed(plugin: &'a InstalledPlugin) -> Self {
        Self {
            name: plugin.name(),
            version: plugin.version(),
            install_id: plugin.install_id(),
            marketplace: plugin.marketplace(),
            enabled: plugin.enabled(),
            components: ComponentsWire(plugin.components()),
        }
    }
}

pub(super) struct ComponentsWire<'a>(pub(super) &'a [Component]);

impl Serialize for ComponentsWire<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(5))?;
        for (kind, key) in [
            (ComponentKind::Skill, "skills"),
            (ComponentKind::Agent, "agents"),
            (ComponentKind::Command, "commands"),
            (ComponentKind::Instruction, "instructions"),
            (ComponentKind::Hook, "hooks"),
        ] {
            let names: Vec<&str> = self
                .0
                .iter()
                .filter(|c| c.kind == kind)
                .map(|c| c.name.as_str())
                .collect();
            map.serialize_entry(key, &names)?;
        }
        map.end()
    }
}

#[derive(Serialize)]
pub(super) struct OutdatedWire<'a> {
    pub(super) plugin: Wire<'a>,
    pub(super) check: &'a UpgradeState,
}

impl<'a> OutdatedWire<'a> {
    /// Builds an `OutdatedWire` from a plugin and its upgrade state.
    ///
    /// # Arguments
    ///
    /// * `plugin` - Installed plugin to embed.
    /// * `check` - Upgrade state paired with the plugin.
    pub(super) fn from_entry(plugin: &'a InstalledPlugin, check: &'a UpgradeState) -> Self {
        Self {
            plugin: Wire::from_installed(plugin),
            check,
        }
    }
}

#[cfg(test)]
#[path = "wire_test.rs"]
mod tests;
