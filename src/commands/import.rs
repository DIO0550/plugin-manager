//! plm import コマンド
//!
//! Claude Code Plugin形式のGitHubリポジトリから、
//! 特定のコンポーネントを選択してインポートする。

use crate::component::{AgentFormat, Component, ComponentDeployment, ComponentKind};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::import::{ImportRecord, ImportRegistry};
use crate::output::CommandSummary;
use crate::source::parse_source;
use crate::target::{all_targets, parse_target, PluginOrigin, Scope, Target, TargetKind};
use crate::tui;
use chrono::Utc;
use clap::Parser;
use std::env;
use std::path::Path;

#[derive(Debug, Parser)]
pub struct Args {
    /// GitHub repository in owner/repo format
    pub source: String,

    /// Specific component paths to import (e.g., skills/pdf, agents/review)
    /// Format: <kind>/<name> where kind is: skills, agents, commands, instructions, hooks
    #[arg(long = "component", value_name = "PATH")]
    pub component: Vec<String>,

    /// Filter by component type (cannot be used with --component)
    #[arg(long = "type", value_enum, conflicts_with = "component")]
    pub component_type: Vec<ComponentKind>,

    /// Target environments to deploy to (if not specified, TUI selection)
    #[arg(long, value_enum)]
    pub target: Option<Vec<TargetKind>>,

    /// Deployment scope (if not specified, TUI selection)
    #[arg(long, value_enum)]
    pub scope: Option<Scope>,

    /// Force re-download even if cached
    #[arg(long)]
    pub force: bool,
}

/// Parse a component path string into (ComponentKind, name)
///
/// Format: `<plural_kind>/<name>`
/// - plural_kind: skills, agents, commands, instructions, hooks (case-insensitive)
/// - name: component name (case-sensitive)
///
/// # Examples
/// - "skills/pdf" -> Ok((Skill, "pdf"))
/// - "SKILLS/pdf" -> Ok((Skill, "pdf")) (kind normalized)
/// - "skills/PDF" -> Ok((Skill, "PDF")) (name preserved)
/// - "skill/pdf" -> Err (singular not allowed)
pub fn parse_component_path(path: &str) -> Result<(ComponentKind, String), String> {
    // 1. Trim whitespace
    let path = path.trim();

    // 2. Remove trailing slash
    let path = path.trim_end_matches('/');

    // 3. Check for consecutive slashes
    if path.contains("//") {
        return Err(format!(
            "Invalid component path format: '{}'. Consecutive slashes are not allowed.",
            path
        ));
    }

    // 4. Split by first slash
    let (kind_str, name) = path.split_once('/').ok_or_else(|| {
        format!(
            "Invalid component path format: '{}'. Expected: <kind>/<name>",
            path
        )
    })?;

    // 5. Check for nested paths (more than one slash)
    if name.contains('/') {
        return Err(format!(
            "Invalid component path format: '{}'. Nested paths are not allowed.",
            path
        ));
    }

    // 6. Check empty name
    if name.is_empty() {
        return Err(format!(
            "Invalid component path format: '{}'. Component name cannot be empty.",
            path
        ));
    }

    // 7. Parse kind (case-insensitive, must be plural form)
    let kind = match kind_str.to_lowercase().as_str() {
        "skills" => ComponentKind::Skill,
        "agents" => ComponentKind::Agent,
        "commands" => ComponentKind::Command,
        "instructions" => ComponentKind::Instruction,
        "hooks" => ComponentKind::Hook,
        _ => {
            return Err(format!(
                "Invalid component kind: '{}'. Valid kinds: skills, agents, commands, instructions, hooks",
                kind_str
            ))
        }
    };

    Ok((kind, name.to_string()))
}

