//! plm init コマンド
//!
//! コンポーネント（skill / agent / command）のテンプレートを生成する。

use crate::component::ComponentName;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

/// `plm init --type` で指定可能なコンポーネント種別。
///
/// ドメインの [`crate::component::ComponentKind`] のうち、テンプレート生成対象のみ。
/// 旧 CLI 値 `prompt` は `command` の alias。
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    #[value(alias = "prompt")]
    Command,
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Component name
    pub name: String,

    /// Component type to generate
    #[arg(long = "type", value_enum)]
    pub component_type: ComponentType,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm init`.
pub async fn run(args: Args) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let created = init_in_dir(&args, &cwd)?;
    print_created(&args.component_type, &created);
    Ok(())
}

/// 指定ディレクトリ配下にテンプレートを生成する（テスト注入用）。
///
/// # Arguments
///
/// * `args` - CLI 引数
/// * `base_dir` - 生成先の親ディレクトリ（通常は CWD）
pub(crate) fn init_in_dir(args: &Args, base_dir: &Path) -> Result<PathBuf, String> {
    let name = ComponentName::new(&args.name).ok_or_else(|| {
        format!(
            "Invalid component name '{}': must be a single path segment",
            args.name
        )
    })?;

    match args.component_type {
        ComponentType::Skill => create_skill(base_dir, name),
        ComponentType::Agent => create_agent(base_dir, name),
        ComponentType::Command => create_command(base_dir, name),
    }
}

fn create_skill(base_dir: &Path, name: ComponentName<'_>) -> Result<PathBuf, String> {
    let dir = base_dir.join(name.as_str());
    if dir.exists() {
        return Err(format!("Path already exists: {}", dir.display()));
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("SKILL.md");
    let content = skill_template(name);
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn create_agent(base_dir: &Path, name: ComponentName<'_>) -> Result<PathBuf, String> {
    let path = base_dir.join(format!("{}.agent.md", name.as_str()));
    if path.exists() {
        return Err(format!("Path already exists: {}", path.display()));
    }
    fs::write(&path, agent_template(name)).map_err(|e| e.to_string())?;
    Ok(path)
}

fn create_command(base_dir: &Path, name: ComponentName<'_>) -> Result<PathBuf, String> {
    let path = base_dir.join(format!("{}.prompt.md", name.as_str()));
    if path.exists() {
        return Err(format!("Path already exists: {}", path.display()));
    }
    fs::write(&path, command_template(name)).map_err(|e| e.to_string())?;
    Ok(path)
}

fn skill_template(name: ComponentName<'_>) -> String {
    let name = name.as_str();
    format!(
        r#"---
name: {name}
description: スキルの説明
metadata:
  short-description: 短い説明
---

# {name}

スキルの詳細な指示をここに記述...
"#
    )
}

fn agent_template(name: ComponentName<'_>) -> String {
    let name = name.as_str();
    format!(
        r#"---
name: {name}
description: エージェントの説明
tools: ['search', 'fetch', 'edit']
---

# {name}

エージェントの指示をここに記述...
"#
    )
}

fn command_template(name: ComponentName<'_>) -> String {
    let name = name.as_str();
    format!(
        r#"---
name: {name}
description: コマンドの説明
---

# {name}

コマンドの内容をここに記述...
"#
    )
}

fn print_created(kind: &ComponentType, path: &Path) {
    match kind {
        ComponentType::Skill => {
            println!(
                "📁 Created {}/",
                path.file_name().unwrap().to_string_lossy()
            );
            println!("   └── SKILL.md");
        }
        ComponentType::Agent | ComponentType::Command => {
            println!("📁 Created {}", path.file_name().unwrap().to_string_lossy());
        }
    }
}

#[cfg(test)]
#[path = "init_test.rs"]
mod tests;
