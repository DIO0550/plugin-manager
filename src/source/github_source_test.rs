use super::*;
use crate::host::HostKind;
use crate::plugin::meta::{PluginMeta, TargetStatus};
use tempfile::tempdir;

fn sample_repo() -> Repo {
    Repo::new(HostKind::GitHub, "owner", "repo", None)
}

#[test]
fn direct_source_saves_github_metadata() {
    let dir = tempdir().unwrap();
    let source = GitHubSource::new(sample_repo());

    source.save_source_meta(dir.path(), "main", "abc123").unwrap();

    let saved = meta::load_meta(dir.path()).unwrap();
    assert_eq!(saved.marketplace.as_deref(), Some("github"));
    assert!(saved.is_github());
    assert_eq!(saved.source_repo.as_deref(), Some("owner/repo"));
    assert_eq!(saved.git_ref.as_deref(), Some("main"));
    assert_eq!(saved.commit_sha.as_deref(), Some("abc123"));
}

#[test]
fn marketplace_sources_save_marketplace_metadata() {
    // Local と External の両方で、ソースパスの有無によらず取得元を保持する。
    for source_path in [Some("plugins/example".to_string()), None] {
        let dir = tempdir().unwrap();
        let source = GitHubSource::with_marketplace_plugin(
            sample_repo(),
            "example-marketplace".to_string(),
            source_path,
            "example-plugin".to_string(),
        );

        source.save_source_meta(dir.path(), "v1", "def456").unwrap();

        let saved = meta::load_meta(dir.path()).unwrap();
        assert_eq!(saved.marketplace.as_deref(), Some("example-marketplace"));
        assert!(!saved.is_github());
        assert_eq!(saved.source_repo.as_deref(), Some("owner/repo"));
        assert_eq!(saved.git_ref.as_deref(), Some("v1"));
        assert_eq!(saved.commit_sha.as_deref(), Some("def456"));
    }
}

#[test]
fn saving_source_preserves_existing_install_metadata() {
    let dir = tempdir().unwrap();
    let mut existing = PluginMeta {
        installed_at: Some("2026-01-01T00:00:00Z".to_string()),
        marketplace: Some("github".to_string()),
        ..Default::default()
    };
    existing.set_status("codex", TargetStatus::Disabled);
    existing.add_managed_file("codex", &dir.path().join("hooks.json"));
    meta::write_meta(dir.path(), &existing).unwrap();
    let source = GitHubSource::with_marketplace_plugin(
        sample_repo(),
        "example-marketplace".to_string(),
        None,
        "example-plugin".to_string(),
    );

    source.save_source_meta(dir.path(), "main", "abc123").unwrap();

    let saved = meta::load_meta(dir.path()).unwrap();
    assert_eq!(saved.installed_at, existing.installed_at);
    assert_eq!(saved.status_by_target, existing.status_by_target);
    assert_eq!(saved.managed_files, existing.managed_files);
    assert_eq!(saved.marketplace.as_deref(), Some("example-marketplace"));
    assert!(!saved.is_github());
    assert!(saved.updated_at.is_some());
}
