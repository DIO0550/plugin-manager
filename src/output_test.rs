use super::CommandSummary;

#[test]
fn format_partial_success_uses_warning_prefix() {
    let summary = CommandSummary::format(12, 2);
    assert!(
        summary.prefix.contains('⚠'),
        "partial success should use warning prefix, got {}",
        summary.prefix
    );
    assert!(summary.message.contains("12"));
    assert!(summary.message.contains("2"));
    assert!(summary.message.contains("succeeded"));
    assert!(summary.message.contains("failed"));
}

#[test]
fn format_all_failures_uses_error_prefix() {
    let summary = CommandSummary::format(0, 3);
    assert!(
        summary.prefix.contains('✗'),
        "total failure should use error prefix, got {}",
        summary.prefix
    );
}

#[test]
fn format_all_success_uses_check_prefix() {
    let summary = CommandSummary::format(4, 0);
    assert!(
        summary.prefix.contains('✓'),
        "full success should use check prefix, got {}",
        summary.prefix
    );
    assert!(summary.message.contains("4"));
}

#[test]
fn format_empty_uses_neutral_prefix() {
    let summary = CommandSummary::format(0, 0);
    assert!(summary.prefix.contains('•'));
    assert!(summary.message.contains("No matching components found"));
}
