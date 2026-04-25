use super::*;
use crate::component::convert::{AgentFormat, CommandFormat};

#[test]
fn test_default_is_none() {
    let cfg = ConversionConfig::default();
    assert!(matches!(cfg, ConversionConfig::None));
}

#[test]
fn test_clone_preserves_variant() {
    let cmd = ConversionConfig::Command {
        source: CommandFormat::ClaudeCode,
        dest: CommandFormat::Copilot,
    };
    let cloned = cmd.clone();
    assert!(matches!(
        cloned,
        ConversionConfig::Command {
            source: CommandFormat::ClaudeCode,
            dest: CommandFormat::Copilot,
        }
    ));

    let agent = ConversionConfig::Agent {
        source: AgentFormat::ClaudeCode,
        dest: AgentFormat::Codex,
    };
    let cloned = agent.clone();
    assert!(matches!(
        cloned,
        ConversionConfig::Agent {
            source: AgentFormat::ClaudeCode,
            dest: AgentFormat::Codex,
        }
    ));
}
