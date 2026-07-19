//! plm import コマンド
//!
//! Claude Code Plugin形式のGitHubリポジトリから、
//! 特定のコンポーネントを選択してインポートする。

use crate::commands::args::{InteractiveScopeArgs, MultiTargetArgs};
use crate::component::{
    AgentFormat, Component, ComponentDeployment, ComponentKind, ConversionConfig,
};
use crate::component::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::import::{ImportRecord, ImportRegistry};
use crate::output::CommandSummary;
use crate::plugin::PackageCache;
use crate::source::parse_source;
use crate::target::{
    all_targets, apply_codex_hooks_flag, parse_target, CodexTarget, CursorTarget, PluginOrigin,
    Scope, Target, TargetKind,
};
use crate::tui;
use chrono::Utc;
use clap::Parser;
use std::cell::Cell;
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

    #[command(flatten)]
    pub target: MultiTargetArgs,

    #[command(flatten)]
    pub scope: InteractiveScopeArgs,

    /// Force re-download even if cached
    #[arg(long)]
    pub force: bool,

    /// Codex Hook 配置時の `[features] codex_hooks = true` 自動追記を抑止する。
    /// デフォルトでは `~/.codex/config.toml`（または project の `.codex/config.toml`）
    /// に追記される。`--no-enable-flag` を指定すると config.toml には触れない。
    #[arg(long = "no-enable-flag", action = clap::ArgAction::SetFalse, default_value_t = true)]
    pub enable_flag: bool,
}

