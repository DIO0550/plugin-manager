//! JSON 出力フォーマット

use crate::application::{PluginInfo, Source};
use crate::component::{Component, ComponentKind};
use crate::plugin::Author;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::path::Path;

#[derive(Serialize)]
struct Wire<'a> {
    name: &'a str,
    version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<WireAuthor<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installed_at: Option<&'a str>,
    source: WireSource<'a>,
    components: WireComponents<'a>,
    enabled: bool,
    cache_path: &'a Path,
}

#[derive(Serialize)]
struct WireAuthor<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
}

impl<'a> From<&'a Author> for WireAuthor<'a> {
    fn from(a: &'a Author) -> Self {
        Self {
            name: &a.name,
            email: a.email.as_deref(),
            url: a.url.as_deref(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum WireSource<'a> {
    Github { repository: &'a str },
    Marketplace { name: &'a str },
}

impl<'a> From<&'a Source> for WireSource<'a> {
    fn from(s: &'a Source) -> Self {
        match s {
            Source::GitHub { repository } => WireSource::Github { repository },
            Source::Marketplace { name } => WireSource::Marketplace { name },
        }
    }
}

struct WireComponents<'a>(&'a [Component]);

impl Serialize for WireComponents<'_> {
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

impl<'a> From<&'a PluginInfo> for Wire<'a> {
    fn from(info: &'a PluginInfo) -> Self {
        Self {
            name: info.installed.name(),
            version: info.installed.version(),
            description: info.installed.description(),
            author: info.installed.author().map(WireAuthor::from),
            installed_at: info.installed_at.as_deref(),
            source: WireSource::from(&info.source),
            components: WireComponents(info.installed.components()),
            enabled: info.installed.enabled(),
            cache_path: info.installed.cache_path(),
        }
    }
}

pub(super) fn render_json(info: &PluginInfo) -> Result<String, String> {
    let wire = Wire::from(info);
    serde_json::to_string_pretty(&wire).map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

pub(super) fn print_json(info: &PluginInfo) -> Result<(), String> {
    let s = render_json(info)?;
    println!("{s}");
    Ok(())
}
