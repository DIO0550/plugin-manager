//! コンポーネント識別子
//!
//! コンポーネントを一意に指し示す値オブジェクト。

use crate::component::{ComponentKind, Scope};
use crate::target::PluginOrigin;

/// コンポーネントを一意に指し示す値オブジェクト
///
/// - 必須: `kind`, `name`
/// - オプション: `scope`（配置スコープが確定している文脈でのみ `Some`）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentIdentity {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Option<Scope>,
}

impl ComponentIdentity {
    /// Create a new `ComponentIdentity` without a scope.
    pub fn new(kind: ComponentKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            scope: None,
        }
    }

    /// Attach a scope to this identity.
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// `{marketplace}/{plugin}/{name}` 形式の完全修飾名を返す
    pub fn qualified_name(&self, origin: &PluginOrigin) -> String {
        format!("{}/{}/{}", origin.marketplace, origin.plugin, self.name)
    }
}

#[cfg(test)]
#[path = "identity_test.rs"]
mod tests;
