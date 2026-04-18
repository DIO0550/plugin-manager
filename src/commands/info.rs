//! plm info コマンド
//!
//! インストール済みプラグインの詳細情報を表示する。

mod json;
mod table;
mod wire;
mod yaml;

use crate::application::get_plugin_info;
use crate::plugin::PackageCache;
use clap::{Parser, ValueEnum};

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
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    let detail = get_plugin_info(&cache, &args.name)
        .map_err(|e| format!("Failed to get plugin info: {e}"))?;

    match args.format {
        OutputFormat::Table => table::print_table(&detail),
        OutputFormat::Json => json::print_json(&detail)?,
        OutputFormat::Yaml => yaml::print_yaml(&detail)?,
    }

    Ok(())
}

#[cfg(test)]
#[path = "info/info_test.rs"]
mod tests;
