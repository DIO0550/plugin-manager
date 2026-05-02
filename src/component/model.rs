//! コンポーネントモデル（値オブジェクト群）
//!
//! `kind`: ComponentKind / Component / Scope
//! `placement`: ComponentRef / PlacementContext / PlacementLocation
//! `scoped_path`: ScopedPath
//! `file_operation`: FileOperation

mod file_operation;
mod kind;
mod placement;
mod scoped_path;

pub use file_operation::FileOperation;
pub use kind::{Component, ComponentKind, Scope};
pub use placement::{
    ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext,
};
pub use scoped_path::ScopedPath;
