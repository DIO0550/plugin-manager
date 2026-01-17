use super::*;
use crate::component::{ComponentKind, Scope};
use crate::fs::mock::MockFs;

#[test]
fn test_needs_update_newer_source() {
    let fs = MockFs::new();
    fs.add_file("/src/test.md", "new content");
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs.add_file("/dst/test.md", "old content");

    // Note: MockFs の mtime は追加時刻なので、src の方が古い
    // 実際のテストでは内容ハッシュで判定される
    let src = PlacedComponent::new(ComponentKind::Skill, "test", Scope::Personal, "/src/test.md");
    let dst = PlacedComponent::new(ComponentKind::Skill, "test", Scope::Personal, "/dst/test.md");

    // 内容が違うので更新が必要
    let result = needs_update(&src, &dst, &fs).unwrap();
    assert!(result);
}

#[test]
fn test_needs_update_same_content() {
    let fs = MockFs::new();
    fs.add_file("/src/test.md", "same content");
    fs.add_file("/dst/test.md", "same content");

    let src = PlacedComponent::new(ComponentKind::Skill, "test", Scope::Personal, "/src/test.md");
    let dst = PlacedComponent::new(ComponentKind::Skill, "test", Scope::Personal, "/dst/test.md");

    let result = needs_update(&src, &dst, &fs).unwrap();
    assert!(!result);
}
