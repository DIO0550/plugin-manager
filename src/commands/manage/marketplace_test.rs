use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn plm() -> Command {
    Command::cargo_bin("plm").unwrap()
}

fn write_marketplace_config(home: &TempDir) {
    let plm_dir = home.path().join(".plm");
    fs::create_dir_all(&plm_dir).unwrap();
    fs::write(
        plm_dir.join("marketplaces.json"),
        r#"{
          "marketplaces": [
            {
              "name": "my-mp",
              "source": "github:owner/repo"
            }
          ]
        }"#,
    )
    .unwrap();
}

#[test]
fn show_normalizes_uppercase_marketplace_name() {
    let home = TempDir::new().unwrap();
    write_marketplace_config(&home);

    plm()
        .env("PLM_HOME", home.path())
        .args(["marketplace", "show", "My-MP"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Marketplace: my-mp"));
}

#[test]
fn remove_normalizes_uppercase_marketplace_name() {
    let home = TempDir::new().unwrap();
    write_marketplace_config(&home);

    plm()
        .env("PLM_HOME", home.path())
        .args(["marketplace", "remove", "My-MP"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed marketplace 'my-mp'."));

    let content = fs::read_to_string(home.path().join(".plm/marketplaces.json")).unwrap();
    let json: Value = serde_json::from_str(&content).unwrap();
    assert!(json["marketplaces"].as_array().unwrap().is_empty());
}
