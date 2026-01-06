//! Windows パス形式の検出ユーティリティ

/// Windows ドライブレターで始まるか（例: "C:", "a:"）
///
/// クロスプラットフォームの一貫性のため、Unix 上でも使用する。
pub fn starts_with_drive_letter(path: &str) -> bool {
    let bytes = path.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
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
}
