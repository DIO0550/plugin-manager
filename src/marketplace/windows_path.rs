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

/// Windows UNC パスで始まるか（例: "//server/share"）
///
/// 元の形式は `\\server\share` だが、バックスラッシュ正規化後は `//` になる。
pub fn starts_with_unc(path: &str) -> bool {
    path.starts_with("//")
}

#[cfg(test)]
#[path = "windows_path_test.rs"]
mod tests;
