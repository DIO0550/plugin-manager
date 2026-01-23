#[cfg(test)]
mod tests {
    use crate::marketplace::{
        normalize_name, normalize_source_path, to_display_source, to_internal_source,
        validate_name, MarketplaceConfig, MarketplaceEntry,
    };
    use tempfile::TempDir;

    // ==================== normalize_name tests ====================

    #[test]
    fn normalize_name_converts_uppercase_to_lowercase() {
        let result = normalize_name("My-PLUGIN").unwrap();
        assert_eq!(result, "my-plugin");
    }

    #[test]
    fn normalize_name_allows_periods() {
        let result = normalize_name("my.plugin.name").unwrap();
        assert_eq!(result, "my.plugin.name");
    }

    #[test]
    fn normalize_name_allows_hyphens_and_underscores() {
        let result = normalize_name("my-plugin_name").unwrap();
        assert_eq!(result, "my-plugin_name");
    }

    #[test]
    fn normalize_name_allows_numbers() {
        let result = normalize_name("plugin123").unwrap();
        assert_eq!(result, "plugin123");
    }

    // ==================== validate_name tests ====================

    #[test]
    fn validate_name_accepts_valid_name() {
        assert!(validate_name("my-plugin").is_ok());
        assert!(validate_name("plugin.v1").is_ok());
        assert!(validate_name("a_b_c").is_ok());
    }

    #[test]
    fn validate_name_rejects_invalid_characters() {
        assert!(validate_name("my plugin").is_err()); // space
        assert!(validate_name("my@plugin").is_err()); // @
        assert!(validate_name("my!plugin").is_err()); // !
        assert!(validate_name("プラグイン").is_err()); // non-ASCII
    }

    #[test]
    fn validate_name_rejects_too_long_name() {
        let long_name = "a".repeat(65);
        assert!(validate_name(&long_name).is_err());
    }

    #[test]
    fn validate_name_accepts_max_length_name() {
        let max_name = "a".repeat(64);
        assert!(validate_name(&max_name).is_ok());
    }

    #[test]
    fn validate_name_rejects_leading_period() {
        assert!(validate_name(".hidden").is_err());
    }

    #[test]
    fn validate_name_rejects_trailing_period() {
        assert!(validate_name("plugin.").is_err());
    }

    #[test]
    fn validate_name_rejects_leading_hyphen() {
        assert!(validate_name("-plugin").is_err());
    }

    #[test]
    fn validate_name_rejects_trailing_hyphen() {
        assert!(validate_name("plugin-").is_err());
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert!(validate_name("").is_err());
    }

    // ==================== normalize_source_path tests ====================

    #[test]
    fn normalize_source_path_removes_leading_slash() {
        let result = normalize_source_path("/plugins/marketplace").unwrap();
        assert_eq!(result, Some("plugins/marketplace".to_string()));
    }

    #[test]
    fn normalize_source_path_removes_trailing_slash() {
        let result = normalize_source_path("plugins/marketplace/").unwrap();
        assert_eq!(result, Some("plugins/marketplace".to_string()));
    }

    #[test]
    fn normalize_source_path_removes_both_slashes() {
        let result = normalize_source_path("/plugins/").unwrap();
        assert_eq!(result, Some("plugins".to_string()));
    }

    #[test]
    fn normalize_source_path_removes_dot_slash_prefix() {
        let result = normalize_source_path("./plugins/marketplace").unwrap();
        assert_eq!(result, Some("plugins/marketplace".to_string()));
    }

