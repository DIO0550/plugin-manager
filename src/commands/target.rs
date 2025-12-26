use crate::cli::{TargetArgs, TargetCommand};

pub async fn run(args: TargetArgs) -> Result<(), String> {
    match args.command {
        TargetCommand::List => {
            println!("target list: not implemented");
            Ok(())
        }
        TargetCommand::Add { target } => {
            println!("target add {target:?}: not implemented");
            Ok(())
        }
        TargetCommand::Remove { target } => {
            println!("target remove {target:?}: not implemented");
            Ok(())
        }
    }
}
