use super::*;
use crate::fs::RealFs;
use crate::scan::AttachedEntry;
use std::fs;
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
