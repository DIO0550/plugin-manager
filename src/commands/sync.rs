//! plm sync コマンド

use crate::component::Scope;
use crate::sync::{
    sync, PlacedComponent, SyncDestination, SyncOptions, SyncResult, SyncSource, SyncableKind,
};
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

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm sync`.
pub async fn run(args: Args) -> Result<(), String> {
    if args.from == args.to {
        return Err("Cannot sync to the same target".to_string());
    }

    let project_root = env::current_dir().map_err(|e| e.to_string())?;

    let source = SyncSource::new(args.from, &project_root).map_err(|e| e.to_string())?;
    let dest = SyncDestination::new(args.to, &project_root).map_err(|e| e.to_string())?;

    let options = SyncOptions {
        component_type: args.component_type,
        scope: args.scope,
        dry_run: args.dry_run,
    };

    let result = sync(&source, &dest, &options).map_err(|e| e.to_string())?;

    print_result(&result, source.name(), dest.name());

    if result.failure_count() > 0 {
        return Err(format!("{} item(s) failed to sync", result.failure_count()));
    }

    Ok(())
}

/// # Arguments
///
/// * `result` - Outcome returned by the `sync` engine.
/// * `from_name` - Display name of the source target.
/// * `to_name` - Display name of the destination target.
fn print_result(result: &SyncResult, from_name: &str, to_name: &str) {
    println!("Sync: {} -> {}\n", from_name.cyan(), to_name.cyan());

    let total = result.total_count();
    if total == 0 {
        println!("No components to sync.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Type", "Name", "Scope", "Action"]);

    for component in &result.created {
        add_component_row(&mut table, component, "Create", Color::Green);
    }

    for component in &result.updated {
        add_component_row(&mut table, component, "Update", Color::Yellow);
    }

    for component in &result.deleted {
        add_component_row(&mut table, component, "Delete", Color::Red);
    }

    for component in &result.skipped {
        add_component_row(&mut table, component, "Skip (no change)", Color::DarkGrey);
    }

    for component in &result.unsupported {
        add_component_row(&mut table, component, "Skip (unsupported)", Color::DarkGrey);
    }

    println!("{table}");

    let prefix = if result.dry_run {
        "Would sync"
    } else {
        "Synced"
    };
    println!(
        "\n{}: {} created, {} updated, {} deleted, {} skipped",
        prefix.bold(),
        result.create_count().to_string().green(),
        result.update_count().to_string().yellow(),
        result.delete_count().to_string().red(),
        result.skip_count().to_string().dimmed()
    );

    if !result.failed.is_empty() {
        println!("\n{}", "Failed items:".red().bold());
        for failure in &result.failed {
            println!(
                "  {} ({}/{}): {} - {}",
                failure.component.name().red(),
                failure.component.kind().display_name(),
                failure.component.scope().display_name(),
                failure.action.display_name(),
                failure.error
            );
        }
    }
}

/// # Arguments
///
/// * `table` - Table to append a row to.
/// * `component` - Placed component that describes the row.
/// * `action` - Human-readable action label (Create, Update, ...).
/// * `color` - Foreground color used for the action cell.
fn add_component_row(table: &mut Table, component: &PlacedComponent, action: &str, color: Color) {
    table.add_row(vec![
        Cell::new(component.kind().display_name()),
        Cell::new(component.name()),
        Cell::new(component.scope().display_name()),
        Cell::new(action).fg(color),
    ]);
}

#[cfg(test)]
#[path = "sync_test.rs"]
mod tests;
