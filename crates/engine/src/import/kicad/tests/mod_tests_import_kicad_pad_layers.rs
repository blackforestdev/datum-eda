//! M7-IMP-011 pure-helper tests for explicit pad copper-layer membership.
//!
//! These tests exercise `resolve_pad_copper_layers` and
//! `resolve_pad_primary_copper_layer` directly against token-list inputs and a
//! parsed layer table. They do NOT fabricate any KiCad s-expression content.

use crate::error::EngineError;
use crate::import::kicad::skeleton::{
    resolve_pad_copper_layers, resolve_pad_primary_copper_layer,
};

fn tokens(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn table(entries: &[(&str, i32)]) -> std::collections::HashMap<String, i32> {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn smd_front_copper_resolves_to_single_f_cu_membership() {
    let t = table(&[]);
    let layers = resolve_pad_copper_layers(&tokens(&["F.Cu", "F.Mask", "F.Paste"]), &t).unwrap();
    assert_eq!(layers, vec![0]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 0);
}

#[test]
fn smd_back_copper_resolves_to_single_b_cu_membership() {
    let t = table(&[]);
    let layers = resolve_pad_copper_layers(&tokens(&["B.Cu", "B.Mask", "B.Paste"]), &t).unwrap();
    assert_eq!(layers, vec![31]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 31);
}

#[test]
fn through_hole_star_cu_expands_to_all_declared_copper_layers() {
    let t = table(&[("F.Cu", 0), ("In1.Cu", 2), ("In2.Cu", 4), ("B.Cu", 31)]);
    let layers = resolve_pad_copper_layers(&tokens(&["*.Cu", "*.Mask"]), &t).unwrap();
    assert_eq!(layers, vec![0, 2, 4, 31]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 0);
}

#[test]
fn f_and_b_cu_wildcard_expands_to_outer_copper_only() {
    let t = table(&[("F.Cu", 0), ("In1.Cu", 2), ("B.Cu", 31)]);
    let layers = resolve_pad_copper_layers(&tokens(&["F&B.Cu", "*.Mask"]), &t).unwrap();
    assert_eq!(layers, vec![0, 31]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 0);
}

#[test]
fn multi_copper_entries_union_and_primary_is_topmost() {
    let t = table(&[("F.Cu", 0), ("In1.Cu", 2), ("In2.Cu", 4), ("B.Cu", 31)]);
    let layers = resolve_pad_copper_layers(&tokens(&["F.Cu", "In1.Cu", "B.Cu"]), &t).unwrap();
    assert_eq!(layers, vec![0, 2, 31]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 0);
}

#[test]
fn inner_copper_only_resolves_to_that_inner_layer() {
    let t = table(&[("In1.Cu", 2), ("In2.Cu", 4)]);
    let layers = resolve_pad_copper_layers(&tokens(&["In1.Cu"]), &t).unwrap();
    assert_eq!(layers, vec![2]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 2);

    let layers = resolve_pad_copper_layers(&tokens(&["In2.Cu"]), &t).unwrap();
    assert_eq!(layers, vec![4]);
    assert_eq!(resolve_pad_primary_copper_layer(&layers).unwrap(), 4);
}

#[test]
fn non_copper_only_list_is_rejected() {
    let t = table(&[]);
    let err = resolve_pad_copper_layers(&tokens(&["F.Paste", "F.Mask"]), &t)
        .expect_err("non-copper (layers ...) must fail, not silently fall back");
    match err {
        EngineError::Import(msg) => {
            assert!(msg.contains("unsupported"), "message should flag unsupported encoding; got {msg}");
        }
        other => panic!("expected EngineError::Import, got {other:?}"),
    }
}

#[test]
fn unknown_dot_cu_name_without_table_entry_is_rejected() {
    let t = table(&[]);
    let err = resolve_pad_copper_layers(&tokens(&["Mystery.Cu"]), &t)
        .expect_err("unresolvable copper-looking name must not silently succeed");
    assert!(matches!(err, EngineError::Import(_)));
}

#[test]
fn empty_token_list_is_rejected() {
    let t = table(&[]);
    let err = resolve_pad_copper_layers(&tokens(&[]), &t)
        .expect_err("empty (layers ...) must not silently succeed");
    assert!(matches!(err, EngineError::Import(_)));
}

