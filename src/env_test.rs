use super::*;

#[test]
fn test_get_existing_var() {
    std::env::set_var("TEST_ENV_VAR", "test_value");
    assert_eq!(EnvVar::get("TEST_ENV_VAR"), Some("test_value".to_string()));
    std::env::remove_var("TEST_ENV_VAR");
}

#[test]
fn test_get_empty_var() {
    std::env::set_var("TEST_EMPTY_VAR", "");
    assert_eq!(EnvVar::get("TEST_EMPTY_VAR"), None);
    std::env::remove_var("TEST_EMPTY_VAR");
}

#[test]
fn test_get_nonexistent_var() {
    assert_eq!(EnvVar::get("NONEXISTENT_VAR_12345"), None);
}