/// Filter components by paths or types
///
/// Returns (filtered_components, skipped_paths)
/// - If paths is non-empty: filter by kind + name exact match
/// - If types is non-empty: filter by kind only
/// - If both empty: return all components
pub fn filter_components(
    components: Vec<Component>,
    paths: &[(ComponentKind, String)],
    types: &[ComponentKind],
) -> (Vec<Component>, Vec<String>) {
    use std::collections::HashSet;

    // If both empty, return all components
    if paths.is_empty() && types.is_empty() {
        return (components, vec![]);
    }

    // Filter by types (kind only)
    if !types.is_empty() {
        let filtered = components
            .into_iter()
            .filter(|c| types.contains(&c.kind))
            .collect();
        return (filtered, vec![]);
    }

    // Filter by paths (kind + name exact match)
    // Deduplicate paths while preserving order
    let mut seen_paths: HashSet<(ComponentKind, &str)> = HashSet::new();
    let mut unique_paths: Vec<&(ComponentKind, String)> = Vec::new();

    for path in paths {
        let key = (path.0, path.1.as_str());
        if seen_paths.insert(key) {
            unique_paths.push(path);
        }
    }

    // Build result in input order
    let mut filtered = Vec::new();
    let mut matched_paths: HashSet<(ComponentKind, &str)> = HashSet::new();

    for path in &unique_paths {
        let key = (path.0, path.1.as_str());
        // Find matching component
        if let Some(component) = components
            .iter()
            .find(|c| c.kind == path.0 && c.name == path.1)
        {
            if matched_paths.insert(key) {
                filtered.push(component.clone());
            }
        }
    }

    // Collect skipped paths (not found in components)
    let mut seen_skipped: HashSet<String> = HashSet::new();
    let mut skipped = Vec::new();

    for path in &unique_paths {
        let key = (path.0, path.1.as_str());
        if !matched_paths.contains(&key) {
            let path_str = format!("{}/{}", path.0.plural(), path.1);
            if seen_skipped.insert(path_str.clone()) {
                skipped.push(path_str);
            }
        }
    }

    (filtered, skipped)
}

struct ImportContext<'a> {
    origin: &'a PluginOrigin,
    scope: Scope,
    project_root: &'a Path,
    source_repo: &'a str,
    git_ref: &'a str,
    commit_sha: &'a str,
}

enum DeployOutcome {
    Success,
    Failure,
    /// Deploy succeeded but ImportRecord recording skipped (e.g. gemini)
    Skipped,
}

fn build_deployment(
    target: &dyn Target,
    component: &Component,
    ctx: &ImportContext,
) -> Result<Option<ComponentDeployment>, String> {
    if !target.supports(component.kind) {
        return Ok(None);
    }

    let placement_ctx = PlacementContext {
        component: ComponentRef::new(component.kind, &component.name),
        origin: ctx.origin,
        scope: PlacementScope(ctx.scope),
        project: ProjectContext::new(ctx.project_root),
    };

    let target_path = match target.placement_location(&placement_ctx) {
        Some(location) => location.into_path(),
        None => return Ok(None),
    };

    let mut builder = ComponentDeployment::builder()
        .component(component)
        .scope(ctx.scope)
        .target_path(target_path);

    if component.kind == ComponentKind::Agent {
        builder = builder
            .source_agent_format(AgentFormat::ClaudeCode)
            .dest_agent_format(target.agent_format());
    }

    builder.build().map(Some).map_err(|e| e.to_string())
}

fn deploy_one(
    deployment: &ComponentDeployment,
    target_name: &str,
    ctx: &ImportContext,
    import_registry: &mut ImportRegistry,
) -> DeployOutcome {
    match deployment.execute() {
        Ok(_result) => {
            println!(
                "  + {} {}: {} -> {}",
                target_name, deployment.kind, deployment.name,
                deployment.path().display()
            );

            let target_kind = match target_name {
                "antigravity" => TargetKind::Antigravity,
                "codex" => TargetKind::Codex,
                "copilot" => TargetKind::Copilot,
                _ => return DeployOutcome::Skipped,
            };

            let record = ImportRecord {
                source_repo: ctx.source_repo.to_string(),
                kind: deployment.kind,
                name: deployment.name.clone(),
                target: target_kind,
                scope: deployment.scope,
                path: deployment.path().to_path_buf(),
                imported_at: Utc::now().to_rfc3339(),
                git_ref: ctx.git_ref.to_string(),
                commit_sha: ctx.commit_sha.to_string(),
            };
            if let Err(e) = import_registry.record(record) {
                eprintln!("Warning: Failed to record import history: {}", e);
            }

            DeployOutcome::Success
        }
        Err(e) => {
            println!(
                "  x {} {}: {} - {}",
                target_name, deployment.kind, deployment.name, e
            );
            DeployOutcome::Failure
        }
    }
}

