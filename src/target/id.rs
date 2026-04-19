//! ターゲット識別子（値オブジェクト）

/// ターゲット識別子（型安全）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(String);

impl TargetId {
    /// Create a new target identifier.
    ///
    /// # Arguments
    /// * `name` - Target name (for example, `"codex"` or `"copilot"`).
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TargetId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "id_test.rs"]
mod tests;
