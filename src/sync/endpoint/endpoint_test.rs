//! 共通テスト集約: `Endpoint` ディスパッチ + `TargetBinding` 経由の共通振る舞い
//!
//! 実環境（ホームディレクトリ・プロジェクト直下 `AGENTS.md`）に依存しないよう
//! `FakeTarget` を内部定義し `with_target` 経由で注入する。

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use super::{Endpoint, SyncDestination, SyncSource};
use crate::component::{
    AgentFormat, CommandFormat, ComponentKind, PlacementContext, PlacementLocation, Scope,
};
use crate::error::Result;
use crate::sync::model::{PlacedRef, SyncOptions, SyncableKind};
use crate::target::{Target, TargetKind};

/// 実環境非依存の Target 実装。
///
/// `list_placed` / `placement_location` / `supports_scope` の戻り値を
/// フィールドで制御し、テストの再現性を確保する。
struct FakeTarget {
    name: &'static str,
    display_name: &'static str,
    kind: TargetKind,
    supported_components: Vec<ComponentKind>,
    /// (kind, scope) ごとに `list_placed` が返す名前列
    placed: Vec<(ComponentKind, Scope, Vec<String>)>,
    /// `placement_location` が返す固定パス（`None` ならサポート外）
    location: Option<PathBuf>,
    /// `supports_scope` で対応する (kind, scope) の組
    supports_scopes: Vec<(ComponentKind, Scope)>,
    /// `placement_location` の呼出履歴（`name` を観測する）
    placement_calls: RefCell<Vec<String>>,
}

// `Target: Send + Sync` を満たすために手動で `Sync` を実装する必要がある。
// `RefCell` は `!Sync` だが、本テストはシングルスレッドで参照するのみのため
// `unsafe impl Sync` で許可する。
unsafe impl Sync for FakeTarget {}

impl Default for FakeTarget {
    fn default() -> Self {
        Self {
            name: "fake",
            display_name: "Fake",
            kind: TargetKind::Codex,
            supported_components: Vec::new(),
            placed: Vec::new(),
            location: None,
            supports_scopes: Vec::new(),
            placement_calls: RefCell::new(Vec::new()),
        }
    }
}

impl Target for FakeTarget {
    fn name(&self) -> &'static str {
        self.name
    }

    fn display_name(&self) -> &'static str {
        self.display_name
    }

    fn kind(&self) -> TargetKind {
        self.kind
    }

    fn command_format(&self) -> CommandFormat {
        CommandFormat::Codex
    }

    fn agent_format(&self) -> AgentFormat {
        AgentFormat::Codex
    }

    fn supported_components(&self) -> &[ComponentKind] {
        &self.supported_components
    }

    fn supports_scope(&self, kind: ComponentKind, scope: Scope) -> bool {
        self.supports_scopes.contains(&(kind, scope))
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        self.placement_calls
            .borrow_mut()
            .push(context.component.name.to_string());
        self.location.clone().map(PlacementLocation::file)
    }

    fn list_placed(
        &self,
        kind: ComponentKind,
        scope: Scope,
        _project_root: &Path,
    ) -> Result<Vec<String>> {
        Ok(self
            .placed
            .iter()
            .find(|(k, s, _)| *k == kind && *s == scope)
            .map(|(_, _, names)| names.clone())
            .unwrap_or_default())
    }
}

fn fake_source(target: FakeTarget, root: &Path) -> SyncSource {
    SyncSource::with_target(Box::new(target), root)
}

fn fake_destination(target: FakeTarget, root: &Path) -> SyncDestination {
    SyncDestination::with_target(Box::new(target), root)
}

// --- Endpoint dispatch ---

#[test]
fn test_endpoint_source_dispatch_name() {
    let src = fake_source(
        FakeTarget {
            name: "fake-src",
            ..Default::default()
        },
        Path::new("."),
    );
    let ep = Endpoint::Source(src);
    assert_eq!(ep.name(), "fake-src");
    assert!(ep.as_source().is_some());
    assert!(ep.as_destination().is_none());
}

