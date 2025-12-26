use crate::cli::SyncArgs;

pub async fn run(args: SyncArgs) -> Result<(), String> {
    println!("sync: {:?}", args);
    Err("not implemented".to_string())
}
