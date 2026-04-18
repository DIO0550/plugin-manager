//! Table 出力フォーマット

use crate::application::{PluginInfo, Source};
use crate::component::ComponentKind;
use comfy_table::{presets::UTF8_FULL, Table};
use std::fmt::Write;

pub(super) fn render_table(info: &PluginInfo) -> String {
    let mut out = String::new();

    // 基本情報
    writeln!(out, "Plugin Information").unwrap();
    writeln!(out, "==================").unwrap();
    writeln!(out).unwrap();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Field", "Value"]);

    table.add_row(vec!["Name", info.installed.name()]);
    table.add_row(vec!["Version", info.installed.version()]);
    table.add_row(vec![
        "Description",
        info.installed.description().unwrap_or("-"),
    ]);

    writeln!(out, "{table}").unwrap();
    writeln!(out).unwrap();

    // 作者情報
    if let Some(author) = info.installed.author() {
        writeln!(out, "Author").unwrap();
        writeln!(out, "------").unwrap();

        let mut author_table = Table::new();
        author_table.load_preset(UTF8_FULL);
        author_table.set_header(vec!["Field", "Value"]);

        author_table.add_row(vec!["Name", &author.name]);
        if let Some(email) = &author.email {
            author_table.add_row(vec!["Email", email]);
        }
        if let Some(url) = &author.url {
            author_table.add_row(vec!["URL", url]);
        }

        writeln!(out, "{author_table}").unwrap();
        writeln!(out).unwrap();
    }

    // インストール情報
    writeln!(out, "Installation").unwrap();
    writeln!(out, "------------").unwrap();

    let mut install_table = Table::new();
    install_table.load_preset(UTF8_FULL);
    install_table.set_header(vec!["Field", "Value"]);

    install_table.add_row(vec![
        "Installed At",
        info.installed_at.as_deref().unwrap_or("N/A"),
    ]);

    let source_str = match &info.source {
        Source::GitHub { repository } => format!("GitHub ({})", repository),
        Source::Marketplace { name } => format!("Marketplace ({})", name),
    };
    install_table.add_row(vec!["Source", &source_str]);

    writeln!(out, "{install_table}").unwrap();
    writeln!(out).unwrap();

    // コンポーネント
    writeln!(out, "Components").unwrap();
    writeln!(out, "----------").unwrap();

    let mut comp_table = Table::new();
    comp_table.load_preset(UTF8_FULL);
    comp_table.set_header(vec!["Type", "Items"]);

    let components = info.installed.components();
    for (kind, label) in [
        (ComponentKind::Skill, "Skills"),
        (ComponentKind::Agent, "Agents"),
        (ComponentKind::Command, "Commands"),
        (ComponentKind::Instruction, "Instructions"),
        (ComponentKind::Hook, "Hooks"),
    ] {
        let names: Vec<&str> = components
            .iter()
            .filter(|c| c.kind == kind)
            .map(|c| c.name.as_str())
            .collect();
        comp_table.add_row(vec![label, &format_list(&names)]);
    }

    writeln!(out, "{comp_table}").unwrap();
    writeln!(out).unwrap();

    // デプロイ情報
    writeln!(out, "Deployment").unwrap();
    writeln!(out, "----------").unwrap();

    let mut deploy_table = Table::new();
    deploy_table.load_preset(UTF8_FULL);
    deploy_table.set_header(vec!["Field", "Value"]);

    let status = if info.installed.enabled() {
        "enabled"
    } else {
        "disabled"
    };
    deploy_table.add_row(vec!["Status", status]);
    deploy_table.add_row(vec![
        "Cache Path",
        &info.installed.cache_path().to_string_lossy(),
    ]);

    writeln!(out, "{deploy_table}").unwrap();

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
