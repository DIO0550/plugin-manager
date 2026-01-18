//! コンポーネント関連の型定義
//!
//! プラグインに含まれるコンポーネントの種類、デプロイ、サマリを定義する。

mod deployment;
mod kind;
mod placement;
mod summary;

pub use deployment::{ComponentDeployment, ComponentDeploymentBuilder};
pub use kind::{Component, ComponentKind, Scope};
pub use placement::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
pub use summary::{ComponentName, ComponentTypeCount};