#[test]
fn test_endpoint_destination_dispatch_name() {
    let dst = fake_destination(
        FakeTarget {
            name: "fake-dst",
            ..Default::default()
        },
        Path::new("."),
    );
    let ep = Endpoint::Destination(dst);
    assert_eq!(ep.name(), "fake-dst");
    assert!(ep.as_destination().is_some());
    assert!(ep.as_source().is_none());
}

#[test]
fn test_endpoint_dispatch_command_format() {
    let src = fake_source(FakeTarget::default(), Path::new("."));
    let ep = Endpoint::Source(src);
    assert_eq!(ep.command_format(), CommandFormat::Codex);
}

// --- placed_components / path_for / resolve_path 経由 ---

#[test]
fn test_placed_components_empty_returns_empty_vec() {
    let src = fake_source(FakeTarget::default(), Path::new("/tmp/fake"));
    let v = src.placed_components(&SyncOptions::default()).unwrap();
    assert!(v.is_empty());
}

#[test]
fn test_placed_components_maps_names_to_placed_components() {
    let target = FakeTarget {
        placed: vec![(
            ComponentKind::Skill,
            Scope::Project,
            vec!["alpha".to_string()],
        )],
        location: Some(PathBuf::from("/tmp/fake/alpha")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let opts = SyncOptions::default()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Project);
    let comps = src.placed_components(&opts).unwrap();
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].name(), "alpha");
    assert_eq!(comps[0].kind(), ComponentKind::Skill);
    assert_eq!(comps[0].scope(), Scope::Project);
    assert_eq!(comps[0].path, PathBuf::from("/tmp/fake/alpha"));
}

#[test]
fn test_placed_components_duplicate_ref_returns_error() {
    let target = FakeTarget {
        placed: vec![(
            ComponentKind::Skill,
            Scope::Project,
            vec!["dup".to_string(), "dup".to_string()],
        )],
        location: Some(PathBuf::from("/tmp/fake/dup")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let opts = SyncOptions::default()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Project);
    let err = src.placed_components(&opts).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("Duplicate placed component ref"),
        "expected duplicate error message, got: {msg}"
    );
}

