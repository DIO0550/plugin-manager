use super::*;
use crate::component::convert::{AgentConversionResult, AgentFormat, ConversionResult};
use crate::component::CommandFormat;
use crate::hooks::converter::ConversionWarning;

// ========================================
// HookConvertOutput tests
// ========================================

#[test]
fn test_hook_convert_result_has_expected_fields() {
    let result = HookConvertOutput {
        warnings: vec![ConversionWarning::MissingVersion],
        script_count: 2,
    };

    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.script_count, 2);
}

#[test]
fn test_deployment_result_hook_converted_variant() {
    let hook_result = HookConvertOutput {
        warnings: vec![],
        script_count: 0,
    };
    let result = DeploymentOutput::HookConverted(hook_result);

    match result {
        DeploymentOutput::HookConverted(hr) => {
            assert_eq!(hr.script_count, 0);
            assert!(hr.warnings.is_empty());
        }
        _ => panic!("Expected HookConverted"),
    }
}

// ========================================
// Display trait tests
// ========================================

#[test]
fn test_display_copied() {
    let result = DeploymentOutput::Copied;
    assert_eq!(result.to_string(), "Copied");
}

#[test]
fn test_display_converted_true() {
    let result = DeploymentOutput::CommandConverted(ConversionResult {
        converted: true,
        source_format: CommandFormat::ClaudeCode,
        dest_format: CommandFormat::Copilot,
    });
    assert_eq!(result.to_string(), "Converted: ClaudeCode → Copilot");
}

#[test]
fn test_display_converted_false() {
    let result = DeploymentOutput::CommandConverted(ConversionResult {
        converted: false,
        source_format: CommandFormat::ClaudeCode,
        dest_format: CommandFormat::ClaudeCode,
    });
    assert_eq!(result.to_string(), "Copied (no conversion needed)");
}

#[test]
fn test_display_agent_converted_true() {
    let result = DeploymentOutput::AgentConverted(AgentConversionResult {
        converted: true,
        source_format: AgentFormat::ClaudeCode,
        dest_format: AgentFormat::Copilot,
    });
    assert_eq!(result.to_string(), "Agent converted: ClaudeCode → Copilot");
}

#[test]
fn test_display_agent_converted_false() {
    let result = DeploymentOutput::AgentConverted(AgentConversionResult {
        converted: false,
        source_format: AgentFormat::ClaudeCode,
        dest_format: AgentFormat::ClaudeCode,
    });
    assert_eq!(result.to_string(), "Copied (no agent conversion needed)");
}

#[test]
fn test_display_hook_converted() {
    let result = DeploymentOutput::HookConverted(HookConvertOutput {
        warnings: vec![ConversionWarning::MissingVersion],
        script_count: 3,
    });
    assert_eq!(result.to_string(), "Hook converted (3 scripts, 1 warning)");
}
