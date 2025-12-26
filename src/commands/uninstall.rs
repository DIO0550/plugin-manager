use crate::cli::UninstallArgs;

pub async fn run(args: UninstallArgs) -> Result<(), String> {
    println!("uninstall: {:?}", args);
    Err("not implemented".to_string())
}
