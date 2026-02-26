//! plm info コマンド
//!
//! インストール済みプラグインの詳細情報を表示する。

use crate::application::{get_plugin_info, PluginDetail, PluginSource};
use crate::plugin::PluginCache;
use clap::{Parser, ValueEnum};
use comfy_table::{presets::UTF8_FULL, Table};

/// 出力形式
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Parser)]
pub struct Args {
    /// プラグイン名（marketplace/plugin 形式も可）
    pub name: String,

    /// 出力形式
    #[arg(long, short = 'f', value_enum, default_value = "table")]
    pub format: OutputFormat,
}

pub async fn run(args: Args) -> Result<(), String> {
    let cache = PluginCache::new().map_err(|e| e.to_string())?;
    let detail = get_plugin_info(&cache, &args.name).map_err(|e| e.to_string())?;

    match args.format {
        OutputFormat::Table => print_table(&detail),
        OutputFormat::Json => print_json(&detail)?,
        OutputFormat::Yaml => print_yaml(&detail)?,
    }

    Ok(())
}

fn print_table(detail: &PluginDetail) {
    // 基本情報
    println!("Plugin Information");
    println!("==================");
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Field", "Value"]);

    table.add_row(vec!["Name", &detail.name]);
    table.add_row(vec!["Version", &detail.version]);
    table.add_row(vec![
        "Description",
        detail.description.as_deref().unwrap_or("-"),
    ]);

    println!("{table}");
    println!();

    // 作者情報
    if let Some(author) = &detail.author {
        println!("Author");
        println!("------");

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

        println!("{author_table}");
        println!();
    }

    // インストール情報
    println!("Installation");
    println!("------------");

    let mut install_table = Table::new();
    install_table.load_preset(UTF8_FULL);
    install_table.set_header(vec!["Field", "Value"]);

    install_table.add_row(vec![
        "Installed At",
        detail.installed_at.as_deref().unwrap_or("N/A"),
    ]);

    let source_str = match &detail.source {
        PluginSource::GitHub { repository } => format!("GitHub ({})", repository),
        PluginSource::Marketplace { name } => format!("Marketplace ({})", name),
    };
    install_table.add_row(vec!["Source", &source_str]);

    println!("{install_table}");
    println!();

    // コンポーネント
    println!("Components");
    println!("----------");

    let mut comp_table = Table::new();
    comp_table.load_preset(UTF8_FULL);
    comp_table.set_header(vec!["Type", "Items"]);

    comp_table.add_row(vec!["Skills", &format_list(&detail.components.skills)]);
    comp_table.add_row(vec!["Agents", &format_list(&detail.components.agents)]);
    comp_table.add_row(vec!["Commands", &format_list(&detail.components.commands)]);
    comp_table.add_row(vec![
        "Instructions",
        &format_list(&detail.components.instructions),
    ]);
    comp_table.add_row(vec!["Hooks", &format_list(&detail.components.hooks)]);

    println!("{comp_table}");
    println!();

    // デプロイ情報
    println!("Deployment");
    println!("----------");

    let mut deploy_table = Table::new();
    deploy_table.load_preset(UTF8_FULL);
    deploy_table.set_header(vec!["Field", "Value"]);

    let status = if detail.enabled {
        "enabled"
    } else {
        "disabled"
    };
    deploy_table.add_row(vec!["Status", status]);
    deploy_table.add_row(vec!["Cache Path", &detail.cache_path]);

    println!("{deploy_table}");
}

fn format_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

fn print_json(detail: &PluginDetail) -> Result<(), String> {
    serde_json::to_string_pretty(detail)
        .map(|json| println!("{json}"))
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

fn print_yaml(detail: &PluginDetail) -> Result<(), String> {
    serde_yaml::to_string(detail)
        .map(|yaml| print!("{yaml}"))
        .map_err(|e| format!("Failed to serialize to YAML: {}", e))
}

#[cfg(test)]
#[path = "info_test.rs"]
mod tests;
