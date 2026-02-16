use crate::marketplace::{
    normalize_name, normalize_source_path, to_display_source, MarketplaceConfig,
    MarketplaceFetcher, MarketplaceRegistration, MarketplaceRegistry,
};
use crate::repo;
use clap::{Parser, Subcommand};
use comfy_table::{presets::UTF8_FULL_CONDENSED, Table};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List registered marketplaces
    #[command(
        long_about = "Display all registered plugin marketplaces with their source repositories."
    )]
    List,

    /// Add a marketplace
    #[command(
        long_about = "Register a GitHub repository as a plugin marketplace. Use owner/repo format or full URL."
    )]
    Add {
        /// GitHub repository (owner/repo) or URL
        source: String,

        /// Marketplace name (defaults to repository name if not specified)
        #[arg(long)]
        name: Option<String>,

        /// Subdirectory where marketplace.json is located
        #[arg(long)]
        path: Option<String>,
    },

    /// Remove a marketplace
    #[command(
        long_about = "Unregister a marketplace. Installed plugins from this marketplace are not affected."
    )]
    Remove {
        /// Marketplace name
        name: String,
    },

    /// Update marketplace cache
    #[command(
        long_about = "Refresh the local cache for marketplaces. Fetches latest plugin listings from remote repositories."
    )]
    Update {
        /// Update only a specific marketplace (updates all if not specified)
        name: Option<String>,
    },

    /// Show marketplace details
    #[command(
        long_about = "Display detailed information about a marketplace including its plugins."
    )]
    Show {
        /// Marketplace name
        name: String,
    },
}

pub async fn run(args: Args) -> Result<(), String> {
    match args.command {
        Command::List => run_list().await,
        Command::Add { source, name, path } => run_add(source, name, path).await,
        Command::Remove { name } => run_remove(name).await,
        Command::Update { name } => run_update(name).await,
        Command::Show { name } => run_show(name).await,
    }
}

async fn run_list() -> Result<(), String> {
    let config = MarketplaceConfig::load()?;
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;

    let entries = config.list();
    if entries.is_empty() {
        println!("No marketplaces registered.");
        println!("Use 'plm marketplace add <owner/repo>' to add a marketplace.");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["NAME", "SOURCE", "PLUGINS", "LAST UPDATED"]);

    for entry in entries {
        let source_display = to_display_source(&entry.source);
        let (plugins_count, last_updated) = match registry.get(&entry.name) {
            Ok(Some(cache)) => {
                let count = cache.plugins.len().to_string();
                let updated = cache.fetched_at.format("%Y-%m-%d %H:%M").to_string();
                (count, updated)
            }
            _ => ("N/A".to_string(), "Never".to_string()),
        };

        table.add_row(vec![
            &entry.name,
            &source_display,
            &plugins_count,
            &last_updated,
        ]);
    }

    println!("{table}");
    Ok(())
}

