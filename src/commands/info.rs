use crate::cli::InfoArgs;

pub async fn run(args: InfoArgs) -> Result<(), String> {
    println!("info: {:?}", args);
    Err("not implemented".to_string())
}
