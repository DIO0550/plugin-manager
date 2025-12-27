use crate::cli::{MarketplaceArgs, MarketplaceCommand};

pub async fn run(args: MarketplaceArgs) -> Result<(), String> {
    match args.command {
        MarketplaceCommand::List => {
            println!("marketplace list: not implemented");
            Ok(())
        }
        MarketplaceCommand::Add { source, name } => {
            println!("marketplace add {source} (name: {name:?}): not implemented");
            Ok(())
        }
        MarketplaceCommand::Remove { name } => {
            println!("marketplace remove {name}: not implemented");
            Ok(())
        }
        MarketplaceCommand::Update { name } => {
            println!("marketplace update {name:?}: not implemented");
            Ok(())
        }
    }
}
