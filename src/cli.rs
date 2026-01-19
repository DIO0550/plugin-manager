use clap::{Parser, Subcommand};

use crate::commands::{
    disable, enable, import, info, init, install, list, marketplace, pack, sync, target, uninstall,
    update,
};

#[derive(Debug, Parser)]
#[command(name = "plm")]
#[command(about = "Plugin Manager CLI")]
#[command(long_about = r#"PLM (Plugin Manager CLI) is a unified tool for managing plugins across AI coding assistants.

Supported environments:
  - OpenAI Codex: Skills, Instructions
  - VSCode Copilot: Skills, Agents, Prompts, Instructions

Install plugins from GitHub or marketplaces, manage their lifecycle, and keep them synchronized across environments."#)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage target environments
    #[command(long_about = "Manage target environments where plugins are deployed. Add or remove codex/copilot targets from your PLM configuration.")]
    Target(target::Args),

    /// Manage plugin marketplaces
    #[command(long_about = "Manage plugin marketplaces. Register GitHub repositories as marketplaces to browse and install plugins from.")]
    Marketplace(marketplace::Args),

    /// Install plugins from marketplace or GitHub
    #[command(long_about = r#"Install plugins from GitHub repositories or registered marketplaces.

SOURCE FORMATS:
  owner/repo              GitHub repository (e.g., user/my-plugin)
  plugin@marketplace      Plugin from a registered marketplace

OPTIONS:
  --type    Filter which component types to install (skill, agent, command, instruction)
  --target  Specify which environments to deploy to (codex, copilot)
  --scope   Choose personal or project scope
  --force   Re-download even if cached"#)]
    Install(install::Args),

    /// List installed components
    #[command(long_about = r#"List installed plugins with their components and status.

OUTPUT FORMATS:
  (default)   Table format with Name, Version, Components, Status, Marketplace
  --json      JSON array for scripting
  --simple    Plugin names only, one per line

FILTERING:
  --type      Filter by component type
  --target    Filter by target environment
  --outdated  Show only plugins with available updates"#)]
    List(list::Args),

    /// Show component details
    #[command(long_about = r#"Show detailed information about an installed plugin.

SECTIONS DISPLAYED:
  - Plugin Information (name, version, description)
  - Author (name, email, URL)
  - Installation (date, source)
  - Components (skills, agents, commands, instructions, hooks)
  - Deployment (status, cache path)

OUTPUT FORMATS:
  -f, --format table  Default table view
  -f, --format json   JSON for scripting
  -f, --format yaml   YAML format"#)]
    Info(info::Args),

    /// Enable a component
    #[command(long_about = r#"Enable a plugin by deploying its components from cache to target environments.

The plugin must already be installed (cached). Components are copied from the cache directory to the appropriate target locations.

OPTIONS:
  --target           Enable for a specific environment only (codex, copilot)
  -m, --marketplace  Specify marketplace name (default: github)"#)]
    Enable(enable::Args),

    /// Disable a component
    #[command(long_about = r#"Disable a plugin by removing its components from target environments.

Components are removed from target locations but the cache is preserved, allowing the plugin to be re-enabled without re-downloading.

OPTIONS:
  --target           Disable for a specific environment only (codex, copilot)
  -m, --marketplace  Specify marketplace name (default: github)"#)]
    Disable(disable::Args),

    /// Remove a component
    #[command(long_about = r#"Completely remove a plugin including cache and deployed components.

The plugin and all its components will be removed from all target environments. This action cannot be undone.

OPTIONS:
  -m, --marketplace  Specify marketplace name
  -f, --force        Skip confirmation prompt"#)]
    Uninstall(uninstall::Args),

    /// Update components
    #[command(long_about = r#"Check for and apply updates to installed plugins.

Specify a plugin name to update a single plugin, or use --all to update all installed plugins.

OPTIONS:
  --all     Update all installed plugins
  --target  Filter by target environment (codex, copilot)"#)]
    Update(update::Args),

    /// Generate templates
    #[command(long_about = r#"Generate plugin templates for creating new plugins.

Creates a new plugin skeleton with the specified component type.

OPTIONS:
  --type  Component type to generate (skill, agent, prompt, instruction)"#)]
    Init(init::Args),

    /// Create distribution package
    Pack(pack::Args),

    /// Sync between environments
    #[command(long_about = r#"Synchronize plugins between different target environments.

Copies components from one target to another, supporting create, update, and delete operations.

OPTIONS:
  --from     Source target environment (codex, copilot)
  --to       Destination target environment (codex, copilot)
  --type     Component type to sync (all if not specified)
  --scope    Scope to sync (personal, project, or both)
  --dry-run  Preview changes without applying them"#)]
    Sync(sync::Args),

    /// Import from Claude Code Plugin
    #[command(long_about = r#"Import existing Claude Code Plugins and deploy them to other environments.

Fetches plugins from a Claude Code Plugin repository and converts them for use with PLM.

OPTIONS:
  --type       Component type to import (skill, agent, prompt, instruction)
  --component  Specific component name to import"#)]
    Import(import::Args),

    /// Plugin management (TUI)
    #[command(long_about = "Open interactive TUI for managing plugins visually.")]
    Managed,
}
