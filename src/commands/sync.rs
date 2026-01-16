//! plm sync コマンド

use crate::component::Scope;
use crate::sync::{SyncAction, SyncExecutor, SyncOptions, SyncPlan, SyncResult, SyncableKind};
use crate::target::TargetKind;
use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use owo_colors::OwoColorize;
use std::env;

#[derive(Debug, Parser)]
pub struct Args {
    /// Source target
    #[arg(long, value_enum)]
    pub from: TargetKind,

    /// Destination target
    #[arg(long, value_enum)]
    pub to: TargetKind,

    /// Component type to sync (all if not specified)
    #[arg(long = "type", value_enum)]
    pub component_type: Option<SyncableKind>,

    /// Scope to sync (both if not specified)
    #[arg(long, value_enum)]
    pub scope: Option<Scope>,

    /// Preview only, do not actually sync
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: Args) -> Result<(), String> {
    let project_root = env::current_dir().map_err(|e| e.to_string())?;

    // SyncExecutor を作成
    let executor = SyncExecutor::new(args.from.clone(), args.to.clone(), &project_root)
        .map_err(|e| e.to_string())?;

    // オプションを構築
    let options = SyncOptions {
        component_type: args.component_type,
        scope: args.scope,
    };

    // 同期計画を作成
    let plan = executor.plan(&options).map_err(|e| e.to_string())?;

    // 計画を表示
    print_plan(&plan);

    if plan.items.is_empty() {
        println!("\nNo components to sync.");
        return Ok(());
    }

    // dry-run の場合は終了
    if args.dry_run {
        println!(
            "\n{} {} item(s) would be synced ({} create, {} overwrite, {} skip)",
            "Dry run:".yellow().bold(),
            plan.actionable_count(),
            plan.create_count(),
            plan.overwrite_count(),
            plan.skip_count()
        );
        return Ok(());
    }

    // アクション可能なアイテムがない場合
    if plan.actionable_count() == 0 {
        println!("\nNo actionable items (all skipped).");
        return Ok(());
    }

    // 同期を実行
    println!("\nSyncing...");
    let result = executor.execute(&plan);

    // 結果を表示
    print_result(&result, &plan);

    // 失敗があれば非0終了
    if !result.failed.is_empty() {
        return Err(format!(
            "{} item(s) failed to sync",
            result.failed.len()
        ));
    }

    Ok(())
}

fn print_plan(plan: &SyncPlan) {
    println!(
        "Sync plan: {} -> {}\n",
        plan.from_target.cyan(),
        plan.to_target.cyan()
    );

    if plan.items.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Type", "Name", "Scope", "Action", "Reason"]);

    for item in &plan.items {
        let action_cell = match &item.action {
            SyncAction::Create => Cell::new("Create").fg(Color::Green),
            SyncAction::Overwrite => Cell::new("Overwrite").fg(Color::Yellow),
            SyncAction::Skip { .. } => Cell::new("Skip").fg(Color::DarkGrey),
        };

        let reason = item.action.skip_reason().unwrap_or("-");

        table.add_row(vec![
            Cell::new(item.kind.display_name()),
            Cell::new(&item.name),
            Cell::new(item.scope.display_name()),
            action_cell,
            Cell::new(reason),
        ]);
    }

    println!("{table}");
}

fn print_result(result: &SyncResult, plan: &SyncPlan) {
    let succeeded = result.succeeded.len();
    let failed = result.failed.len();
    let skipped = plan.skip_count();

    println!(
        "\n{}: {} succeeded, {} failed, {} skipped",
        "Result".bold(),
        succeeded.to_string().green(),
        failed.to_string().red(),
        skipped.to_string().dimmed()
    );

    if !result.failed.is_empty() {
        println!("\n{}", "Failed items:".red().bold());
        for failure in &result.failed {
            println!(
                "  {} ({}/{}): {}",
                failure.item.name.red(),
                failure.item.kind.display_name(),
                failure.item.scope.display_name(),
                failure.error
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentKind;
    use crate::sync::SyncItem;
    use std::path::PathBuf;

    #[test]
    fn test_args_parsing() {
        use clap::CommandFactory;
        let cmd = Args::command();
        cmd.debug_assert();
    }

    #[test]
    fn test_print_plan_empty() {
        let plan = SyncPlan {
            from_target: "codex".to_string(),
            to_target: "copilot".to_string(),
            items: vec![],
        };
        // Should not panic
        print_plan(&plan);
    }

    #[test]
    fn test_print_result_success() {
        let plan = SyncPlan {
            from_target: "codex".to_string(),
            to_target: "copilot".to_string(),
            items: vec![SyncItem {
                kind: ComponentKind::Skill,
                name: "test".to_string(),
                scope: Scope::Personal,
                source_path: PathBuf::from("/src"),
                target_path: PathBuf::from("/dst"),
                action: SyncAction::Create,
            }],
        };

        let result = SyncResult {
            succeeded: vec![plan.items[0].clone()],
            failed: vec![],
        };

        // Should not panic
        print_result(&result, &plan);
    }
}
