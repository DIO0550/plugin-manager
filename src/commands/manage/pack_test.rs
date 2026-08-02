//! plm pack コマンドのテスト

use super::*;
use std::fs;
use std::io::Read;
use tempfile::TempDir;

fn write_skill(dir: &std::path::Path, name: &str, skill_md: &str) {
    let skill_dir = dir.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), skill_md).unwrap();
}

fn valid_skill_md(name: &str) -> String {
    format!(
        r#"---
name: {name}
description: a test skill
---

# {name}
"#
    )
}

fn read_zip_entries(zip_path: &std::path::Path) -> Vec<String> {
    let file = fs::File::open(zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut names = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).unwrap();
        names.push(entry.name().to_string());
    }
    names.sort();
    names
}

fn read_zip_file(zip_path: &std::path::Path, name: &str) -> String {
    let file = fs::File::open(zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut entry = archive.by_name(name).unwrap();
    let mut buf = String::new();
    entry.read_to_string(&mut buf).unwrap();
    buf
}

#[test]
fn pack_skill_creates_zip_with_skill_md() {
    let tmp = TempDir::new().unwrap();
    write_skill(tmp.path(), "my-skill", &valid_skill_md("my-skill"));

    let zip_path = pack_path(&tmp.path().join("my-skill"), tmp.path()).unwrap();
    assert_eq!(zip_path, tmp.path().join("my-skill.zip"));
    assert!(zip_path.is_file());

    let entries = read_zip_entries(&zip_path);
    assert!(entries.iter().any(|e| e == "SKILL.md" || e == "SKILL.md/"));
    let content = read_zip_file(&zip_path, "SKILL.md");
    assert!(content.contains("name: my-skill"));
}

#[test]
fn pack_plugin_creates_zip_with_manifest() {
    let tmp = TempDir::new().unwrap();
    let plugin = tmp.path().join("my-plugin");
    fs::create_dir_all(plugin.join(".claude-plugin")).unwrap();
    fs::write(
        plugin.join(".claude-plugin/plugin.json"),
        r#"{"name":"my-plugin","version":"1.0.0"}"#,
    )
    .unwrap();
    fs::create_dir_all(plugin.join("skills/hello")).unwrap();
    fs::write(
        plugin.join("skills/hello/SKILL.md"),
        valid_skill_md("hello"),
    )
    .unwrap();

    let zip_path = pack_path(&plugin, tmp.path()).unwrap();
    assert_eq!(zip_path, tmp.path().join("my-plugin.zip"));

    let entries = read_zip_entries(&zip_path);
    assert!(entries.iter().any(|e| e.contains("plugin.json")));
    assert!(entries.iter().any(|e| e.contains("SKILL.md")));
}

#[test]
fn pack_skill_rejects_missing_frontmatter_name() {
    let tmp = TempDir::new().unwrap();
    write_skill(
        tmp.path(),
        "bad-skill",
        "---\ndescription: no name\n---\n\n# bad\n",
    );

    let err = pack_path(&tmp.path().join("bad-skill"), tmp.path()).unwrap_err();
    assert!(
        err.to_lowercase().contains("name") || err.to_lowercase().contains("frontmatter"),
        "unexpected error: {err}"
    );
}

#[test]
fn pack_skill_rejects_invalid_yaml() {
    let tmp = TempDir::new().unwrap();
    write_skill(
        tmp.path(),
        "bad-yaml",
        "---\nname: [unterminated\n---\n\n# bad\n",
    );

    let err = pack_path(&tmp.path().join("bad-yaml"), tmp.path()).unwrap_err();
    assert!(
        err.to_lowercase().contains("yaml")
            || err.to_lowercase().contains("frontmatter")
            || err.to_lowercase().contains("parse"),
        "unexpected error: {err}"
    );
}

#[test]
fn pack_rejects_missing_skill_md_and_manifest() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("empty");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("README.md"), "hi").unwrap();

    let err = pack_path(&dir, tmp.path()).unwrap_err();
    assert!(
        err.contains("SKILL.md") || err.contains("plugin.json") || err.contains("Unrecognized"),
        "unexpected error: {err}"
    );
}

#[test]
fn pack_rejects_invalid_plugin_json() {
    let tmp = TempDir::new().unwrap();
    let plugin = tmp.path().join("broken");
    fs::create_dir_all(plugin.join(".claude-plugin")).unwrap();
    fs::write(plugin.join(".claude-plugin/plugin.json"), "{not json").unwrap();

    let err = pack_path(&plugin, tmp.path()).unwrap_err();
    assert!(
        err.to_lowercase().contains("plugin.json")
            || err.to_lowercase().contains("manifest")
            || err.to_lowercase().contains("parse"),
        "unexpected error: {err}"
    );
}

#[test]
fn pack_fails_when_zip_already_exists() {
    let tmp = TempDir::new().unwrap();
    write_skill(tmp.path(), "my-skill", &valid_skill_md("my-skill"));
    fs::write(tmp.path().join("my-skill.zip"), "existing").unwrap();

    let err = pack_path(&tmp.path().join("my-skill"), tmp.path()).unwrap_err();
    assert!(
        err.contains("already exists") || err.contains("exists"),
        "unexpected error: {err}"
    );
}

#[test]
fn pack_excludes_git_and_plm_meta() {
    let tmp = TempDir::new().unwrap();
    write_skill(tmp.path(), "my-skill", &valid_skill_md("my-skill"));
    let skill = tmp.path().join("my-skill");
    fs::create_dir_all(skill.join(".git")).unwrap();
    fs::write(skill.join(".git/config"), "x").unwrap();
    fs::write(skill.join(".plm-meta.json"), "{}").unwrap();
    fs::write(skill.join("notes.txt"), "keep").unwrap();

    let zip_path = pack_path(&skill, tmp.path()).unwrap();
    let entries = read_zip_entries(&zip_path);
    assert!(
        !entries.iter().any(|e| e.contains(".git")),
        "zip should exclude .git: {entries:?}"
    );
    assert!(
        !entries.iter().any(|e| e.contains(".plm-meta.json")),
        "zip should exclude .plm-meta.json: {entries:?}"
    );
    assert!(entries.iter().any(|e| e == "notes.txt"));
}

#[test]
fn pack_rejects_nonexistent_path() {
    let tmp = TempDir::new().unwrap();
    let err = pack_path(&tmp.path().join("missing"), tmp.path()).unwrap_err();
    assert!(
        err.contains("not found") || err.contains("No such") || err.contains("does not exist"),
        "unexpected error: {err}"
    );
}
