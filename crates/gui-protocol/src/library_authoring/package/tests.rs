use super::*;

fn set(draft: &mut LibraryDraft, key: &str, text: &str) {
    draft.focused = draft.fields.iter().position(|f| f.key == key).unwrap();
    draft.change(text.into());
}

#[test]
fn package_draft_retains_terminal_identities_and_exact_optional_intent() {
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        None,
        "packages",
        "revision".into(),
        None,
    )
    .unwrap();
    set(&mut d, "name", "My body");
    set(&mut d, "body_x", "5 mm");
    set(&mut d, "terminal_name", "1");
    let first = d.terminal_id.unwrap();
    set(&mut d, "terminal_role", "power");
    d.switch_terminal(None, false).unwrap();
    set(&mut d, "terminal_name", "2");
    let second = d.terminal_id.unwrap();
    assert_ne!(first, second);
    let a = d.request().unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(d.request().unwrap()).unwrap()
    );
    assert_eq!(a.object_id, d.id);
    assert_eq!(a.fields["body_dimensions"], json!({"x_nm":5000000}));
    assert_eq!(
        a.fields["terminals"][first.to_string()],
        json!({"uuid":first,"name":"1","role":"power"})
    );
    assert_eq!(
        a.fields["terminals"][second.to_string()],
        json!({"uuid":second,"name":"2"})
    );
    assert!(!a.fields.contains_key("mounting_type"));
    d.switch_terminal(None, true).unwrap();
    let b = d.request().unwrap();
    assert_eq!(b.fields["terminals"].as_object().unwrap().len(), 1);
    assert_eq!(b.fields["terminals"][first.to_string()]["name"], json!("1"));
    set(&mut d, "body_y", "not a length");
    assert!(d.request().is_err());
    d.switch_terminal(Some(0), false).unwrap();
    assert_eq!(
        d.terminal_id,
        Some(first),
        "malformed body does not trap terminal navigation"
    );
}

#[test]
fn empty_package_draft_does_not_invent_terminals_dimensions_or_mounting() {
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        None,
        "packages",
        "revision".into(),
        None,
    )
    .unwrap();
    set(&mut d, "name", "Named body");
    assert_eq!(
        d.request().unwrap().fields,
        json!({"name":"Named body"}).as_object().unwrap().clone()
    );
    set(&mut d, "terminal_role", "dangling role");
    assert!(d.request().is_err());
}
