//! ドメインモデル
//!
//! DDDに基づくドメインモデルを定義する。

pub mod placement;

pub use placement::{ComponentRef, PlacementContext, PlacementLocation, PlacementScope, ProjectContext};
