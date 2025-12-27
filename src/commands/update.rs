use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub name: Option<String>,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("update: {:?}", args);
    Err("not implemented".to_string())
}
