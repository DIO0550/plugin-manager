use crate::cli::ImportArgs;

pub async fn run(args: ImportArgs) -> Result<(), String> {
    println!("import: {:?}", args);
    Err("not implemented".to_string())
}
