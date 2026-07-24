//! kind × scope の薄いサポート表現（Phase F）

use crate::component::{ComponentKind, Scope};

/// コンポーネント種別がどのスコープで配置可能か。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeSupport {
    None,
    PersonalOnly,
    ProjectOnly,
    Both,
}

impl ScopeSupport {
    pub(crate) fn allows(self, scope: Scope) -> bool {
        match self {
            Self::None => false,
            Self::PersonalOnly => scope == Scope::Personal,
            Self::ProjectOnly => scope == Scope::Project,
            Self::Both => true,
        }
    }
}

/// `(kind, ScopeSupport)` 表からスコープ可否を判定する。
pub(crate) fn allows_scope(
    table: &[(ComponentKind, ScopeSupport)],
    kind: ComponentKind,
    scope: Scope,
) -> bool {
    table
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, support)| support.allows(scope))
        .unwrap_or(false)
}
