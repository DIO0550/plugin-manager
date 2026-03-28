//! HookName value object for safe path-segment hook names.

use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Hook 名をパスセグメントとして安全な文字列にサニタイズする値オブジェクト
///
/// - `[A-Za-z0-9_-]` 以外をハイフンに置換（`.` も含めて `-` に置換）
/// - 先頭・末尾のハイフンを除去
/// - サニタイズ後の結果が空文字列の場合はフォールバック名として `_hook` をベースに使用
/// - サニタイズにより元名と異なる場合は短いハッシュサフィックスを付加して衝突を防止
#[derive(Debug, Clone)]
pub(crate) struct HookName {
    raw: String,
    safe: String,
}

impl HookName {
    /// 生の名前からサニタイズ済み HookName を生成
    pub fn new(raw: &str) -> Self {
        let sanitized: String = raw
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let trimmed = sanitized.trim_matches('-');
        let base = if trimmed.is_empty() { "_hook" } else { trimmed };

        let safe = if base == raw {
            base.to_string()
        } else {
            let mut hasher = DefaultHasher::new();
            raw.hash(&mut hasher);
            let hash = hasher.finish();
            let mut suffix = String::with_capacity(17);
            let _ = write!(suffix, "-{:016x}", hash);
            format!("{}{}", base, suffix)
        };

        Self {
            raw: raw.to_string(),
            safe,
        }
    }

    /// サニタイズ済みの安全な名前を返す
    pub fn as_safe(&self) -> &str {
        &self.safe
    }

    /// 元の名前を返す
    pub fn raw(&self) -> &str {
        &self.raw
    }
}
