use super::*;
use clap::Parser;

#[test]
fn test_args_parsing_defaults() {
    let args = Args::parse_from(["uninstall", "my-plugin"]);
    assert_eq!(args.name, "my-plugin");
    assert_eq!(args.marketplace, None);
    assert!(!args.force);
}

#[test]
fn test_args_parsing_with_marketplace() {
    let args = Args::parse_from(["uninstall", "my-plugin", "--marketplace", "custom"]);
    assert_eq!(args.name, "my-plugin");
    assert_eq!(args.marketplace, Some("custom".to_string()));
    assert!(!args.force);
}

#[test]
fn test_args_parsing_with_force() {
    let args = Args::parse_from(["uninstall", "my-plugin", "--force"]);
    assert_eq!(args.name, "my-plugin");
    assert!(args.force);
}

#[test]
fn test_args_parsing_with_short_options() {
    let args = Args::parse_from(["uninstall", "my-plugin", "-m", "custom", "-f"]);
    assert_eq!(args.name, "my-plugin");
    assert_eq!(args.marketplace, Some("custom".to_string()));
    assert!(args.force);
}

#[test]
fn test_args_parsing_with_all_options() {
    let args = Args::parse_from([
        "uninstall",
        "my-plugin",
        "--marketplace",
        "custom",
        "--force",
    ]);
    assert_eq!(args.name, "my-plugin");
    assert_eq!(args.marketplace, Some("custom".to_string()));
    assert!(args.force);
}
