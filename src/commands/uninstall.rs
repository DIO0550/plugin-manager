use crate::application::{self, UninstallInfo};
use clap::Parser;
use owo_colors::OwoColorize;
use std::env;
use std::io::{self, Write};

#[derive(Debug, Parser)]
pub struct Args {
    /// プラグイン名（キャッシュディレクトリ名、例: "DIO0550--cc-plugin"）
    pub name: String,

    /// マーケットプレイス（未指定なら "github"）
    #[arg(long, short = 'm')]
    pub marketplace: Option<String>,

    /// 確認プロンプトをスキップ
    #[arg(long, short = 'f')]
    pub force: bool,
}

pub async fn run(args: Args) -> Result<(), String> {
    let project_root =
        env::current_dir().map_err(|e| format!("Failed to get current dir: {}", e))?;

    // 1. 事前チェック: プラグイン存在確認 & 情報取得
    let info = application::get_uninstall_info(&args.name, args.marketplace.as_deref())?;

    // 2. 削除対象の情報表示
    display_uninstall_info(&info);

    // 3. 確認プロンプト（--force でスキップ）
    if !args.force && !confirm_uninstall(&args.name)? {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    // 4. 削除実行
    let result =
        application::uninstall_plugin(&args.name, args.marketplace.as_deref(), &project_root);

    // 5. 結果表示
    if result.success {
        println!(
            "{} Plugin '{}' uninstalled successfully.",
            "✓".green(),
            args.name
        );
        let target_names = result.affected_targets.target_names();
        if !target_names.is_empty() {
            println!(
                "  Removed {} component(s) from: {}",
                result.affected_targets.total_components(),
                target_names.join(", ")
            );
        }
        Ok(())
    } else {
        Err(format!(
            "Failed to uninstall: {}",
            result.error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }
}

/// 削除対象の情報を表示
fn display_uninstall_info(info: &UninstallInfo) {
    println!(
        "{} Plugin: {} (marketplace: {})",
        "i".blue(),
        info.plugin_name.bold(),
        info.marketplace
    );

    if info.components.is_empty() {
        println!("  No components found.");
    } else {
        println!("  Components ({}):", info.components.len());
        for component in &info.components {
            println!("    - {} ({})", component.name, component.kind);
        }
    }

    if !info.affected_targets.is_empty() {
        println!("  Affected targets: {}", info.affected_targets.join(", "));
    }

    println!();
}

/// ユーザーに削除確認を求める
fn confirm_uninstall(plugin_name: &str) -> Result<bool, String> {
    print!(
        "Are you sure you want to uninstall '{}'? [y/N]: ",
        plugin_name
    );
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

#[cfg(test)]
#[path = "uninstall_test.rs"]
mod tests;
