use super::{compute_multi_select_outcome, SelectItem};

fn item(label: &str, value: &str) -> SelectItem<String> {
    SelectItem::new(label, value.to_string())
}

#[test]
fn multi_select_promotes_cursor_when_none_selected() {
    let items = vec![item("A", "a"), item("B", "b"), item("C", "c")];

    let outcome = compute_multi_select_outcome(&items, 1);

    assert_eq!(outcome.selected, vec!["b".to_string()]);
    assert!(!outcome.cancelled);
}

#[test]
fn multi_select_keeps_existing_selection() {
    let items = vec![
        item("A", "a"),
        item("B", "b").with_selected(true),
        item("C", "c"),
    ];

    let outcome = compute_multi_select_outcome(&items, 0);

    assert_eq!(outcome.selected, vec!["b".to_string()]);
}

#[test]
fn multi_select_returns_empty_when_no_items() {
    let items: Vec<SelectItem<String>> = vec![];

    let outcome = compute_multi_select_outcome(&items, 0);

    assert!(outcome.selected.is_empty());
    assert!(!outcome.cancelled);
}

#[test]
fn multi_select_skips_disabled_cursor() {
    let items = vec![item("A", "a").with_enabled(false)];

    let outcome = compute_multi_select_outcome(&items, 0);

    assert!(outcome.selected.is_empty());
}

#[test]
fn multi_select_clamps_cursor_when_idx_out_of_range() {
    let items = vec![item("A", "a"), item("B", "b")];

    let outcome = compute_multi_select_outcome(&items, 99);

    assert_eq!(outcome.selected, vec!["b".to_string()]);
}

#[test]
fn multi_select_promotes_cursor_at_zero_when_first_disabled() {
    let items = vec![item("A", "a").with_enabled(false), item("B", "b")];

    let outcome = compute_multi_select_outcome(&items, 0);

    assert!(
        outcome.selected.is_empty(),
        "Disabled cursor target should not be promoted"
    );
}