/// Parse a component path string into (ComponentKind, name)
///
/// Format: `<plural_kind>/<name>`
/// - plural_kind: skills, agents, commands, instructions, hooks (case-insensitive)
/// - name: component name (case-sensitive)
///
/// # Arguments
///
/// * `path` - Component path in `<plural_kind>/<name>` form.
///
/// # Examples
/// - "skills/pdf" -> Ok((Skill, "pdf"))
/// - "SKILLS/pdf" -> Ok((Skill, "pdf")) (kind normalized)
/// - "skills/PDF" -> Ok((Skill, "PDF")) (name preserved)
/// - "skill/pdf" -> Err (singular not allowed)
pub fn parse_component_path(path: &str) -> Result<(ComponentKind, String), String> {
    let path = path.trim();

    let path = path.trim_end_matches('/');

    if path.contains("//") {
        return Err(format!(
            "Invalid component path format: '{}'. Consecutive slashes are not allowed.",
            path
        ));
    }

    let (kind_str, name) = path.split_once('/').ok_or_else(|| {
        format!(
            "Invalid component path format: '{}'. Expected: <kind>/<name>",
            path
        )
    })?;

    if name.contains('/') {
        return Err(format!(
            "Invalid component path format: '{}'. Nested paths are not allowed.",
            path
        ));
    }

    if name.is_empty() {
        return Err(format!(
            "Invalid component path format: '{}'. Component name cannot be empty.",
            path
        ));
    }

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
///
/// # Arguments
///
/// * `components` - Candidate components scanned from the plugin.
/// * `paths` - Exact (kind, name) pairs requested by the user.
/// * `types` - Component kinds requested by the user.
pub fn filter_components(
    components: Vec<Component>,
    paths: &[(ComponentKind, String)],
    types: &[ComponentKind],
) -> (Vec<Component>, Vec<String>) {
    use std::collections::HashSet;

    if paths.is_empty() && types.is_empty() {
        return (components, vec![]);
    }

    if !types.is_empty() {
        let filtered = components
            .into_iter()
            .filter(|c| types.contains(&c.kind))
            .collect();
        return (filtered, vec![]);
    }

    let mut seen_paths: HashSet<(ComponentKind, &str)> = HashSet::new();
    let mut unique_paths: Vec<&(ComponentKind, String)> = Vec::new();

    for path in paths {
        let key = (path.0, path.1.as_str());
        if seen_paths.insert(key) {
            unique_paths.push(path);
        }
    }

    let mut filtered = Vec::new();
    let mut matched_paths: HashSet<(ComponentKind, &str)> = HashSet::new();

    for path in &unique_paths {
        let key = (path.0, path.1.as_str());
        if let Some(component) = components
            .iter()
            .find(|c| c.kind == path.0 && c.name == path.1)
        {
            if matched_paths.insert(key) {
                filtered.push(component.clone());
            }
        }
    }

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
    plugin_root: &'a Path,
    source_repo: &'a str,
    git_ref: &'a str,
    commit_sha: &'a str,
    /// Codex Hook 配置時に `[features] codex_hooks = true` を自動追記するか。
    enable_codex_hooks_flag: bool,
    /// この import 呼び出し内で feature flag を既に適用したかどうか。
    /// place_components はターゲット/コンポーネントをループするため、
    /// scope 内で 1 回のみ適用するためのガード。
    codex_flag_applied: Cell<bool>,
}

enum DeployOutcome {
    Success,
    Failure,
    /// Deploy succeeded but ImportRecord recording skipped (e.g. gemini)
    Skipped,
}

/// # Arguments
///
/// * `target` - Target environment adapter to deploy to.
/// * `component` - Component being prepared for deployment.
/// * `ctx` - Shared import context (origin, scope, project root).
fn build_deployment(
    target: &dyn Target,
    component: &Component,
    ctx: &ImportContext,
) -> Result<Option<ComponentDeployment>, String> {
    if !target.supports(component.kind) {
        return Ok(None);
    }

    let placement_ctx = PlacementContext {
        component: ComponentRef::from(component),
        origin: ctx.origin,
        scope: PlacementScope::new(ctx.scope),
        project: ProjectContext::new(ctx.project_root),
    };

    let target_path = match target.placement_location(&placement_ctx) {
        Some(location) => location.into_path(),
        None => return Ok(None),
    };

    if component.kind == ComponentKind::Hook {
        let overwrite_error = match target.kind() {
            TargetKind::Codex => CodexTarget::hook_overwrite_error(&target_path, ctx.plugin_root),
            TargetKind::Cursor => CursorTarget::hook_overwrite_error(&target_path, ctx.plugin_root),
            _ => None,
        };
        if let Some(error) = overwrite_error {
            return Err(error);
        }
    }

    if component.kind == ComponentKind::Skill && target.kind() == TargetKind::Cursor {
        if let Some(error) = CursorTarget::skill_overwrite_error(&target_path, ctx.plugin_root) {
            return Err(error);
        }
    }

    let conversion = match component.kind {
        ComponentKind::Agent => ConversionConfig::Agent {
            source: AgentFormat::ClaudeCode,
            dest: target.agent_format(),
        },
        ComponentKind::Hook
            if matches!(
                target.kind(),
                TargetKind::Codex | TargetKind::Copilot | TargetKind::Cursor
            ) =>
        {
            ConversionConfig::Hook {
                target_kind: target.kind(),
                plugin_root: Some(ctx.plugin_root.to_path_buf()),
            }
        }
        ComponentKind::Skill => ConversionConfig::Skill {
            target_kind: target.kind(),
        },
        _ => ConversionConfig::None,
    };

    ComponentDeployment::builder()
        .component(component.clone())
        .scope(ctx.scope)
        .target_path(target_path)
        .conversion(conversion)
        .build()
        .map(Some)
        .map_err(|e| e.to_string())
}

/// # Arguments
///
/// * `deployment` - Prepared deployment to execute.
/// * `target` - Target environment adapter receiving the component.
/// * `ctx` - Shared import context (source repo, git ref, commit sha).
/// * `import_registry` - Registry that records successful imports.
fn deploy_one(
    deployment: &ComponentDeployment,
    target: &dyn Target,
    ctx: &ImportContext,
    import_registry: &mut ImportRegistry,
) -> DeployOutcome {
    match deployment.execute() {
        Ok(_result) => {
            println!(
                "  + {} {}: {} -> {}",
                target.name(),
                deployment.kind(),
                deployment.name(),
                deployment.path().display()
            );

            let target_kind = target.kind();
            if deployment.kind() == ComponentKind::Hook {
                match target_kind {
                    TargetKind::Codex => {
                        crate::install::record_hook_file_ownership(
                            ctx.plugin_root,
                            deployment.path(),
                            "codex",
                        );

                        if ctx.enable_codex_hooks_flag && !ctx.codex_flag_applied.get() {
                            let config_path =
                                CodexTarget::config_toml_path(ctx.scope, ctx.project_root);
                            match apply_codex_hooks_flag(&config_path) {
                                Ok(ffo) => {
                                    if ffo.applied {
                                        println!(
                                            "  + codex: Enabled [features] codex_hooks = true in {}",
                                            ffo.target_path.display()
                                        );
                                    } else if let Some(reason) = &ffo.skipped_reason {
                                        println!(
                                            "  - codex: Skipped feature flag in {}: {}",
                                            ffo.target_path.display(),
                                            reason
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "  ! codex: Failed to enable feature flag in {}: {}",
                                        config_path.display(),
                                        e
                                    );
                                }
                            }
                            ctx.codex_flag_applied.set(true);
                        }
                    }
                    TargetKind::Cursor => {
                        crate::install::record_hook_file_ownership(
                            ctx.plugin_root,
                            deployment.path(),
                            "cursor",
                        );
                    }
                    _ => {}
                }
            }

            if deployment.kind() == ComponentKind::Skill && target_kind == TargetKind::Cursor {
                crate::install::record_cursor_skill_ownership(ctx.plugin_root, deployment.path());
                if let Some(original) = deployment.original_name() {
                    let flattened = deployment.name();
                    if flattened != original {
                        CursorTarget::remove_legacy_flattened_skill_dir(
                            ctx.scope,
                            ctx.project_root,
                            flattened,
                            deployment.path(),
                        );
                    }
                }
            }

            if target_kind == TargetKind::GeminiCli {
                return DeployOutcome::Skipped;
            }

            let record = ImportRecord {
                source_repo: ctx.source_repo.to_string(),
                kind: deployment.kind(),
                name: deployment.name().to_string(),
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
                target.name(),
                deployment.kind(),
                deployment.name(),
                e
            );
            DeployOutcome::Failure
        }
    }
}

/// # Arguments
///
/// * `target_names` - Names of targets to deploy to.
/// * `components` - Components selected for placement.
/// * `ctx` - Shared import context.
/// * `import_registry` - Registry that records successful imports.
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
        let hook_component_conflict = match target.kind() {
            TargetKind::Codex => CodexTarget::hook_component_conflict_error(components),
            TargetKind::Cursor => CursorTarget::hook_component_conflict_error(components),
            _ => None,
        };

        for component in components {
            if component.kind == ComponentKind::Hook {
                if let Some(error) = &hook_component_conflict {
                    println!(
                        "  x {} {}: {} - {}",
                        target.name(),
                        component.kind,
                        component.name,
                        error
                    );
                    total_failure += 1;
                    continue;
                }
            }

            let deployment = match build_deployment(target.as_ref(), component, ctx) {
                Ok(None) => continue,
                Ok(Some(d)) => d,
                Err(e) => {
                    println!(
                        "  x {} {}: {} - {}",
                        target.name(),
                        component.kind,
                        component.name,
                        e
                    );
                    total_failure += 1;
                    continue;
                }
            };

            match deploy_one(&deployment, target.as_ref(), ctx, import_registry) {
                DeployOutcome::Success => total_success += 1,
                DeployOutcome::Failure => total_failure += 1,
                DeployOutcome::Skipped => {}
            }
        }
    }

    Ok((total_success, total_failure))
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm import`.
pub async fn run(args: Args) -> Result<(), String> {
    // Parse component paths up-front so validation errors surface before the
    // potentially expensive plugin download below.
    let component_paths: Vec<(ComponentKind, String)> = args
        .component
        .iter()
        .map(|p| parse_component_path(p))
        .collect::<Result<Vec<_>, _>>()?;

    // Target and scope selection happen before download so the user can cancel
    // without paying the download cost.
    let target_names: Vec<String> = match &args.target.target {
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

    let scope: Scope = match args.scope.scope {
        Some(s) => s,
        None => tui::select_scope().map_err(|e| e.to_string())?,
    };

    println!("\nSelected targets: {}", target_names.join(", "));
    println!("Selected scope: {}", scope);

    let source = parse_source(&args.source).map_err(|e| e.to_string())?;

    println!("\nDownloading plugin...");
    let cache = PackageCache::new().map_err(|e| format!("Failed to access cache: {e}"))?;
    let cached_plugin = source
        .download(&cache, args.force)
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

    let package =
        crate::plugin::MarketplaceContent::try_from(&cached_plugin).map_err(|e| e.to_string())?;
    let components = package.components();

    let (filtered_components, skipped_paths) =
        filter_components(components, &component_paths, &args.component_type);

    for path in &skipped_paths {
        eprintln!("Warning: Component not found: {}", path);
    }

    if filtered_components.is_empty() {
        return Err("No components to import".to_string());
    }

    println!("\nComponents to import: {}", filtered_components.len());
    for c in &filtered_components {
        println!("  - {}: {}", c.kind, c.name);
    }

    let project_root = env::current_dir().map_err(|e| e.to_string())?;
    let origin =
        PluginOrigin::from_cached_plugin(cached_plugin.marketplace.as_deref(), cached_plugin.id());

    let ctx = ImportContext {
        origin: &origin,
        scope,
        project_root: &project_root,
        plugin_root: &cached_plugin.path,
        source_repo: &args.source,
        git_ref: &cached_plugin.git_ref,
        commit_sha: &cached_plugin.commit_sha,
        enable_codex_hooks_flag: args.enable_flag,
        codex_flag_applied: Cell::new(false),
    };

    println!("\nPlacing to targets...");
    let mut import_registry = ImportRegistry::new().map_err(|e| e.to_string())?;

    println!("\nPlacement Results:");
    let (total_success, total_failure) = place_components(
        &target_names,
        &filtered_components,
        &ctx,
        &mut import_registry,
    )?;

    let summary = CommandSummary::format(total_success, total_failure);
    println!("\n{} {}", summary.prefix, summary.message);

    Ok(())
}

#[cfg(test)]
#[path = "import_test.rs"]
mod tests;
