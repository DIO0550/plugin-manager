use super::*;
use std::fs;
use tempfile::TempDir;

// ========================================
// escape_for_bash_double_quote tests
// ========================================

#[test]
fn test_escape_for_bash_double_quote_plain_ascii() {
    let input = "/home/user/plugins";
    assert_eq!(escape_for_bash_double_quote(input), "/home/user/plugins");
}

#[test]
fn test_escape_for_bash_double_quote_backslash() {
    assert_eq!(
        escape_for_bash_double_quote(r"C:\Users\test"),
        r"C:\\Users\\test"
    );
}

#[test]
fn test_escape_for_bash_double_quote_double_quote() {
    assert_eq!(
        escape_for_bash_double_quote(r#"say "hello""#),
        r#"say \"hello\""#
    );
}

#[test]
fn test_escape_for_bash_double_quote_dollar_sign() {
    assert_eq!(escape_for_bash_double_quote("$HOME/path"), r"\$HOME/path");
}

#[test]
fn test_escape_for_bash_double_quote_backtick() {
    assert_eq!(escape_for_bash_double_quote("`cmd`"), r"\`cmd\`");
}

#[test]
fn test_escape_for_bash_double_quote_newline() {
    assert_eq!(
        escape_for_bash_double_quote("line1\nline2"),
        r"line1\nline2"
    );
}

#[test]
fn test_escape_for_bash_double_quote_empty_string() {
    assert_eq!(escape_for_bash_double_quote(""), "");
}

#[test]
fn test_escape_for_bash_double_quote_all_special_chars() {
    assert_eq!(
        escape_for_bash_double_quote("a\\b\"c$d`e\nf"),
        r#"a\\b\"c\$d\`e\nf"#
    );
}

// ========================================
// write_executable_script tests
// ========================================

#[test]
fn test_write_executable_script_creates_file_with_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sh");
    let content = "#!/bin/bash\necho hello";

    write_executable_script(&path, content).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), content);
}

#[cfg(unix)]
#[test]
fn test_write_executable_script_sets_executable_permission() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.sh");

    write_executable_script(&path, "#!/bin/bash").unwrap();

    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn test_write_executable_script_nonexistent_dir_returns_err() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("no_such_subdir").join("test.sh");
    assert!(write_executable_script(&path, "content").is_err());
}
