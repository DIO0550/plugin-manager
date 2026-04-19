use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub path: String,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm pack`.
pub async fn run(args: Args) -> Result<(), String> {
    println!("pack: {:?}", args);
    Err("not implemented".to_string())
}
