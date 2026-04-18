//! JSON / YAML 共通の Wire フォーマット定義
//!
//! `PluginInfo` を serde serialize 用の表現に変換する。
//! JSON と YAML で同一構造を共有し、format 間の意図しないドリフトを防ぐ。

use crate::application::{PluginInfo, Source};
use crate::component::{Component, ComponentKind};
use crate::plugin::Author;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub(super) struct Wire<'a> {
    pub(super) name: &'a str,
    pub(super) version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) author: Option<WireAuthor<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) installed_at: Option<&'a str>,
    pub(super) source: WireSource<'a>,
    pub(super) components: WireComponents<'a>,
    pub(super) enabled: bool,
    pub(super) cache_path: String,
}

#[derive(Serialize)]
pub(super) struct WireAuthor<'a> {
    pub(super) name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) url: Option<&'a str>,
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
pub(super) enum WireSource<'a> {
    GitHub { repository: &'a str },
    Marketplace { name: &'a str },
}

impl<'a> From<&'a Source> for WireSource<'a> {
    fn from(s: &'a Source) -> Self {
        match s {
            Source::GitHub { repository } => WireSource::GitHub { repository },
            Source::Marketplace { name } => WireSource::Marketplace { name },
        }
    }
}

pub(super) struct WireComponents<'a>(pub(super) &'a [Component]);

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
            cache_path: info.installed.cache_path().to_string_lossy().into_owned(),
        }
    }
}
