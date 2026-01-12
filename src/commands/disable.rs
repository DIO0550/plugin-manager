//! plm disable コマンド
//!
//! プラグインを無効化する。ターゲット環境からコンポーネントを削除し（キャッシュは残す）、
//! `.plm-meta.json` の `statusByTarget` を更新する。

use crate::application::{disable_plugin, OperationResult};
use crate::plugin::{meta, PluginCache};
use clap::{Parser, ValueEnum};
use std::env;
use std::process;

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

impl TargetKind {
    fn as_str(&self) -> &'static str {
        match self {
            TargetKind::Codex => "codex",
            TargetKind::Copilot => "copilot",
        }
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Plugin name (e.g., "owner--repo" or "plugin-name")
    pub name: String,

    /// Target environment to disable (codex or copilot)
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// Marketplace name (default: github)
    #[arg(long, short = 'm', default_value = "github")]
    pub marketplace: String,
}

pub async fn run(args: Args) -> Result<(), String> {
    let cache = PluginCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;

    // プラグインがキャッシュに存在するか確認
    // キャッシュが必要な理由: マニフェストから削除対象コンポーネントを特定するため
    if !cache.is_cached(Some(&args.marketplace), &args.name) {
        eprintln!(
            "Error: Plugin '{}' not found in cache (marketplace: {})",
            args.name, args.marketplace
        );
        eprintln!("Hint: Cache is required to identify components to remove.");
        process::exit(1);
    }

    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    let target_filter = args.target.as_ref().map(|t| t.as_str());

    // disable_plugin を実行
    let result = disable_plugin(&args.name, Some(&args.marketplace), &project_root, target_filter);

    // 結果に基づいてステータスを更新
    let plugin_path = cache.plugin_path(Some(&args.marketplace), &args.name);
    update_status_after_disable(&plugin_path, &result);

    // 結果表示
    display_result(&args.name, &result, target_filter);

    // 終了コード
    if result.success {
        Ok(())
    } else {
        process::exit(1);
    }
}

/// disable 後のステータス更新
fn update_status_after_disable(plugin_path: &std::path::Path, result: &OperationResult) {
    // 既存のメタデータを読み込む（なければデフォルト）
    let mut plugin_meta = meta::load_meta(plugin_path).unwrap_or_default();

    // 成功したターゲットのステータスを更新（部分成功の場合も）
    let target_names = result.affected_targets.target_names();
    for target_name in target_names {
        plugin_meta.set_status(target_name, "disabled");
    }

    // フィルタ指定時、対象ターゲットがサポートされていない場合はスキップ扱い
    // （affected_targets にも含まれないので何もしない）

    // メタデータを書き込み
    if let Err(e) = meta::write_meta(plugin_path, &plugin_meta) {
        eprintln!("Warning: Failed to update .plm-meta.json: {}", e);
    }
}

/// 結果を表示
fn display_result(plugin_name: &str, result: &OperationResult, target_filter: Option<&str>) {
    if result.success {
        let targets = result.affected_targets.target_names();
        if targets.is_empty() {
            // ターゲット指定で未対応の場合
            if let Some(filter) = target_filter {
                println!(
                    "Skipped: Plugin '{}' has no components for target '{}'",
                    plugin_name, filter
                );
            } else {
                println!("Disabled: Plugin '{}' (no components removed)", plugin_name);
            }
        } else {
            let target_list = targets.join(", ");
            let component_count = result.affected_targets.total_components();
            println!(
                "Disabled: Plugin '{}' ({} component(s) removed from {})",
                plugin_name, component_count, target_list
            );
        }
    } else {
        if let Some(error) = &result.error {
            eprintln!("Error: Failed to disable plugin '{}': {}", plugin_name, error);
        } else {
            eprintln!("Error: Failed to disable plugin '{}'", plugin_name);
        }

        // 部分成功の場合
        let successful_targets = result.affected_targets.target_names();
        if !successful_targets.is_empty() {
            println!(
                "  Partially disabled: {} target(s) succeeded",
                successful_targets.len()
            );
        }
    }
}
