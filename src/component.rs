//! コンポーネント関連の型定義
//!
//! プラグインに含まれるコンポーネントの種類、デプロイ、サマリを定義する。

pub mod convert;
mod deployment;
mod file_operation;
mod identity;
mod kind;
mod placement;
mod scoped_path;

pub use convert::{AgentFormat, CommandFormat};
pub use deployment::{ComponentDeployment, DeploymentResult};
pub use file_operation::FileOperation;
pub use identity::ComponentIdentity;
pub use kind::{Component, ComponentKind, Scope};
pub use placement::{PlacementContext, PlacementLocation, PlacementScope, ProjectContext};
pub use scoped_path::ScopedPath;
