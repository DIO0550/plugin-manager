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
    pub name: String,

    #[arg(long = "type", value_enum)]
    pub component_type: ComponentType,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("init: {:?}", args);
    Err("not implemented".to_string())
}
