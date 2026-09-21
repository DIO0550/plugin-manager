use super::*;

#[test]
fn allows_scope_reads_table_and_defaults_to_false() {
    let table = &[
        (ComponentKind::Skill, ScopeSupport::Both),
        (ComponentKind::Instruction, ScopeSupport::ProjectOnly),
    ];
    assert!(allows_scope(table, ComponentKind::Skill, Scope::Personal));
    assert!(allows_scope(table, ComponentKind::Skill, Scope::Project));
    assert!(!allows_scope(
        table,
        ComponentKind::Instruction,
        Scope::Personal
    ));
    assert!(allows_scope(
        table,
        ComponentKind::Instruction,
        Scope::Project
    ));
    assert!(!allows_scope(table, ComponentKind::Hook, Scope::Project));
}

#[test]
fn capabilities_macro_keeps_supported_in_lockstep_with_table() {
    const CAPS: Capabilities = capabilities!(
        Skill => Both,
        Command => ProjectOnly,
        Instruction => PersonalOnly,
    );
    assert_eq!(
        CAPS.supported(),
        &[
            ComponentKind::Skill,
            ComponentKind::Command,
            ComponentKind::Instruction,
        ]
    );
    assert_eq!(CAPS.derived_supported(), CAPS.supported());
    assert!(CAPS.allows(ComponentKind::Skill, Scope::Personal));
    assert!(!CAPS.allows(ComponentKind::Command, Scope::Personal));
    assert!(CAPS.allows(ComponentKind::Command, Scope::Project));
    assert!(CAPS.allows(ComponentKind::Instruction, Scope::Personal));
    assert!(!CAPS.allows(ComponentKind::Instruction, Scope::Project));
    assert!(!CAPS.allows(ComponentKind::Agent, Scope::Project));
}
