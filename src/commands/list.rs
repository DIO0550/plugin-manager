use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    Prompt,
    Instruction,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("list: {:?}", args);
    Err("not implemented".to_string())
}
