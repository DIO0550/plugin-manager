//! enable / disable ハンドラ共通ロジック
//!
//! `plm enable` と `plm disable` は操作方向（有効化 vs 無効化）以外ほぼ同一。
//! 本モジュールで共通化し、各ハンドラは薄いラッパーとして動作する。

use crate::application::OperationOutcome;
use crate::commands::args::MarketplaceArgs;
use crate::plugin::{PackageCache, PackageCacheAccess};
use crate::target::TargetKind;
use std::env;

/// toggle 操作の方向を表す
pub enum ToggleOp {
    Enable,
    Disable,
}

impl ToggleOp {
    /// 操作動詞（過去形）: 表示に使用
    pub fn label_past(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "Enabled",
            ToggleOp::Disable => "Disabled",
        }
    }

    /// 操作動詞（現在形）: エラーメッセージに使用
    pub fn label_verb(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "enable",
            ToggleOp::Disable => "disable",
        }
    }

    /// コンポーネント操作の説明
    pub fn component_action(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "deployed to",
            ToggleOp::Disable => "removed from",
        }
    }

    /// コンポーネントが空の場合の説明
    pub fn no_components_label(&self) -> &'static str {
        match self {
            ToggleOp::Enable => "no components deployed",
            ToggleOp::Disable => "no components removed",
        }
    }
}

/// CLI引数の共通部分
pub struct ToggleArgs {
    pub plugin_name: String,
    pub target: Option<TargetKind>,
    pub marketplace: MarketplaceArgs,
}

/// enable / disable の共通実行ロジック
///
/// # Arguments
///
/// * `op` - 実行する操作（Enable / Disable）
/// * `args` - 共通引数
/// * `apply` - 実際の操作を実行するクロージャ（`enable_plugin` または `disable_plugin`）
pub async fn run_toggle<F>(op: ToggleOp, args: ToggleArgs, apply: F) -> Result<(), String>
where
    F: FnOnce(
        &dyn PackageCacheAccess,
        &str,
        Option<&str>,
        &std::path::Path,
        Option<&str>,
    ) -> OperationOutcome,
{
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {}", e))?;
    let marketplace = args.marketplace.marketplace_or_default();

    if !cache.is_cached(Some(marketplace), &args.plugin_name) {
        return Err(format!(
            "Error: Plugin '{}' not found in cache (marketplace: {})",
            args.plugin_name, marketplace
        ));
    }

    let project_root = env::current_dir().unwrap_or_else(|_| ".".into());
    let target_filter = args.target.as_ref().map(|t| t.as_str());

    let result = apply(
        &cache,
        &args.plugin_name,
        Some(marketplace),
        &project_root,
        target_filter,
    );

    if result.success {
        display_result(&op, &args.plugin_name, &result, target_filter);
        Ok(())
    } else {
        let successful_targets = result.affected_targets.target_names();
        if !successful_targets.is_empty() {
            println!(
                "  Partially {}d: {} target(s) succeeded",
                op.label_verb(),
                successful_targets.len()
            );
        }
        if let Some(error) = &result.error {
            Err(format!(
                "Error: Failed to {} plugin '{}': {}",
                op.label_verb(),
                args.plugin_name,
                error
            ))
        } else {
            Err(format!(
                "Error: Failed to {} plugin '{}'",
                op.label_verb(),
                args.plugin_name
            ))
        }
    }
}

/// 成功時の結果を表示
///
/// # Arguments
///
/// * `op` - 実行した操作（表示語の選択に使用）
/// * `plugin_name` - プラグイン識別子
/// * `result` - 操作結果
/// * `target_filter` - 指定されたターゲットフィルタ
fn display_result(
    op: &ToggleOp,
    plugin_name: &str,
    result: &OperationOutcome,
    target_filter: Option<&str>,
) {
    let targets = result.affected_targets.target_names();
    if targets.is_empty() {
        if let Some(filter) = target_filter {
            println!(
                "Skipped: Plugin '{}' has no components for target '{}'",
                plugin_name, filter
            );
        } else {
            println!(
                "{}: Plugin '{}' ({})",
                op.label_past(),
                plugin_name,
                op.no_components_label()
            );
        }
    } else {
        let target_list = targets.join(", ");
        let component_count = result.affected_targets.total_components();
        println!(
            "{}: Plugin '{}' ({} component(s) {} {})",
            op.label_past(),
            plugin_name,
            component_count,
            op.component_action(),
            target_list
        );
    }
}
