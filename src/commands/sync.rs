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
    #[arg(long, value_enum)]
    pub from: TargetKind,

    #[arg(long, value_enum)]
    pub to: TargetKind,

    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("sync: {:?}", args);
    Err("not implemented".to_string())
}
