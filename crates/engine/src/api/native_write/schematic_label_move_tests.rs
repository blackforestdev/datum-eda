use super::*;
use crate::api::native_write::{
    commit_prepared, electrical::binding_fixture::fixture, schematic_connectivity::*,
};
use crate::schematic::LabelKind;
use crate::substrate::{CommitSource, DesignModel, ProjectResolver, electrical_test_partitions};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "move native Label")
}
fn build(
    model: &DesignModel,
    root: &Path,
    label: Uuid,
    at: Point,
) -> Result<PreparedWrite, EngineError> {
    move_label(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"label":label,"instance_path":[],"x_nm":at.x,"y_nm":at.y}),
    )
}
fn label(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    at: Point,
    kind: LabelKind,
) -> NetLabel {
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        kind,
        name: "GND".into(),
        position: at,
    };
    let write = build_create_schematic_label(model, provenance(), sheet, &label).unwrap();
    commit_prepared(model, root, write).unwrap();
    label
}
fn wire(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    from: Point,
    to: Point,
) -> SchematicWire {
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from,
        to,
    };
    let write = build_create_schematic_wire(model, provenance(), sheet, &wire).unwrap();
    commit_prepared(model, root, write).unwrap();
    wire
}
fn source(model: &DesignModel) -> crate::schematic::Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}

#[test]
fn connected_power_move_preserves_all_endpoints_identity_and_single_step_history() {
    let mut f = fixture("connected_power_move_history");
    let label = label(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(1000, 0),
        LabelKind::Power,
    );
    let a = wire(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(0, 0),
        label.position,
    );
    let b = wire(
        &mut f.model,
        &f.root,
        f.sheet,
        label.position,
        Point::new(1000, -1000),
    );
    let before = source(&f.model);
    let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
    let identities = f.model.electrical_identities.clone();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = LabelMoveBasis::capture(&snapshot, label.uuid, &[]).unwrap();
    let at = Point::new(1200, 500);
    let destination = basis.destination_edit(at).unwrap();
    assert_eq!(destination.label.uuid, label.uuid);
    assert_eq!(destination.label.name, "GND");
    assert_eq!(destination.label.kind, LabelKind::Power);
    assert_eq!(destination.wires.len(), 2);
    for w in &destination.wires {
        if w.uuid == a.uuid {
            assert_eq!((w.from, w.to), (a.from, at));
        } else {
            assert_eq!(w.uuid, b.uuid);
            assert_eq!((w.from, w.to), (at, b.to));
        }
    }
    assert_eq!(
        source(&ProjectResolver::new(&f.root).resolve().unwrap()),
        before,
        "preview does not write"
    );
    let write = build(&f.model, &f.root, label.uuid, at).unwrap();
    assert_eq!(
        write
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::SetSchematicLabel { .. }))
            .count(),
        1
    );
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    let after = source(&reopened);
    assert_eq!(
        after.sheets[&f.sheet].labels[&label.uuid],
        destination.label
    );
    for w in destination.wires {
        assert_eq!(after.sheets[&f.sheet].wires[&w.uuid], w);
    }
    assert_eq!(
        electrical_test_partitions(&reopened, &[]).unwrap(),
        partitions
    );
    assert_eq!(reopened.electrical_identities, identities);
    let findings = |s: &crate::schematic::Schematic| {
        let mut rows: Vec<_> = crate::erc::run_prechecks(s)
            .into_iter()
            .map(|f| (f.code, f.component, f.pin, f.object_uuids))
            .collect();
        rows.sort();
        rows
    };
    assert_eq!(
        findings(&before),
        findings(&after),
        "Power label Move never supplies ERC drive"
    );
    reopened
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(source(&reopened), before);
    reopened
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(
        source(&ProjectResolver::new(&f.root).resolve().unwrap()),
        after
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn label_move_refuses_ambiguous_contacts_collapses_and_stale_writes() {
    let mut f = fixture("power_move_refusals");
    let moving = label(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(1000, 0),
        LabelKind::Power,
    );
    wire(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(0, 0),
        moving.position,
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    assert!(LabelMoveBasis::capture(&snapshot, moving.uuid, &[Uuid::new_v4()]).is_err());
    let basis = LabelMoveBasis::capture(&snapshot, moving.uuid, &[]).unwrap();
    assert!(
        basis.destination_edit(Point::new(0, 0)).is_err(),
        "cannot collapse onto fixed pin"
    );
    let prepared = build(&f.model, &f.root, moving.uuid, Point::new(1000, 500)).unwrap();
    label(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(1000, 500),
        LabelKind::Local,
    );
    let before = source(&f.model);
    let revision = f.model.model_revision.clone();
    assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
    assert!(
        build(&f.model, &f.root, moving.uuid, Point::new(1000, 500)).is_err(),
        "recapture sees destination contact"
    );
    assert_eq!(f.model.model_revision, revision);
    assert_eq!(
        source(&ProjectResolver::new(&f.root).resolve().unwrap()),
        before
    );
    let interior = label(
        &mut f.model,
        &f.root,
        f.sheet,
        Point::new(500, 0),
        LabelKind::Global,
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    assert!(
        LabelMoveBasis::capture(&snapshot, interior.uuid, &[]).is_err(),
        "wire interior has no unique movable endpoint"
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn isolated_scalar_label_kinds_share_move_without_renaming_or_retyping() {
    for kind in [
        LabelKind::Local,
        LabelKind::Global,
        LabelKind::Hierarchical,
        LabelKind::Power,
    ] {
        let mut f = fixture("isolated_label_move");
        let original = label(&mut f.model, &f.root, f.sheet, Point::new(1000, 0), kind);
        let at = Point::new(i64::MAX, i64::MIN);
        let prepared = build(&f.model, &f.root, original.uuid, at).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let mut expected = original.clone();
        expected.position = at;
        assert_eq!(
            source(&f.model).sheets[&f.sheet].labels[&original.uuid],
            expected
        );
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
