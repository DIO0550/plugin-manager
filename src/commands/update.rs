use crate::cli::UpdateArgs;

pub async fn run(args: UpdateArgs) -> Result<(), String> {
    println!("update: {:?}", args);
    Err("not implemented".to_string())
}
