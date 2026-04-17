use super::*;

fn pad_rotation_by_reference_and_name(
    board: &crate::board::Board,
    reference: &str,
    pad_name: &str,
) -> i32 {
    let package_uuid = board
        .packages
        .values()
        .find(|pkg| pkg.reference == reference)
        .unwrap_or_else(|| panic!("package {reference} should exist"))
        .uuid;
    board
        .pads
        .values()
        .find(|pad| pad.package == package_uuid && pad.name == pad_name)
        .unwrap_or_else(|| panic!("pad {reference}.{pad_name} should exist"))
        .rotation
}

#[test]
fn datum_test_front_side_rotated_pad_imports_world_rotation() {
    let Some(path) = optional_datum_test_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("datum-test board should parse");

    // KiCad PCB pad `(at ... rot)` is already emitted with the authored board
    // pad angle even though its center remains footprint-local. Q5 pad 1 is
    // written as 90 degrees in the board file and should be preserved exactly.
    assert_eq!(pad_rotation_by_reference_and_name(&board, "Q5", "1"), 90);
}

#[test]
fn doa2526_rotated_footprint_unrotated_pad_imports_world_rotation() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("DOA2526 board should parse");

    // Q1 is placed at 0 degrees and its S-MINI_TOS pads are unrotated locally.
    // Final placed-pad rotation stays at 0 degrees in board space.
    assert_eq!(pad_rotation_by_reference_and_name(&board, "Q1", "1"), 0);
}

#[test]
fn doa2526_rotated_footprint_and_rotated_pad_import_world_rotation() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("DOA2526 board should parse");

    // DOA2526 proves the same rule on a differently rotated footprint: the
    // board-file pad angle itself is the final authored pad orientation.
    assert_eq!(pad_rotation_by_reference_and_name(&board, "R1", "1"), -90);
}
