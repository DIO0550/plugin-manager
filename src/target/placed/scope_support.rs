//! kind × scope の薄いサポート表現（#338）
//!
//! `supported_components` と `can_place_scope` は [`Capabilities`] から導出する。
//! 2 つのスライスを手書きすると乖離するため、構築は [`capabilities!`] マクロに限定する。

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

    pub(crate) fn is_supported(self) -> bool {
        self != Self::None
    }
}

/// kind × scope 表と、そこから導出したサポート種別スライス。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Capabilities {
    table: &'static [(ComponentKind, ScopeSupport)],
    supported: &'static [ComponentKind],
}

impl Capabilities {
    /// [`capabilities!`] 専用。手書きの二重定義を避ける。
    pub(crate) const fn from_macro(
        table: &'static [(ComponentKind, ScopeSupport)],
        supported: &'static [ComponentKind],
    ) -> Self {
        Self { table, supported }
    }

    pub(crate) fn allows(self, kind: ComponentKind, scope: Scope) -> bool {
        allows_scope(self.table, kind, scope)
    }

    pub(crate) fn supported(self) -> &'static [ComponentKind] {
        self.supported
    }

    /// 表から「いずれかのスコープで配置可」な種別を集める（テスト用）。
    pub(crate) fn derived_supported(self) -> Vec<ComponentKind> {
        self.table
            .iter()
            .filter(|(_, support)| support.is_supported())
            .map(|(kind, _)| *kind)
            .collect()
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

/// 単一のエントリ列から `table` と `supported` を同時に生成する。
///
/// `ScopeSupport::None` は書かない。非サポート種別は省略する。
macro_rules! capabilities {
    ($($kind:ident => $support:ident),+ $(,)?) => {
        $crate::target::scope_support::Capabilities::from_macro(
            &[$(
                (
                    $crate::component::ComponentKind::$kind,
                    $crate::target::scope_support::ScopeSupport::$support
                )
            ),+],
            &[$($crate::component::ComponentKind::$kind),+],
        )
    };
}
pub(crate) use capabilities;

#[cfg(test)]
#[path = "scope_support_test.rs"]
mod tests;
