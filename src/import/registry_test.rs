//! ImportRegistry のユニットテスト

use super::*;
use crate::component::ComponentKind;
use crate::target::{Scope, TargetKind};
use std::path::PathBuf;
use tempfile::TempDir;

fn make_test_record(name: &str) -> ImportRecord {
    ImportRecord {
        source_repo: "owner/repo".to_string(),
        kind: ComponentKind::Skill,
        name: name.to_string(),
        target: TargetKind::Codex,
        scope: Scope::Personal,
        path: PathBuf::from(format!("/test/path/{}", name)),
        imported_at: "2024-01-01T00:00:00Z".to_string(),
        git_ref: "main".to_string(),
        commit_sha: "abc123".to_string(),
    }
}

mod import_registry_tests {
    use super::*;

    #[test]
    fn new_registry_with_custom_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let registry = ImportRegistry::with_path(config_path.clone());

        assert_eq!(registry.config_path, config_path);
        assert_eq!(registry.current_state(), "Idle");
    }

    #[test]
    fn load_creates_empty_config_if_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let mut registry = ImportRegistry::with_path(config_path);
        let config = registry.load().unwrap();

        assert!(config.imports.is_empty());
        assert_eq!(registry.current_state(), "Loaded");
    }

    #[test]
    fn load_reads_existing_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        // Create existing config
        let config = ImportsConfig {
            imports: vec![make_test_record("pdf")],
        };
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let mut registry = ImportRegistry::with_path(config_path);
        let loaded = registry.load().unwrap();

        assert_eq!(loaded.imports.len(), 1);
        assert_eq!(loaded.imports[0].name, "pdf");
    }

    #[test]
    fn list_returns_all_records() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let config = ImportsConfig {
            imports: vec![make_test_record("pdf"), make_test_record("json")],
        };
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let mut registry = ImportRegistry::with_path(config_path);
        let records = registry.list().unwrap();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn record_saves_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let mut registry = ImportRegistry::with_path(config_path.clone());
        registry.record(make_test_record("pdf")).unwrap();

        // Verify file was created
        assert!(config_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: ImportsConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.imports.len(), 1);
        assert_eq!(config.imports[0].name, "pdf");
    }

    #[test]
    fn record_appends_to_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let mut registry = ImportRegistry::with_path(config_path.clone());
        registry.record(make_test_record("pdf")).unwrap();
        registry.record(make_test_record("json")).unwrap();

        let records = registry.list().unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn list_by_source_filters_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let mut record1 = make_test_record("pdf");
        record1.source_repo = "owner1/repo1".to_string();

        let mut record2 = make_test_record("json");
        record2.source_repo = "owner2/repo2".to_string();

        let config = ImportsConfig {
            imports: vec![record1, record2],
        };
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let mut registry = ImportRegistry::with_path(config_path);
        let filtered = registry.list_by_source("owner1/repo1").unwrap();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "pdf");
    }

    #[test]
    fn state_transitions_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("imports.json");

        let mut registry = ImportRegistry::with_path(config_path);

        // Initial state
        assert_eq!(registry.current_state(), "Idle");

        // After load
        registry.load().unwrap();
        assert_eq!(registry.current_state(), "Loaded");

        // After record (which saves and returns to Idle)
        registry.record(make_test_record("pdf")).unwrap();
        assert_eq!(registry.current_state(), "Idle");
    }

    #[test]
    fn creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("imports.json");

        let mut registry = ImportRegistry::with_path(nested_path.clone());
        registry.record(make_test_record("pdf")).unwrap();

        assert!(nested_path.exists());
    }
}
