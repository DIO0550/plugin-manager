use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub path: String,
}

pub async fn run(args: Args) -> Result<(), String> {
    println!("pack: {:?}", args);
    Err("not implemented".to_string())
}