fn place_components(
    target_names: &[String],
    components: &[Component],
    ctx: &ImportContext,
    import_registry: &mut ImportRegistry,
) -> Result<(usize, usize), String> {
    let mut total_success = 0;
    let mut total_failure = 0;

    for target_name in target_names {
        let target = parse_target(target_name).map_err(|e| e.to_string())?;

        for component in components {
            let deployment = match build_deployment(target.as_ref(), component, ctx) {
                Ok(None) => continue,
                Ok(Some(d)) => d,
                Err(e) => {
                    println!(
                        "  x {} {}: {} - {}",
                        target.name(), component.kind, component.name, e
                    );
                    total_failure += 1;
                    continue;
                }
            };

            match deploy_one(&deployment, target.name(), ctx, import_registry) {
                DeployOutcome::Success => total_success += 1,
                DeployOutcome::Failure => total_failure += 1,
                DeployOutcome::Skipped => {}
            }
        }
    }

    Ok((total_success, total_failure))
}

pub async fn run(args: Args) -> Result<(), String> {
    // 1. Parse component paths (validate early, before download)
    let component_paths: Vec<(ComponentKind, String)> = args
        .component
        .iter()
        .map(|p| parse_component_path(p))
        .collect::<Result<Vec<_>, _>>()?;

    // 2. Target selection (before download)
    let target_names: Vec<String> = match &args.target {
        Some(targets) => targets.iter().map(|t| t.as_str().to_string()).collect(),
        None => {
            let available = all_targets();
            let available_refs: Vec<&dyn Target> = available.iter().map(|t| t.as_ref()).collect();
            let all_components = ComponentKind::all().to_vec();

            tui::select_targets(&available_refs, &all_components).map_err(|e| e.to_string())?
        }
    };

    if target_names.is_empty() {
        return Err("No targets selected".to_string());
    }

    // 3. Scope selection (before download)
    let scope: Scope = match args.scope {
        Some(s) => s,
        None => tui::select_scope().map_err(|e| e.to_string())?,
    };

    println!("\nSelected targets: {}", target_names.join(", "));
    println!("Selected scope: {}", scope);

    // 4. Parse source
    let source = parse_source(&args.source).map_err(|e| e.to_string())?;

    // 5. Download plugin
    println!("\nDownloading plugin...");
    let cached_plugin = source
        .download(args.force)
        .await
        .map_err(|e| e.to_string())?;

    println!("\nPlugin downloaded successfully!");
    println!("  Name: {}", cached_plugin.name);
    println!("  Version: {}", cached_plugin.version());
    println!("  Path: {}", cached_plugin.path.display());
    println!("  Ref: {}", cached_plugin.git_ref);
    println!("  SHA: {}", cached_plugin.commit_sha);

    if let Some(desc) = cached_plugin.description() {
        println!("  Description: {}", desc);
    }

    // 6. Scan components
    let components = cached_plugin.components();

    // 7. Filter components
    let (filtered_components, skipped_paths) =
        filter_components(components, &component_paths, &args.component_type);

    // Show warnings for skipped paths
    for path in &skipped_paths {
        eprintln!("Warning: Component not found: {}", path);
    }

    // Check if any components to import
    if filtered_components.is_empty() {
        return Err("No components to import".to_string());
    }

    println!("\nComponents to import: {}", filtered_components.len());
    for c in &filtered_components {
        println!("  - {}: {}", c.kind, c.name);
    }

    // 8. Build import context
    let project_root = env::current_dir().map_err(|e| e.to_string())?;
    let origin =
        PluginOrigin::from_cached_plugin(cached_plugin.marketplace.as_deref(), &cached_plugin.name);

    let ctx = ImportContext {
        origin: &origin,
        scope,
        project_root: &project_root,
        source_repo: &args.source,
        git_ref: &cached_plugin.git_ref,
        commit_sha: &cached_plugin.commit_sha,
    };

    // 9. Place components
    println!("\nPlacing to targets...");
    let mut import_registry = ImportRegistry::new().map_err(|e| e.to_string())?;

    println!("\nPlacement Results:");
    let (total_success, total_failure) =
        place_components(&target_names, &filtered_components, &ctx, &mut import_registry)?;

    // 10. Summary
    let summary = CommandSummary::format(total_success, total_failure);
    println!("\n{} {}", summary.prefix, summary.message);

    Ok(())
}

#[cfg(test)]
#[path = "import_test.rs"]
mod tests;
