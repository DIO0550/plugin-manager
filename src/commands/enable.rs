use crate::cli::EnableArgs;

pub async fn run(args: EnableArgs) -> Result<(), String> {
    println!("enable: {:?}", args);
    Err("not implemented".to_string())
}
