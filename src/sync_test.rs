use super::*;
use crate::component::{
    AgentFormat, CommandFormat, ComponentKind, PlacementContext, PlacementLocation, Scope,
};
use crate::fs::mock::MockFs;
use crate::target::{Target, TargetKind};
use std::path::{Path, PathBuf};

struct FakeTarget {
    kind: TargetKind,
    supported_components: Vec<ComponentKind>,
    supported_scopes: Vec<(ComponentKind, Scope)>,
    placed: Vec<(ComponentKind, Scope, Vec<String>)>,
    base_path: PathBuf,
}

impl Default for FakeTarget {
    fn default() -> Self {
        Self {
            kind: TargetKind::Codex,
            supported_components: vec![ComponentKind::Skill],
            supported_scopes: vec![(ComponentKind::Skill, Scope::Project)],
            placed: Vec::new(),
            base_path: PathBuf::from("/tmp/fake"),
        }
    }
}

impl FakeTarget {
    fn with_placed(names: Vec<&str>) -> Self {
        Self {
            placed: vec![(
                ComponentKind::Skill,
                Scope::Project,
                names.into_iter().map(str::to_string).collect(),
            )],
            ..Default::default()
        }
    }

    fn without_skill_support() -> Self {
        Self {
            supported_components: Vec::new(),
            supported_scopes: Vec::new(),
            ..Default::default()
        }
    }
}

impl Target for FakeTarget {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn display_name(&self) -> &'static str {
        "Fake"
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
        self.supported_scopes.contains(&(kind, scope))
    }

    fn placement_location(&self, context: &PlacementContext) -> Option<PlacementLocation> {
        Some(PlacementLocation::file(
            self.base_path.join(&context.component.name),
        ))
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
            .find(|(placed_kind, placed_scope, _)| *placed_kind == kind && *placed_scope == scope)
            .map(|(_, _, names)| names.clone())
            .unwrap_or_default())
    }
}

fn fake_source(target: FakeTarget) -> SyncSource {
    SyncSource::with_target(Box::new(target), Path::new("/tmp/source"))
}

fn fake_destination(target: FakeTarget) -> SyncDestination {
    SyncDestination::with_target(Box::new(target), Path::new("/tmp/dest"))
}

fn skill_project_options() -> SyncOptions {
    SyncOptions::default()
        .with_component_type(SyncableKind::Skill)
        .with_scope(Scope::Project)
}

#[test]
fn test_needs_update_newer_source() {
    let fs = MockFs::new();
    fs.add_file("/src/test.md", "new content");
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs.add_file("/dst/test.md", "old content");

    // Note: MockFs の mtime は追加時刻なので、src の方が古い
    // 実際のテストでは内容ハッシュで判定される
    let src = PlacedComponent::new(
        ComponentKind::Skill,
        "test",
        Scope::Personal,
        "/src/test.md",
    );
    let dst = PlacedComponent::new(
        ComponentKind::Skill,
        "test",
        Scope::Personal,
        "/dst/test.md",
    );

    // 内容が違うので更新が必要
    let result = needs_update(&src, &dst, &fs).unwrap();
    assert!(result);
}

#[test]
fn test_needs_update_same_content() {
    let fs = MockFs::new();
    fs.add_file("/src/test.md", "same content");
    fs.add_file("/dst/test.md", "same content");

    let src = PlacedComponent::new(
        ComponentKind::Skill,
        "test",
        Scope::Personal,
        "/src/test.md",
    );
    let dst = PlacedComponent::new(
        ComponentKind::Skill,
        "test",
        Scope::Personal,
        "/dst/test.md",
    );

    let result = needs_update(&src, &dst, &fs).unwrap();
    assert!(!result);
}

#[test]
fn test_collect_components_uses_endpoint_dispatch_for_source() {
    let src = fake_source(FakeTarget::with_placed(vec!["alpha"]));
    let options = skill_project_options();

    let components = collect_components(endpoint::Endpoint::Source(&src), &options).unwrap();

    assert_eq!(components.len(), 1);
    assert_eq!(components[0].name(), "alpha");
    assert_eq!(components[0].kind(), ComponentKind::Skill);
    assert_eq!(components[0].scope(), Scope::Project);
}

#[test]
fn test_collect_components_uses_endpoint_dispatch_for_destination() {
    let dest = fake_destination(FakeTarget::with_placed(vec!["alpha"]));
    let options = skill_project_options();

    let components = collect_components(endpoint::Endpoint::Destination(&dest), &options).unwrap();

    assert_eq!(components.len(), 1);
    assert_eq!(components[0].name(), "alpha");
    assert_eq!(components[0].kind(), ComponentKind::Skill);
    assert_eq!(components[0].scope(), Scope::Project);
}

#[test]
fn test_collect_components_propagates_source_scan_error() {
    let src = fake_source(FakeTarget::with_placed(vec!["nested/alpha"]));
    let options = skill_project_options();

    let err = collect_components(endpoint::Endpoint::Source(&src), &options).unwrap_err();

    assert!(
        err.to_string().contains("Invalid component name"),
        "expected invalid component name error, got: {err:?}"
    );
}

#[test]
fn test_collect_components_propagates_destination_scan_error() {
    let dest = fake_destination(FakeTarget::with_placed(vec!["nested/alpha"]));
    let options = skill_project_options();

    let err = collect_components(endpoint::Endpoint::Destination(&dest), &options).unwrap_err();

    assert!(
        err.to_string().contains("Invalid component name"),
        "expected invalid component name error, got: {err:?}"
    );
}

#[test]
fn test_sync_with_fs_propagates_source_scan_error_without_sync_failure() {
    let source = fake_source(FakeTarget::with_placed(vec!["nested/alpha"]));
    let dest = fake_destination(FakeTarget::default());
    let fs = MockFs::new();

    let err = sync_with_fs(&source, &dest, &skill_project_options(), &fs).unwrap_err();

    assert!(
        err.to_string().contains("Invalid component name"),
        "expected invalid component name error, got: {err:?}"
    );
}

#[test]
fn test_sync_with_fs_propagates_destination_scan_error_without_sync_failure() {
    let source = fake_source(FakeTarget::default());
    let dest = fake_destination(FakeTarget::with_placed(vec!["nested/alpha"]));
    let fs = MockFs::new();

    let err = sync_with_fs(&source, &dest, &skill_project_options(), &fs).unwrap_err();

    assert!(
        err.to_string().contains("Invalid component name"),
        "expected invalid component name error, got: {err:?}"
    );
}

#[test]
fn test_sync_with_fs_uses_destination_supports_for_unsupported_dry_run() {
    let source = fake_source(FakeTarget::with_placed(vec!["alpha"]));
    let dest = fake_destination(FakeTarget::without_skill_support());
    let fs = MockFs::new();

    let result = sync_with_fs(
        &source,
        &dest,
        &SyncOptions {
            dry_run: true,
            ..skill_project_options()
        },
        &fs,
    )
    .unwrap();

    assert!(result.dry_run);
    assert_eq!(result.unsupported.len(), 1);
    assert_eq!(result.unsupported[0].name(), "alpha");
    assert!(result.created.is_empty());
    assert!(result.failed.is_empty());
}