    #[test]
    fn normalize_source_path_returns_none_for_empty() {
        let result = normalize_source_path("").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_source_path_returns_none_for_dot_only() {
        let result = normalize_source_path(".").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_source_path_returns_none_for_dot_slash_only() {
        let result = normalize_source_path("./").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_source_path_rejects_double_dot() {
        assert!(normalize_source_path("../plugins").is_err());
        assert!(normalize_source_path("plugins/../other").is_err());
        assert!(normalize_source_path("plugins/..").is_err());
        assert!(normalize_source_path("./..").is_err());
    }

    #[test]
    fn normalize_source_path_rejects_backslash() {
        assert!(normalize_source_path("plugins\\marketplace").is_err());
    }

    // ==================== to_display_source / to_internal_source tests ====================

    #[test]
    fn to_display_source_removes_github_prefix() {
        let result = to_display_source("github:owner/repo");
        assert_eq!(result, "owner/repo");
    }

    #[test]
    fn to_display_source_returns_unchanged_if_no_prefix() {
        let result = to_display_source("owner/repo");
        assert_eq!(result, "owner/repo");
    }

    #[test]
    fn to_internal_source_adds_github_prefix() {
        let result = to_internal_source("owner/repo");
        assert_eq!(result, "github:owner/repo");
    }

    #[test]
    fn to_internal_source_returns_unchanged_if_already_prefixed() {
        let result = to_internal_source("github:owner/repo");
        assert_eq!(result, "github:owner/repo");
    }

    // ==================== MarketplaceConfig tests ====================

    #[test]
    fn config_load_returns_empty_for_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");
        let config = MarketplaceConfig::load_from(path).unwrap();
        assert!(config.list().is_empty());
    }

    #[test]
    fn config_load_reads_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        std::fs::write(
            &path,
            r#"{"marketplaces":[{"name":"test-mp","source":"owner/repo"}]}"#,
        )
        .unwrap();

        let config = MarketplaceConfig::load_from(path).unwrap();
        assert_eq!(config.list().len(), 1);
        assert_eq!(config.list()[0].name, "test-mp");
    }

    #[test]
    fn config_save_writes_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path.clone()).unwrap();

        let entry = MarketplaceEntry {
            name: "my-mp".to_string(),
            source: "owner/repo".to_string(),
            source_path: None,
        };
        config.add(entry).unwrap();
        config.save().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("my-mp"));
        assert!(content.contains("owner/repo"));
    }

    #[test]
    fn config_add_entry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        let entry = MarketplaceEntry {
            name: "test-mp".to_string(),
            source: "owner/repo".to_string(),
            source_path: None,
        };
        config.add(entry).unwrap();

        assert_eq!(config.list().len(), 1);
        assert!(config.exists("test-mp"));
    }

    #[test]
    fn config_add_rejects_duplicate_name() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        let entry1 = MarketplaceEntry {
            name: "test-mp".to_string(),
            source: "owner/repo1".to_string(),
            source_path: None,
        };
        config.add(entry1).unwrap();

        let entry2 = MarketplaceEntry {
            name: "test-mp".to_string(),
            source: "owner/repo2".to_string(),
            source_path: None,
        };
        assert!(config.add(entry2).is_err());
    }

    #[test]
    fn config_remove_entry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        let entry = MarketplaceEntry {
            name: "test-mp".to_string(),
            source: "owner/repo".to_string(),
            source_path: None,
        };
        config.add(entry).unwrap();
        config.remove("test-mp").unwrap();

        assert!(config.list().is_empty());
        assert!(!config.exists("test-mp"));
    }

    #[test]
    fn config_remove_nonexistent_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        assert!(config.remove("nonexistent").is_err());
    }

    #[test]
    fn config_get_returns_entry() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        let entry = MarketplaceEntry {
            name: "test-mp".to_string(),
            source: "owner/repo".to_string(),
            source_path: Some("plugins".to_string()),
        };
        config.add(entry).unwrap();

        let result = config.get("test-mp");
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, "owner/repo");
        assert_eq!(result.unwrap().source_path, Some("plugins".to_string()));
    }

    #[test]
    fn config_get_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let config = MarketplaceConfig::load_from(path).unwrap();

        assert!(config.get("nonexistent").is_none());
    }

    #[test]
    fn config_list_returns_all_entries() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("marketplaces.json");
        let mut config = MarketplaceConfig::load_from(path).unwrap();

        config
            .add(MarketplaceEntry {
                name: "mp1".to_string(),
                source: "owner/repo1".to_string(),
                source_path: None,
            })
            .unwrap();
        config
            .add(MarketplaceEntry {
                name: "mp2".to_string(),
                source: "owner/repo2".to_string(),
                source_path: None,
            })
            .unwrap();

        let list = config.list();
        assert_eq!(list.len(), 2);
    }
}
