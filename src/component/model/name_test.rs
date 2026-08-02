use super::ComponentName;

#[test]
fn accepts_plain_name() {
    assert_eq!(ComponentName::new("my-skill").unwrap().as_str(), "my-skill");
}

#[test]
fn rejects_empty_dot_and_separators() {
    assert!(ComponentName::new("").is_none());
    assert!(ComponentName::new(".").is_none());
    assert!(ComponentName::new("..").is_none());
    assert!(ComponentName::new("a/b").is_none());
    assert!(ComponentName::new("a\\b").is_none());
    assert!(ComponentName::new("a\0b").is_none());
}