async fn run_add(source: String, name: Option<String>, path: Option<String>) -> Result<(), String> {
    // 1. Parse source as owner/repo
    let parsed_repo = repo::from_url(&source).map_err(|e| e.to_string())?;

    // 2. Determine name (--name or repository name)
    let raw_name = name.unwrap_or_else(|| parsed_repo.name().to_string());

    // 3. Normalize name
    let normalized_name = normalize_name(&raw_name)?;

    // 4. Load config and check for duplicates
    let mut config = MarketplaceConfig::load()?;
    if config.exists(&normalized_name) {
        return Err(format!(
            "Marketplace '{}' already exists. Use --name to specify a different name.",
            normalized_name
        ));
    }

    // 5. Normalize source_path
    let source_path = match path {
        Some(p) => normalize_source_path(&p)?,
        None => None,
    };

    // 6. Fetch marketplace.json from GitHub
    println!(
        "Fetching marketplace.json from {}...",
        parsed_repo.full_name()
    );
    let fetcher = MarketplaceFetcher::new();
    let cache = fetcher
        .fetch_as_cache(&parsed_repo, &normalized_name, source_path.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let plugin_count = cache.plugins.len();

    // 7. Add to config
    let entry = MarketplaceRegistration {
        name: normalized_name.clone(),
        source: parsed_repo.full_name(),
        source_path,
    };
    config.add(entry)?;
    config.save()?;

    // 8. Save cache
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    registry.store(&cache).map_err(|e| e.to_string())?;

    println!(
        "Added marketplace '{}' with {} plugin(s).",
        normalized_name, plugin_count
    );
    Ok(())
}

async fn run_remove(name: String) -> Result<(), String> {
    // 1. Load config and check if exists
    let mut config = MarketplaceConfig::load()?;
    if !config.exists(&name) {
        return Err(format!("Marketplace '{}' not found.", name));
    }

    // 2. Remove from config
    config.remove(&name)?;
    config.save()?;

    // 3. Remove cache file (warn if not exists)
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    if let Err(e) = registry.remove(&name) {
        eprintln!("Warning: Failed to remove cache file: {}", e);
    }

    println!("Removed marketplace '{}'.", name);
    Ok(())
}

async fn run_update(name: Option<String>) -> Result<(), String> {
    let config = MarketplaceConfig::load()?;
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;
    let fetcher = MarketplaceFetcher::new();

    // Determine which marketplaces to update
    let entries: Vec<_> = match &name {
        Some(n) => {
            let entry = config
                .get(n)
                .ok_or_else(|| format!("Marketplace '{}' not found.", n))?;
            vec![entry.clone()]
        }
        None => config.list().to_vec(),
    };

    if entries.is_empty() {
        println!("No marketplaces to update.");
        return Ok(());
    }

    let mut successes = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for entry in entries {
        print!("Updating '{}'... ", entry.name);
        let repo = match repo::from_url(&entry.source) {
            Ok(r) => r,
            Err(e) => {
                println!("FAILED");
                failures.push((entry.name.clone(), e.to_string()));
                continue;
            }
        };

        match fetcher
            .fetch_as_cache(&repo, &entry.name, entry.source_path.as_deref())
            .await
        {
            Ok(cache) => {
                if let Err(e) = registry.store(&cache) {
                    println!("FAILED");
                    failures.push((entry.name.clone(), e.to_string()));
                } else {
                    println!("{} plugin(s)", cache.plugins.len());
                    successes += 1;
                }
            }
            Err(e) => {
                println!("FAILED");
                failures.push((entry.name.clone(), e.to_string()));
            }
        }
    }

    // Report results
    if successes > 0 {
        println!("\nUpdated {} marketplace(s).", successes);
    }

    if !failures.is_empty() {
        eprintln!("\nFailed to update {} marketplace(s):", failures.len());
        for (name, error) in &failures {
            eprintln!("  {}: {}", name, error);
        }
        if successes == 0 {
            return Err("All updates failed.".to_string());
        }
    }

    Ok(())
}

async fn run_show(name: String) -> Result<(), String> {
    let config = MarketplaceConfig::load()?;
    let registry = MarketplaceRegistry::new().map_err(|e| e.to_string())?;

    // Get entry from config
    let entry = config
        .get(&name)
        .ok_or_else(|| format!("Marketplace '{}' not found.", name))?;

    // Header
    println!("Marketplace: {}", entry.name);
    println!("Source: {}", to_display_source(&entry.source));
    println!("Path: {}", entry.source_path.as_deref().unwrap_or("(root)"));

    // Get cache
    match registry.get(&name) {
        Ok(Some(cache)) => {
            // Owner info
            if let Some(owner) = &cache.owner {
                let email = owner
                    .email
                    .as_ref()
                    .map(|e| format!(" <{}>", e))
                    .unwrap_or_default();
                println!("Owner: {}{}", owner.name, email);
            }

            // Last updated
            println!(
                "Last Updated: {}",
                cache.fetched_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!();

            // Plugins table
            println!("Plugins ({}):", cache.plugins.len());
            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(vec!["NAME", "DESCRIPTION", "VERSION"]);

            for plugin in &cache.plugins {
                table.add_row(vec![
                    &plugin.name,
                    plugin.description.as_deref().unwrap_or("-"),
                    plugin.version.as_deref().unwrap_or("-"),
                ]);
            }

            println!("{table}");
        }
        _ => {
            println!("Status: (not cached)");
            println!();
            println!(
                "Run 'plm marketplace update {}' to fetch plugin information.",
                name
            );
        }
    }

    Ok(())
}
