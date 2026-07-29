use super::*;
use crate::fs::RealFs;
use crate::scan::AttachedEntry;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn overlay_copies_references_into_skill_target() {
    let temp = TempDir::new().unwrap();
    let plugin = temp.path().join("plugin");
    let skill_target = temp.path().join("dest/my-skill");
    fs::create_dir_all(plugin.join("references")).unwrap();
    fs::write(plugin.join("references/tdd-guidelines.md"), "tdd\n").unwrap();
    fs::create_dir_all(&skill_target).unwrap();
    fs::write(skill_target.join("SKILL.md"), "# skill\n").unwrap();

    let entries = vec![AttachedEntry::new("references")];
    let warnings = overlay_attached_resources(&RealFs, &plugin, &entries, &skill_target).unwrap();

    assert!(warnings.is_empty());
    assert_eq!(
        fs::read_to_string(skill_target.join("references/tdd-guidelines.md")).unwrap(),
        "tdd\n"
    );
}

#[test]
fn overlay_prefers_skill_local_on_collision() {
    let temp = TempDir::new().unwrap();
    let plugin = temp.path().join("plugin");
    let skill_target = temp.path().join("dest/my-skill");
    fs::create_dir_all(plugin.join("references")).unwrap();
    fs::write(
        plugin.join("references/tdd-guidelines.md"),
        "plugin-version\n",
    )
    .unwrap();
    fs::create_dir_all(skill_target.join("references")).unwrap();
    fs::write(
        skill_target.join("references/tdd-guidelines.md"),
        "skill-version\n",
    )
    .unwrap();
    fs::write(skill_target.join("SKILL.md"), "# skill\n").unwrap();

    let entries = vec![AttachedEntry::new("references")];
    let warnings = overlay_attached_resources(&RealFs, &plugin, &entries, &skill_target).unwrap();

    assert_eq!(
        warnings,
        vec![AttachedResourceWarning::SkillPreferred {
            relative: PathBuf::from("references/tdd-guidelines.md"),
        }]
    );
    assert_eq!(
        fs::read_to_string(skill_target.join("references/tdd-guidelines.md")).unwrap(),
        "skill-version\n"
    );
}

#[test]
fn overlay_merges_non_colliding_files_into_existing_dir() {
    let temp = TempDir::new().unwrap();
    let plugin = temp.path().join("plugin");
    let skill_target = temp.path().join("dest/my-skill");
    fs::create_dir_all(plugin.join("references")).unwrap();
    fs::write(plugin.join("references/tdd-guidelines.md"), "plugin\n").unwrap();
    fs::create_dir_all(skill_target.join("references")).unwrap();
    fs::write(
        skill_target.join("references/exploration.md"),
        "skill-local\n",
    )
    .unwrap();

    let entries = vec![AttachedEntry::new("references")];
    let warnings = overlay_attached_resources(&RealFs, &plugin, &entries, &skill_target).unwrap();

    assert!(warnings.is_empty());
    assert_eq!(
        fs::read_to_string(skill_target.join("references/tdd-guidelines.md")).unwrap(),
        "plugin\n"
    );
    assert_eq!(
        fs::read_to_string(skill_target.join("references/exploration.md")).unwrap(),
        "skill-local\n"
    );
}

#[test]
fn overlay_skips_when_total_size_exceeds_limit() {
    let temp = TempDir::new().unwrap();
    let plugin = temp.path().join("plugin");
    let skill_target = temp.path().join("dest/my-skill");
    fs::create_dir_all(plugin.join("references")).unwrap();
    fs::write(plugin.join("references/big.md"), "abcdefghij").unwrap(); // 10 bytes
    fs::create_dir_all(&skill_target).unwrap();
    fs::write(skill_target.join("SKILL.md"), "# skill\n").unwrap();

    let entries = vec![AttachedEntry::new("references")];
    let warnings =
        overlay_attached_resources_with_limit(&RealFs, &plugin, &entries, &skill_target, 5)
            .unwrap();

    assert_eq!(
        warnings,
        vec![AttachedResourceWarning::SkippedTooLarge {
            total_bytes: 10,
            limit_bytes: 5,
        }]
    );
    assert!(!skill_target.join("references/big.md").exists());
}

#[test]
fn overlay_size_measurement_uses_filesystem_abstraction() {
    use crate::fs::mock::MockFs;

    let fs = MockFs::new();
    fs.add_dir("/plugin");
    fs.add_dir("/plugin/references");
    fs.add_file("/plugin/references/a.md", "12345");
    fs.add_dir("/dest/skill");
    fs.add_file("/dest/skill/SKILL.md", "# s\n");

    let entries = vec![AttachedEntry::new("references")];
    let warnings = overlay_attached_resources_with_limit(
        &fs,
        Path::new("/plugin"),
        &entries,
        Path::new("/dest/skill"),
        4,
    )
    .unwrap();

    assert!(matches!(
        &warnings[..],
        [AttachedResourceWarning::SkippedTooLarge {
            total_bytes: 5,
            limit_bytes: 4
        }]
    ));
    assert!(!fs.exists(Path::new("/dest/skill/references/a.md")));
}
