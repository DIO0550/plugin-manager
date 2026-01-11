use super::*;

#[test]
fn test_target_effect() {
    let effect = TargetEffect::new("codex", 3);
    assert_eq!(effect.target_name(), "codex");
    assert_eq!(effect.component_count(), 3);
}

#[test]
fn test_target_error() {
    let error = TargetError::new("copilot", "Permission denied");
    assert_eq!(error.target_name(), "copilot");
    assert_eq!(error.message(), "Permission denied");
}

#[test]
fn test_affected_targets_success() {
    let mut affected = AffectedTargets::new();
    affected.record_success("codex", 2);
    affected.record_success("copilot", 3);

    assert_eq!(affected.total_components(), 5);
    assert_eq!(affected.target_names(), vec!["codex", "copilot"]);
    assert!(!affected.has_errors());

    let result = affected.into_result();
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn test_affected_targets_with_errors() {
    let mut affected = AffectedTargets::new();
    affected.record_success("codex", 2);
    affected.record_error("copilot", "Failed");

    assert!(affected.has_errors());

    let result = affected.into_result();
    assert!(!result.success);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("copilot: Failed"));
}

#[test]
fn test_affected_targets_zero_components_not_recorded() {
    let mut affected = AffectedTargets::new();
    affected.record_success("codex", 0);
    affected.record_success("copilot", 3);

    assert_eq!(affected.target_names(), vec!["copilot"]);
}

#[test]
fn test_operation_result_error() {
    let result = OperationResult::error("Plugin not found");
    assert!(!result.success);
    assert_eq!(result.error, Some("Plugin not found".to_string()));
    assert!(result.affected_targets.target_names().is_empty());
}
