//! コンポーネントのデプロイ処理

use crate::component::{Component, ComponentKind, Scope};
use crate::error::{PlmError, Result};
use crate::path_ext::PathExt;
use std::path::{Path, PathBuf};

/// コンポーネントのデプロイ情報
///
/// 配置の実行（コピー/削除など）を担当する。
/// 配置先の決定は `PlacementLocation` が担当する。
#[derive(Debug, Clone)]
pub struct ComponentDeployment {
    pub kind: ComponentKind,
    pub name: String,
    pub scope: Scope,
    source_path: PathBuf,
    target_path: PathBuf,
}

impl ComponentDeployment {
    /// Builderを生成
    pub fn builder() -> ComponentDeploymentBuilder {
        ComponentDeploymentBuilder::new()
    }

    /// 配置先パスを取得
    pub fn path(&self) -> &Path {
        &self.target_path
    }

    /// ソースパスを取得
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// 配置を実行（ファイルコピー）
    pub fn execute(&self) -> Result<()> {
        match self.kind {
            ComponentKind::Skill => {
                // Skills are directories
                self.source_path.copy_dir_to(&self.target_path)?;
            }
            ComponentKind::Agent
            | ComponentKind::Command
            | ComponentKind::Instruction
            | ComponentKind::Hook => {
                // These are files
                self.source_path.copy_file_to(&self.target_path)?;
            }
        }
        Ok(())
    }
}

/// ComponentDeployment のビルダー
#[derive(Debug, Default)]
pub struct ComponentDeploymentBuilder {
    kind: Option<ComponentKind>,
    name: Option<String>,
    scope: Option<Scope>,
    source_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
}

impl ComponentDeploymentBuilder {
    /// 新しいビルダーを生成
    pub fn new() -> Self {
        Self::default()
    }

    /// Component から kind, name, source_path を設定
    pub fn component(mut self, component: &Component) -> Self {
        self.kind = Some(component.kind);
        self.name = Some(component.name.clone());
        self.source_path = Some(component.path.clone());
        self
    }

    /// コンポーネント種別を設定
    pub fn kind(mut self, kind: ComponentKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// コンポーネント名を設定
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// スコープを設定
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// ソースパスを設定
    pub fn source_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    /// ターゲットパスを設定
    pub fn target_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.target_path = Some(path.into());
        self
    }

    /// ComponentDeployment を構築
    pub fn build(self) -> Result<ComponentDeployment> {
        let kind = self
            .kind
            .ok_or_else(|| PlmError::Validation("kind is required".to_string()))?;
        let name = self
            .name
            .ok_or_else(|| PlmError::Validation("name is required".to_string()))?;
        let scope = self
            .scope
            .ok_or_else(|| PlmError::Validation("scope is required".to_string()))?;
        let source_path = self
            .source_path
            .ok_or_else(|| PlmError::Validation("source_path is required".to_string()))?;
        let target_path = self
            .target_path
            .ok_or_else(|| PlmError::Validation("target_path is required".to_string()))?;

        Ok(ComponentDeployment {
            kind,
            name,
            scope,
            source_path,
            target_path,
        })
    }
}

#[cfg(test)]
mod tests {
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
}

