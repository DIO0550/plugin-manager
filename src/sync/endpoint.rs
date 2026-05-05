//! Sync endpoint sub-parent.
//!
//! Hosts the shared `TargetBinding` (common fields/methods of source/destination)
//! and the crate-internal `Endpoint` enum used for variant-aware dispatch.
//!
//! 形式 A（`#[path = "*_test.rs"] mod tests;` を leaf 末尾に置く）に加えて、
//! 共通テスト集約用の `endpoint_test.rs` を本ファイル末尾でも宣言する。

mod binding;
mod destination;
mod source;

use self::binding::TargetBinding;
pub use self::destination::SyncDestination;
pub use self::source::SyncSource;

use std::path::{Path, PathBuf};

use crate::component::CommandFormat;
use crate::error::{PlmError, Result};
use crate::scan::is_instruction_file;
use crate::sync::model::{PlacedComponent, SyncOptions};
use crate::target::PluginOrigin;

/// Crate-internal abstraction over a sync endpoint variant.
///
/// External callers must continue to use `SyncSource` / `SyncDestination`.
/// `Endpoint` exists to deduplicate dispatch logic inside the `sync` feature.
#[derive(Debug)]
pub(crate) enum Endpoint {
    Source(SyncSource),
    Destination(SyncDestination),
}

impl Endpoint {
    /// `Source` variant ならその `&SyncSource` を返す
    pub(crate) fn as_source(&self) -> Option<&SyncSource> {
        match self {
            Self::Source(s) => Some(s),
            Self::Destination(_) => None,
        }
    }

    /// `Destination` variant ならその `&SyncDestination` を返す
    pub(crate) fn as_destination(&self) -> Option<&SyncDestination> {
        match self {
            Self::Source(_) => None,
            Self::Destination(d) => Some(d),
        }
    }

    /// variant に内包された `TargetBinding` への参照を返す集約点。
    pub(crate) fn binding(&self) -> &TargetBinding {
        match self {
            Self::Source(s) => s.binding(),
            Self::Destination(d) => d.binding(),
        }
    }

    /// ターゲット名（`binding()` 経由で `TargetBinding` に集約）
    pub(crate) fn name(&self) -> &'static str {
        self.binding().name()
    }

    /// Command フォーマット（`binding()` 経由で `TargetBinding` に集約）
    pub(crate) fn command_format(&self) -> CommandFormat {
        self.binding().command_format()
    }

    /// 配置済みコンポーネント一覧（`binding()` 経由で `TargetBinding` に集約）
    pub(crate) fn placed_components(&self, options: &SyncOptions) -> Result<Vec<PlacedComponent>> {
        self.binding().placed_components(options)
    }

    /// 配置済みコンポーネントのパスを解決（`binding()` 経由で `TargetBinding` に集約）
    pub(crate) fn path_for(&self, component: &PlacedComponent) -> Result<PathBuf> {
        self.binding().path_for(component)
    }
}

/// コンポーネント名をパース。
///
/// フラット化後の識別子は `flattened_name` 単一セグメントのみ。Instruction
/// (AGENTS.md / copilot-instructions.md / GEMINI.md) はそのままファイル名で
/// 受け取る特例とする。空文字・パス区切り (`/` `\`) ・null バイト・`.` / `..` ・
/// 複合パスを含むレガシー識別子は `InvalidArgument` で拒否する
/// (`placement_location` が `base.join(name)` を直接行うためベースディレクトリ
/// 外への書き込みを防ぐ)。
///
/// `endpoint` サブ親直下に配置することで、`source` / `destination` 両 leaf から
/// `super::parse_component_name` で参照できる。
pub(super) fn parse_component_name(name: &str) -> Result<(PluginOrigin, &str)> {
    if is_instruction_file(name) {
        return Ok((PluginOrigin::from_marketplace("", ""), name));
    }

    validate_flattened_name(name)?;

    Ok((PluginOrigin::placeholder(), name))
}

/// `flattened_name` が単一の安全なパスセグメントであることを検証する。
fn validate_flattened_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(PlmError::InvalidArgument(
            "Component name must not be empty".to_string(),
        ));
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(PlmError::InvalidArgument(format!(
            "Invalid component name '{}': must not contain path separators or null bytes",
            name
        )));
    }
    if name == "." || name == ".." {
        return Err(PlmError::InvalidArgument(format!(
            "Invalid component name '{}': must not be a parent/current directory reference",
            name
        )));
    }
    let mut components = Path::new(name).components();
    let first = components.next();
    if components.next().is_some() || !matches!(first, Some(std::path::Component::Normal(_))) {
        return Err(PlmError::InvalidArgument(format!(
            "Invalid component name '{}': must be a single flattened segment",
            name
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "endpoint/endpoint_test.rs"]
mod tests;
