//! コンポーネント関連の型定義
//!
//! プラグインに含まれるコンポーネントの種類、デプロイ、サマリを定義する。

pub mod convert;
mod deployment;
mod kind;
mod placement;
mod summary;

pub use convert::{CommandFormat, ConversionResult};
pub use deployment::{ComponentDeployment, ComponentDeploymentBuilder, DeploymentResult};
pub use kind::{Component, ComponentKind, Scope};
pub use placement::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
pub use summary::{ComponentName, ComponentTypeCount};
