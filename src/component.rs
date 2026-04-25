//! コンポーネント関連の型定義
//!
//! プラグインに含まれるコンポーネントの種類、デプロイ、サマリを定義する。

pub mod convert;
mod deployment;
mod file_operation;
mod kind;
mod placement;
mod scoped_path;

pub use convert::{AgentFormat, CommandFormat};
#[allow(unused_imports)]
pub use deployment::ComponentDeploymentBuilder;
pub use deployment::{ComponentDeployment, ConversionConfig, DeploymentOutput};
pub use file_operation::FileOperation;
pub use kind::{Component, ComponentKind, Scope};
pub use placement::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
pub use scoped_path::ScopedPath;
