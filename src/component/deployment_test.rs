use super::*;
use std::fs;
use tempfile::TempDir;

// ========================================
// Builder tests
// ========================================

#[test]
fn test_builder_builds_with_all_fields() {
    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Agent)
        .name("test-agent")
        .scope(Scope::Project)
        .source_path("/src/agent.md")
        .target_path("/dest/agent.md")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Agent);
    assert_eq!(deployment.name, "test-agent");
    assert_eq!(deployment.scope, Scope::Project);
    assert_eq!(deployment.source_path(), Path::new("/src/agent.md"));
    assert_eq!(deployment.path(), Path::new("/dest/agent.md"));
}

#[test]
fn test_builder_fails_without_kind() {
    let result = ComponentDeployment::builder()
        .name("test")
        .scope(Scope::Personal)
        .source_path("/src")
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_name() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .scope(Scope::Personal)
        .source_path("/src")
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_scope() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("test")
        .source_path("/src")
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_source_path() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("test")
        .scope(Scope::Personal)
        .target_path("/dest")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_fails_without_target_path() {
    let result = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("test")
        .scope(Scope::Personal)
        .source_path("/src")
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_from_component() {
    let component = Component {
        kind: ComponentKind::Command,
        name: "my-command".to_string(),
        path: PathBuf::from("/plugin/commands/my-command.md"),
    };

    let deployment = ComponentDeployment::builder()
        .component(&component)
        .scope(Scope::Project)
        .target_path("/target/my-command.md")
        .build()
        .unwrap();

    assert_eq!(deployment.kind, ComponentKind::Command);
    assert_eq!(deployment.name, "my-command");
    assert_eq!(
        deployment.source_path(),
        Path::new("/plugin/commands/my-command.md")
    );
}

// ========================================
// Execute tests
// ========================================

#[test]
fn test_execute_copies_file_for_agent() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("agent.md");
    let target = temp.path().join("dest/agent.md");

    fs::write(&source, "agent content").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Agent)
        .name("test-agent")
        .scope(Scope::Project)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "agent content");
}

#[test]
fn test_execute_copies_file_for_command() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("cmd.prompt.md");
    let target = temp.path().join("dest/cmd.prompt.md");

    fs::write(&source, "command content").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Command)
        .name("test-cmd")
        .scope(Scope::Project)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_file_for_instruction() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("instruction.md");
    let target = temp.path().join("dest/instruction.md");

    fs::write(&source, "instruction content").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Instruction)
        .name("test-instruction")
        .scope(Scope::Personal)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_file_for_hook() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("hook.sh");
    let target = temp.path().join("dest/hook.sh");

    fs::write(&source, "#!/bin/bash").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Hook)
        .name("test-hook")
        .scope(Scope::Personal)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
}

#[test]
fn test_execute_copies_directory_for_skill() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("my-skill");
    let target = temp.path().join("dest/my-skill");

    fs::create_dir(&source).unwrap();
    fs::write(source.join("SKILL.md"), "skill content").unwrap();
    fs::write(source.join("helper.py"), "print('hello')").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("my-skill")
        .scope(Scope::Project)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(target.exists());
    assert!(target.is_dir());
    assert!(target.join("SKILL.md").exists());
    assert!(target.join("helper.py").exists());
}

#[test]
fn test_execute_skill_replaces_existing_directory() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("skill");
    let target = temp.path().join("dest/skill");

    // Create source
    fs::create_dir(&source).unwrap();
    fs::write(source.join("new.md"), "new").unwrap();

    // Create existing target with different content
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("old.md"), "old").unwrap();

    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Skill)
        .name("skill")
        .scope(Scope::Project)
        .source_path(&source)
        .target_path(&target)
        .build()
        .unwrap();

    deployment.execute().unwrap();

    assert!(!target.join("old.md").exists());
    assert!(target.join("new.md").exists());
}

// ========================================
// Accessor tests
// ========================================

#[test]
fn test_path_returns_target_path() {
    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Agent)
        .name("test")
        .scope(Scope::Project)
        .source_path("/src/test.md")
        .target_path("/dest/test.md")
        .build()
        .unwrap();

    assert_eq!(deployment.path(), Path::new("/dest/test.md"));
}

#[test]
fn test_source_path_returns_source() {
    let deployment = ComponentDeployment::builder()
        .kind(ComponentKind::Agent)
        .name("test")
        .scope(Scope::Project)
        .source_path("/src/test.md")
        .target_path("/dest/test.md")
        .build()
        .unwrap();

    assert_eq!(deployment.source_path(), Path::new("/src/test.md"));
}
