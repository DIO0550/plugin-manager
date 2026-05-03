use super::*;

#[test]
fn test_drive_letter_uppercase() {
    assert!(starts_with_drive_letter("C:foo"));
    assert!(starts_with_drive_letter("C:\\foo"));
    assert!(starts_with_drive_letter("C:/foo"));
}

#[test]
fn test_drive_letter_lowercase() {
    assert!(starts_with_drive_letter("c:foo"));
    assert!(starts_with_drive_letter("a:bar"));
}

#[test]
fn test_drive_letter_only() {
    assert!(starts_with_drive_letter("C:"));
}

#[test]
fn test_not_drive_letter() {
    assert!(!starts_with_drive_letter("foo"));
    assert!(!starts_with_drive_letter("plugins/foo"));
    assert!(!starts_with_drive_letter("./foo"));
    assert!(!starts_with_drive_letter(""));
    assert!(!starts_with_drive_letter("1:foo")); // 数字は対象外
}

#[test]
fn test_short_path() {
    assert!(!starts_with_drive_letter("C"));
    assert!(!starts_with_drive_letter(""));
}

#[test]
fn test_unc_path() {
    assert!(starts_with_unc("//server/share"));
    assert!(starts_with_unc("//server"));
    assert!(starts_with_unc("//"));
}

#[test]
fn test_not_unc_path() {
    assert!(!starts_with_unc("/foo"));
    assert!(!starts_with_unc("foo"));
    assert!(!starts_with_unc(""));
    assert!(!starts_with_unc("C:/foo"));
}
