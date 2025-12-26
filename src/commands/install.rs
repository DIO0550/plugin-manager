use crate::cli::InstallArgs;

pub async fn run(args: InstallArgs) -> Result<(), String> {
    println!("install: {:?}", args);
    Err("not implemented".to_string())
}
