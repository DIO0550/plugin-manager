use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

#[derive(Debug, Parser)]
pub struct Args {
    pub name: String,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("enable: {:?}", args);
    Err("not implemented".to_string())
}
