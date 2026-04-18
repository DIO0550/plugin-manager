//! Table 出力フォーマット

use crate::application::{PluginInfo, Source};
use crate::component::ComponentKind;
use crate::plugin::Author;
use comfy_table::{presets::UTF8_FULL, Table};
use std::fmt::Write;

pub(super) fn render_table(info: &PluginInfo) -> String {
    let mut out = render_basic_info(info);
    if let Some(author) = info.installed.author() {
        out.push_str(&render_author(author));
    }
    out.push_str(&render_installation(info));
    out.push_str(&render_components(info));
    out.push_str(&render_deployment(info));
    out
}

pub(super) fn print_table(info: &PluginInfo) {
    print!("{}", render_table(info));
}

pub(super) fn format_list(items: &[&str]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

fn render_basic_info(info: &PluginInfo) -> String {
    let mut out = String::new();
    writeln!(out, "Plugin Information").unwrap();
    writeln!(out, "==================").unwrap();
    writeln!(out).unwrap();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Field", "Value"])
        .add_row(vec!["Name", info.installed.name()])
        .add_row(vec!["Version", info.installed.version()])
        .add_row(vec![
            "Description",
            info.installed.description().unwrap_or("-"),
        ]);

    writeln!(out, "{table}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_author(author: &Author) -> String {
    let mut out = String::new();
    writeln!(out, "Author").unwrap();
    writeln!(out, "------").unwrap();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Field", "Value"])
        .add_row(vec!["Name", &author.name]);
    if let Some(email) = &author.email {
        table.add_row(vec!["Email", email]);
    }
    if let Some(url) = &author.url {
        table.add_row(vec!["URL", url]);
    }

    writeln!(out, "{table}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_installation(info: &PluginInfo) -> String {
    let mut out = String::new();
    writeln!(out, "Installation").unwrap();
    writeln!(out, "------------").unwrap();

    let source_str = match &info.source {
        Source::GitHub { repository } => format!("GitHub ({})", repository),
        Source::Marketplace { name } => format!("Marketplace ({})", name),
    };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Field", "Value"])
        .add_row(vec![
            "Installed At",
            info.installed_at.as_deref().unwrap_or("N/A"),
        ])
        .add_row(vec!["Source", &source_str]);

    writeln!(out, "{table}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_components(info: &PluginInfo) -> String {
    let mut out = String::new();
    writeln!(out, "Components").unwrap();
    writeln!(out, "----------").unwrap();

    let components = info.installed.components();
    let rows = [
        (ComponentKind::Skill, "Skills"),
        (ComponentKind::Agent, "Agents"),
        (ComponentKind::Command, "Commands"),
        (ComponentKind::Instruction, "Instructions"),
        (ComponentKind::Hook, "Hooks"),
    ]
    .into_iter()
    .map(|(kind, label)| {
        let names: Vec<&str> = components
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.as_str())
            .collect();
        vec![label.to_string(), format_list(&names)]
    });

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Type", "Items"])
        .add_rows(rows);

    writeln!(out, "{table}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_deployment(info: &PluginInfo) -> String {
    let mut out = String::new();
    writeln!(out, "Deployment").unwrap();
    writeln!(out, "----------").unwrap();

    let status = if info.installed.enabled() {
        "enabled"
    } else {
        "disabled"
    };
    let cache_path = info.installed.cache_path().to_string_lossy().into_owned();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Field", "Value"])
        .add_row(vec!["Status", status])
        .add_row(vec!["Cache Path", cache_path.as_str()]);

    writeln!(out, "{table}").unwrap();
    out
}
