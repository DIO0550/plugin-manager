use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub name: String,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("info: {:?}", args);
    Err("not implemented".to_string())
}
