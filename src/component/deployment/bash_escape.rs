//! Bash スクリプト書き出しと double-quote エスケープのユーティリティ

use crate::error::Result;
use std::fs;
use std::path::Path;

/// スクリプトファイルを書き出し、Unix では実行権限 (0o755) を設定する。
///
/// 親ディレクトリは作成しない（呼び出し側の責任）。`fs::write` が Err を返した
/// 場合はそのまま伝播させる。
///
/// # Arguments
///
/// * `path` - File path to write the script to.
/// * `content` - Script contents to write.
pub(super) fn write_executable_script(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

/// bash ダブルクォート内で特別な意味を持つ文字をエスケープする。
///
/// 対象文字: `\`, `"`, `$`, `` ` ``, `\n`
/// 用途: `@@PLUGIN_ROOT@@` プレースホルダーの置換値として使用
///
/// # Arguments
///
/// * `s` - Raw string to escape for safe interpolation inside bash double quotes.
pub(super) fn escape_for_bash_double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' | '"' | '$' | '`' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => {
                out.push('\\');
                out.push('n');
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
#[path = "bash_escape_test.rs"]
mod tests;
