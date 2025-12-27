use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    Prompt,
    Instruction,
}

#[derive(Debug, Parser)]
pub struct Args {
    /// owner/repo 形式
    pub repo: String,

    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    #[arg(long)]
    pub component: Option<String>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("import: {:?}", args);
    Err("not implemented".to_string())
}
