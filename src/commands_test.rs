use super::{decide_default_action, DefaultAction};

#[test]
fn decide_default_action_returns_launch_managed_when_tty() {
    assert_eq!(decide_default_action(true), DefaultAction::LaunchManaged);
}

#[test]
fn decide_default_action_returns_print_help_when_non_tty() {
    assert_eq!(decide_default_action(false), DefaultAction::PrintHelp);
}
