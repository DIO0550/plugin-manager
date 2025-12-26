use crate::cli::ListArgs;

pub async fn run(args: ListArgs) -> Result<(), String> {
    println!("list: {:?}", args);
    Err("not implemented".to_string())
}
