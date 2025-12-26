use crate::cli::DisableArgs;

pub async fn run(args: DisableArgs) -> Result<(), String> {
    println!("disable: {:?}", args);
    Err("not implemented".to_string())
}
