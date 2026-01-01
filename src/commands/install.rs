//! プラグインインストールコマンド
//!
//! ## フロー
//!
//! 1. TUI選択（ダウンロード前）
//!    - `--target` 未指定時: TUIでターゲット選択
//!    - `--scope` 未指定時: TUIでスコープ選択
//! 2. ダウンロード
//! 3. 配置

use crate::component::{ComponentDeployment, ComponentKind};
use crate::domain::{ComponentRef, PlacementContext, PlacementScope, ProjectContext};
use crate::output::CommandSummary;
use crate::source::parse_source;
use crate::target::{all_targets, parse_target, PluginOrigin, Scope, Target, TargetKind};
use crate::tui;
use clap::Parser;
use std::env;


#[derive(Debug, Parser)]
pub struct Args {
    /// owner/repo 形式、または plugin@marketplace 形式
    pub source: String,

    /// コンポーネント種別を指定（複数指定可、未指定なら全コンポーネント）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<Vec<ComponentKind>>,

    /// デプロイ先ターゲット（複数指定可、未指定ならTUIで選択）
    #[arg(long, value_enum)]
    pub target: Option<Vec<TargetKind>>,

    /// デプロイスコープ（未指定ならTUIで選択）
    #[arg(long, value_enum)]
    pub scope: Option<Scope>,

    /// キャッシュを無視して再ダウンロード
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: Args) -> std::result::Result<(), String> {
    // 1. ターゲット選択（ダウンロード前）
    let target_names: Vec<String> = match &args.target {
        Some(targets) => targets.iter().map(|t| t.as_str().to_string()).collect(),
        None => {
            // TUIでターゲット選択
            let available = all_targets();
            let available_refs: Vec<&dyn Target> = available.iter().map(|t| t.as_ref()).collect();
            let all_components = ComponentKind::all().to_vec();

            tui::select_targets(&available_refs, &all_components).map_err(|e| e.to_string())?
        }
    };

    if target_names.is_empty() {
        return Err("No targets selected".to_string());
    }

    // 2. スコープ選択（ダウンロード前）
    let scope: Scope = match args.scope {
        Some(s) => s,
        None => {
            // TUIでスコープ選択
            tui::select_scope().map_err(|e| e.to_string())?
        }
    };

    println!("\nSelected targets: {}", target_names.join(", "));
    println!("Selected scope: {}", scope);

    // 3. ソースをパース
    let source = parse_source(&args.source).map_err(|e| e.to_string())?;

    // 4. ダウンロード
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

    // コンポーネント情報
    println!("\nComponents:");
    if let Some(skills) = cached_plugin.skills() {
        println!("  - Skills: {}", skills);
    }
    if let Some(agents) = cached_plugin.agents() {
        println!("  - Agents: {}", agents);
    }
    if let Some(commands) = cached_plugin.commands() {
        println!("  - Commands: {}", commands);
    }

    // 5. コンポーネントをスキャン
    let mut components = cached_plugin.components();

    // コンポーネントフィルタを適用
    if let Some(filter) = &args.component_type {
        components.retain(|c| filter.contains(&c.kind));
    }

    // 6. 配置
    println!("\nPlacing to targets...");

    let project_root = env::current_dir().map_err(|e| e.to_string())?;

    // プラグインの出自情報を作成
    let origin = PluginOrigin::from_cached_plugin(
        cached_plugin.marketplace.as_deref(),
        &cached_plugin.name,
    );

    let mut total_success = 0;
    let mut total_failure = 0;

    println!("\nPlacement Results:");
    for target_name in &target_names {
        let target = parse_target(target_name).map_err(|e| e.to_string())?;

        let mut target_success = true;

        for component in &components {
            // ターゲットがこのコンポーネントをサポートしているか確認
            if !target.supports(component.kind) {
                continue;
            }

            // 配置コンテキストを構築
            let ctx = PlacementContext {
                component: ComponentRef::new(component.kind, &component.name),
                origin: &origin,
                scope: PlacementScope(scope),
                project: ProjectContext::new(&project_root),
            };

            // 配置先を取得（サポートしていない場合は None）
            let target_path = match target.placement_location(&ctx) {
                Some(location) => location.into_path(),
                None => continue,
            };

            // デプロイ情報を構築
            let deployment = match ComponentDeployment::builder()
                .component(component)
                .scope(scope)
                .target_path(target_path)
                .build()
            {
                Ok(d) => d,
                Err(e) => {
                    println!(
                        "  x {} {}: {} - {}",
                        target.name(),
                        component.kind,
                        component.name,
                        e
                    );
                    total_failure += 1;
                    target_success = false;
                    continue;
                }
            };

            // デプロイ実行
            match deployment.execute() {
                Ok(()) => {
                    println!(
                        "  + {} {}: {} -> {}",
                        target.name(),
                        component.kind,
                        component.name,
                        deployment.path().display()
                    );
                    total_success += 1;
                }
                Err(e) => {
                    println!(
                        "  x {} {}: {} - {}",
                        target.name(),
                        component.kind,
                        component.name,
                        e
                    );
                    total_failure += 1;
                    target_success = false;
                }
            }
        }

        if !target_success {
            println!("  {} - FAILED", target.name());
        }
    }

    // 結果サマリー
    let summary = CommandSummary::format(total_success, total_failure);
    println!("\n{} {}", summary.prefix, summary.message);

    Ok(())
}