#[test]
fn test_path_for_uses_placement_location() {
    use crate::sync::model::PlacedComponent;

    let target = FakeTarget {
        location: Some(PathBuf::from("/tmp/fake/skills/x")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let comp = PlacedComponent::new(
        ComponentKind::Skill,
        "x",
        Scope::Project,
        PathBuf::from("/ignored"),
    );
    let path = src.path_for(&comp).unwrap();
    assert_eq!(path, PathBuf::from("/tmp/fake/skills/x"));
}

#[test]
fn test_resolve_path_unsupported_returns_error() {
    use crate::sync::model::PlacedComponent;

    let target = FakeTarget {
        location: None, // placement_location が None → サポート外
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let comp = PlacedComponent::new(
        ComponentKind::Skill,
        "y",
        Scope::Project,
        PathBuf::from("/ignored"),
    );
    let err = src.path_for(&comp).unwrap_err();
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("Cannot resolve path"),
        "expected resolve error, got: {msg}"
    );
}

#[test]
fn test_resolve_path_passes_instruction_name_through() {
    use crate::sync::model::PlacedComponent;

    let target = FakeTarget {
        location: Some(PathBuf::from("/tmp/fake/AGENTS.md")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let comp = PlacedComponent::new(
        ComponentKind::Instruction,
        "AGENTS.md",
        Scope::Project,
        PathBuf::from("/ignored"),
    );
    // Instruction 名は parse_component_name の特例で素通りする → エラーにならない
    let path = src.path_for(&comp).unwrap();
    assert_eq!(path, PathBuf::from("/tmp/fake/AGENTS.md"));
}

#[test]
fn test_resolve_path_rejects_invalid_name() {
    use crate::sync::model::PlacedComponent;

    let target = FakeTarget {
        location: Some(PathBuf::from("/tmp/fake/x")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let comp = PlacedComponent::new(
        ComponentKind::Skill,
        "a/b",
        Scope::Project,
        PathBuf::from("/ignored"),
    );
    // parse_component_name がスラッシュを拒否してエラー
    assert!(src.path_for(&comp).is_err());
}

// --- target_kinds / target_scopes フィルタ ---
//
// `target_kinds` / `target_scopes` は `TargetBinding` の private なメソッドだが、
// `placed_components` 経由で間接的に観測する。`list_placed` を呼び出すたびに
// FakeTarget の `placement_calls` 履歴が積まれる動作を利用してフィルタ尊重を検証する。

#[test]
fn test_placed_components_respects_component_type_filter() {
    let target = FakeTarget {
        placed: vec![
            (
                ComponentKind::Skill,
                Scope::Personal,
                vec!["s1".to_string()],
            ),
            (
                ComponentKind::Agent,
                Scope::Personal,
                vec!["a1".to_string()],
            ),
        ],
        location: Some(PathBuf::from("/tmp/fake/x")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let opts = SyncOptions::default()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Personal);
    let comps = src.placed_components(&opts).unwrap();
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].kind(), ComponentKind::Skill);
}

#[test]
fn test_placed_components_respects_scope_filter() {
    let target = FakeTarget {
        placed: vec![
            (
                ComponentKind::Skill,
                Scope::Personal,
                vec!["s1".to_string()],
            ),
            (ComponentKind::Skill, Scope::Project, vec!["s2".to_string()]),
        ],
        location: Some(PathBuf::from("/tmp/fake/x")),
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let opts = SyncOptions::default()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Project);
    let comps = src.placed_components(&opts).unwrap();
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].name(), "s2");
    assert_eq!(comps[0].scope(), Scope::Project);
}

// --- SyncDestination::supports ---

#[test]
fn test_supports_returns_true_when_kind_and_scope_supported() {
    let target = FakeTarget {
        supported_components: vec![ComponentKind::Skill],
        supports_scopes: vec![(ComponentKind::Skill, Scope::Project)],
        ..Default::default()
    };
    let dst = fake_destination(target, Path::new("/tmp/fake"));
    let r = PlacedRef::new(ComponentKind::Skill, "x", Scope::Project);
    assert!(dst.supports(&r));
}

#[test]
fn test_supports_returns_false_when_scope_unsupported() {
    let target = FakeTarget {
        supported_components: vec![ComponentKind::Skill],
        supports_scopes: vec![(ComponentKind::Skill, Scope::Personal)],
        ..Default::default()
    };
    let dst = fake_destination(target, Path::new("/tmp/fake"));
    let r = PlacedRef::new(ComponentKind::Skill, "x", Scope::Project);
    assert!(!dst.supports(&r));
}

#[test]
fn test_supports_returns_false_when_kind_unsupported() {
    let target = FakeTarget {
        supported_components: vec![ComponentKind::Agent],
        supports_scopes: vec![(ComponentKind::Skill, Scope::Project)],
        ..Default::default()
    };
    let dst = fake_destination(target, Path::new("/tmp/fake"));
    let r = PlacedRef::new(ComponentKind::Skill, "x", Scope::Project);
    assert!(!dst.supports(&r));
}

// --- Debug 出力フォーマット完全保存 ---

#[test]
fn test_sync_source_debug_format_preserved() {
    let target = FakeTarget {
        name: "fake-src",
        ..Default::default()
    };
    let src = fake_source(target, Path::new("/tmp/fake"));
    let dbg = format!("{:?}", src);
    assert!(
        dbg.starts_with("SyncSource {"),
        "expected SyncSource {{ ... }}, got: {dbg}"
    );
    assert!(dbg.contains("target: \"fake-src\""), "got: {dbg}");
    assert!(dbg.contains("project_root:"), "got: {dbg}");
}

#[test]
fn test_sync_destination_debug_format_preserved() {
    let target = FakeTarget {
        name: "fake-dst",
        ..Default::default()
    };
    let dst = fake_destination(target, Path::new("/tmp/fake"));
    let dbg = format!("{:?}", dst);
    assert!(
        dbg.starts_with("SyncDestination {"),
        "expected SyncDestination {{ ... }}, got: {dbg}"
    );
    assert!(dbg.contains("target: \"fake-dst\""), "got: {dbg}");
    assert!(dbg.contains("project_root:"), "got: {dbg}");
}
