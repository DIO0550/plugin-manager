//! `ComponentDeployment` のスリムな Builder
//!
//! 必須フィールドは `component` / `scope` / `target_path` の 3 個。
//! 変換設定は `ConversionConfig::None` を default とする任意フィールド。

use super::conversion::ConversionConfig;
use super::executor::ComponentDeployment;
use crate::component::{Component, Scope};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ComponentDeploymentBuilder {
    component: Option<Component>,
    scope: Option<Scope>,
    target_path: Option<PathBuf>,
    conversion: ConversionConfig,
}

impl ComponentDeploymentBuilder {
    pub fn component(mut self, component: Component) -> Self {
        self.component = Some(component);
        self
    }

    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    pub fn conversion(mut self, conversion: ConversionConfig) -> Self {
        self.conversion = conversion;
        self
    }

    pub fn build(self) -> Result<ComponentDeployment, String> {
        let component = self
            .component
            .ok_or_else(|| "component is required".to_string())?;
        let scope = self.scope.ok_or_else(|| "scope is required".to_string())?;
        let target_path = self
            .target_path
            .ok_or_else(|| "target_path is required".to_string())?;

        Ok(ComponentDeployment {
            component,
            scope,
            target_path,
            conversion: self.conversion,
        })
    }
}

#[cfg(test)]
#[path = "builder_test.rs"]
mod tests;
