//! コンポーネント関連の型定義
//!
//! プラグインに含まれるコンポーネントの種類、デプロイ、サマリを定義する。

pub mod convert;
mod deployment;
mod model;

pub use convert::{AgentFormat, CommandFormat};
pub(crate) use deployment::attached::overlay_attached_resources;
pub use deployment::{
    AttachedResourceWarning, ComponentDeployment, ConversionConfig, DeploymentOutput,
};
pub use model::{
    Component, ComponentKind, ComponentRef, FileOperation, PlacementContext, PlacementLocation,
    PlacementScope, ProjectContext, Scope, ScopedPath,
};
