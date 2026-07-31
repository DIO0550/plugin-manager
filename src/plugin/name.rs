//! プラグイン名の値オブジェクト

use std::path::{Component, Path};

/// パスセグメントとして安全なプラグイン名。
///
/// `new` で検証し、不正なら `None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginName<'a>(&'a str);

impl<'a> PluginName<'a> {
    /// 単一パスセグメントとして安全なときだけ `Some` を返す。
    ///
    /// 拒否する条件:
    /// - 空文字
    /// - `/` `\` `\0` を含む
    /// - `.` または `..` 単体
    /// - `Path` として複数コンポーネントになる
    pub fn new(name: &'a str) -> Option<Self> {
        if name.is_empty() {
            return None;
        }
        if name.contains('/') || name.contains('\\') || name.contains('\0') {
            return None;
        }
        if name == "." || name == ".." {
            return None;
        }
        let mut components = Path::new(name).components();
        let first = components.next();
        if components.next().is_some() || !matches!(first, Some(Component::Normal(_))) {
            return None;
        }
        Some(Self(name))
    }

    pub fn as_str(self) -> &'a str {
        self.0
    }
}

#[cfg(test)]
#[path = "name_test.rs"]
mod tests;
