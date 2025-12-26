use crate::cli::InitArgs;

pub async fn run(args: InitArgs) -> Result<(), String> {
    println!("init: {:?}", args);
    Err("not implemented".to_string())
}
