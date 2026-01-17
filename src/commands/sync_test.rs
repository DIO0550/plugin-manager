use super::*;

#[test]
fn test_args_parsing() {
    use clap::CommandFactory;
    let cmd = Args::command();
    cmd.debug_assert();
}
