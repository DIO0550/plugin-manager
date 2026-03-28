//! Tests for HookName value object.

use super::name::HookName;

// ========================================
// HookName sanitization tests
// ========================================

#[test]
fn test_safe_chars_unchanged() {
    assert_eq!(HookName::new("my-hook_v1").as_safe(), "my-hook_v1");
}

#[test]
fn test_replaces_unsafe_chars_with_hash() {
    let hook = HookName::new("my hook$`name");
    let result = hook.as_safe();
    assert!(result.starts_with("my-hook--name-"));
    assert!(result.len() > "my-hook--name".len());
}

#[test]
fn test_dots_replaced_with_hash() {
    let hook = HookName::new("my-hook.v1");
    let result = hook.as_safe();
    assert!(result.starts_with("my-hook-v1-"));
}

#[test]
fn test_dotdot_gets_hash() {
    let hook = HookName::new("..");
    let result = hook.as_safe();
    assert!(result.starts_with("_hook-"));
}

#[test]
fn test_empty_gets_hash() {
    let hook = HookName::new("");
    let result = hook.as_safe();
    assert!(result.starts_with("_hook-"));
}

#[test]
fn test_only_special_chars_gets_hash() {
    let hook = HookName::new("$@!");
    let result = hook.as_safe();
    assert!(result.starts_with("_hook-"));
}

#[test]
fn test_dotfile_gets_hash() {
    let hook = HookName::new(".env");
    let result = hook.as_safe();
    assert!(result.starts_with("env-"));
}

#[test]
fn test_leading_trailing_hyphens_trimmed() {
    let hook = HookName::new("-foo-");
    let result = hook.as_safe();
    assert!(result.starts_with("foo-"));
}

#[test]
fn test_different_inputs_produce_different_results() {
    let a = HookName::new("my hook");
    let b = HookName::new("my-hook");
    assert_ne!(a.as_safe(), b.as_safe());
}

#[test]
fn test_raw_returns_original() {
    let hook = HookName::new("my hook$name");
    assert_eq!(hook.raw(), "my hook$name");
}
