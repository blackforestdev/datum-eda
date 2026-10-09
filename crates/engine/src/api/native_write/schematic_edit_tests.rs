use super::*;
use crate::api::native_write::{commit_prepared, electrical::binding_fixture::fixture};
use crate::substrate::{CommitSource, ProjectResolver};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "native wire gesture")
}
fn p(x: i64, y: i64) -> Point {
    Point { x, y }
}

#[test]
fn native_wire_t_contact_is_atomic_and_survives_undo_redo_reopen() {
    let mut f = fixture("native_wire_t_gesture");
    let sheet = schematic(&f.model).unwrap().sheets[&f.sheet].clone();
    let first =
        build_wire_gesture(&f.model, provenance(), &sheet, &[p(-1000, 0), p(1000, 0)]).unwrap();
    commit_prepared(&mut f.model, &f.root, first).unwrap();
    let sheet = schematic(&f.model).unwrap().sheets[&f.sheet].clone();
    let second = build_wire_gesture(
        &f.model,
        provenance(),
        &sheet,
        &[p(0, 0), p(0, 1000), p(1000, 1000)],
    )
    .unwrap();
    assert_eq!(
        second
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::CreateSchematicWire { .. }))
            .count(),
        2
    );
    assert_eq!(
        second
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::PlaceSchematicMarker { .. }))
            .count(),
        1
    );
    commit_prepared(&mut f.model, &f.root, second).unwrap();
    let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    let sheet = schematic(&reopened).unwrap().sheets[&f.sheet].clone();
    assert_eq!(sheet.wires.len(), 3);
    assert_eq!(sheet.junctions.len(), 1);
    let junction = sheet.junctions.values().next().unwrap();
    assert_ne!(junction.uuid, Uuid::nil());
    assert_eq!(junction.position, p(0, 0));
    reopened
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let mut undone = ProjectResolver::new(&f.root).resolve().unwrap();
    let sheet = schematic(&undone).unwrap().sheets[&f.sheet].clone();
    assert_eq!(sheet.wires.len(), 1);
    assert!(sheet.junctions.is_empty());
    undone
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let redo = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        schematic(&redo).unwrap().sheets[&f.sheet]
            .junctions
            .values()
            .next()
            .unwrap(),
        junction
    );
    std::fs::remove_dir_all(&f.root).unwrap();
}

#[test]
fn wire_contact_is_exact_at_integer_extremes_and_excludes_noncontacts() {
    assert!(strictly_on_segment(p(0, 0), p(i64::MIN, i64::MIN), p(i64::MAX, i64::MAX)).unwrap());
    assert!(!strictly_on_segment(p(0, 1), p(-1000, 0), p(1000, 0)).unwrap());
    assert!(!strictly_on_segment(p(1000, 0), p(-1000, 0), p(1000, 0)).unwrap());
}

#[test]
fn native_symbol_uses_existing_library_binding_and_absolute_pin_positions() {
    use crate::api::native_write::library::{
        PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
    };
    let mut f = fixture("native_symbol_placement");
    let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut objects = Vec::new();
    for kind in [
        "units",
        "symbols",
        "entities",
        "packages",
        "padstacks",
        "footprints",
        "parts",
        "pin_pad_maps",
    ] {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: serde_json::Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            let id = object["uuid"].as_str().unwrap().parse().unwrap();
            objects.push(PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", kind, id),
                object,
            });
        }
    }
    let prepared = build_pool_library_write(&f.model, provenance(), Some("pool"), objects).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let context = NativeWriteContext {
        model: &f.model,
        project_root: &f.root,
    };
    let params = serde_json::json!({"sheet":f.sheet,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U2","value":"Op amp","x_nm":10_000_000,"y_nm":20_000_000});
    let prepared = place_symbol(&context, provenance(), params.clone()).unwrap();
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::CreateComponentInstance { .. }))
            .count(),
        1
    );
    let id = prepared.primary_object_id.unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    let source = schematic(&reopened).unwrap();
    let placed = &source.sheets[&f.sheet].symbols[&id];
    assert_eq!(placed.pins.len(), 5);
    let pin = placed
        .pins
        .iter()
        .find(|pin| {
            pin.library_pin == Some("20000000-0000-4000-8000-000000000101".parse().unwrap())
        })
        .unwrap();
    assert_eq!(pin.position, p(11_500_000, 20_000_000));
    assert!(placed.pins.iter().all(|pin| pin.uuid != Uuid::nil()));
    assert!(
        place_symbol(
            &NativeWriteContext {
                model: &reopened,
                project_root: &f.root
            },
            provenance(),
            params
        )
        .is_err()
    );
    std::fs::remove_dir_all(&f.root).unwrap();
}
