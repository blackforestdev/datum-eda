//! Wire/junction/label/bus/bus-entry/port/no-connect builders for the native
//! write facade.
//!
//! Family C of the native-write migration: all connectivity operation
//! authoring from
//! `crates/cli/src/command_project_schematic_connectivity_mutations.rs` lives
//! here. The CLI callers (direct mutations and the draft-proposal flow in
//! `command_project_schematic_proposals.rs`) are thin argument-parsers: they
//! load the target objects, call a `build_*` function, and either commit the
//! returned [`PreparedWrite`] via [`super::commit_prepared`] or feed its
//! uncommitted batch into `create_draft_proposal_from_batch`.
//!
//! Builders are build-only and take the engine's typed schematic objects;
//! payload shape, guard insertion, and batch stamping are byte-for-byte the
//! CLI's historical behavior.

use crate::error::EngineError;
use crate::schematic::{
    Bus, BusEntry, HierarchicalPort, Junction, NetLabel, NoConnectMarker, SchematicWire,
};
use crate::substrate::{DesignModel, ObjectId, Operation, SchematicMarkerKind};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Build the batch that places a net label on `sheet_id`.
pub fn build_create_schematic_label(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    label: &NetLabel,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicLabel {
            sheet_id,
            label_id: label.uuid,
            label: serde_json::to_value(label)?,
        })
        .primary_object(label.uuid)
        .finish()
}

/// Build the batch that rewrites an existing net label (revision guard
/// stamped automatically).
pub fn build_set_schematic_label(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    label: &NetLabel,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicLabel {
            sheet_id,
            label_id: label.uuid,
            label: serde_json::to_value(label)?,
        })
        .primary_object(label.uuid)
        .finish()
}

/// Build the batch that deletes an existing net label (revision guard
/// stamped automatically).
pub fn build_delete_schematic_label(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    label: &NetLabel,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicLabel {
            sheet_id,
            label_id: label.uuid,
            label: serde_json::to_value(label)?,
        })
        .primary_object(label.uuid)
        .finish()
}

/// Build the batch that draws a wire segment on `sheet_id`.
pub fn build_create_schematic_wire(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    wire: &SchematicWire,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicWire {
            sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        })
        .primary_object(wire.uuid)
        .finish()
}

/// Build the batch that deletes an existing wire segment.
pub fn build_delete_schematic_wire(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    wire: &SchematicWire,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicWire {
            sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        })
        .primary_object(wire.uuid)
        .finish()
}

/// Build the batch that places a junction on `sheet_id`.
pub fn build_create_schematic_junction(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    junction: &Junction,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicJunction {
            sheet_id,
            junction_id: junction.uuid,
            junction: serde_json::to_value(junction)?,
        })
        .primary_object(junction.uuid)
        .finish()
}

/// Build the normalized marker-placement batch required by the schematic
/// authoring contract. The marker kind selects the persisted sheet map while
/// keeping the journal vocabulary as one `PlaceSchematicMarker` operation.
pub fn build_place_schematic_marker(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    marker_id: ObjectId,
    kind: SchematicMarkerKind,
    marker: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::PlaceSchematicMarker {
            sheet_id,
            marker_id,
            marker_kind: kind,
            marker,
        })
        .primary_object(marker_id)
        .finish()
}

/// Build the batch that deletes an existing junction.
pub fn build_delete_schematic_junction(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    junction: &Junction,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id: junction.uuid,
            junction: serde_json::to_value(junction)?,
        })
        .primary_object(junction.uuid)
        .finish()
}

/// Build the batch that places a hierarchical port on `sheet_id`.
pub fn build_create_schematic_port(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    port: &HierarchicalPort,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicPort {
            sheet_id,
            port_id: port.uuid,
            port: serde_json::to_value(port)?,
        })
        .primary_object(port.uuid)
        .finish()
}

/// Build the batch that rewrites an existing hierarchical port (revision
/// guard stamped automatically).
pub fn build_set_schematic_port(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    port: &HierarchicalPort,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicPort {
            sheet_id,
            port_id: port.uuid,
            port: serde_json::to_value(port)?,
        })
        .primary_object(port.uuid)
        .finish()
}

/// Build the batch that deletes an existing hierarchical port (revision
/// guard stamped automatically).
pub fn build_delete_schematic_port(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    port: &HierarchicalPort,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicPort {
            sheet_id,
            port_id: port.uuid,
            port: serde_json::to_value(port)?,
        })
        .primary_object(port.uuid)
        .finish()
}

/// Build the batch that creates a bus on `sheet_id`.
pub fn build_create_schematic_bus(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    bus: &Bus,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicBus {
            sheet_id,
            bus_id: bus.uuid,
            bus: serde_json::to_value(bus)?,
        })
        .primary_object(bus.uuid)
        .finish()
}

/// Build the batch that rewrites an existing bus (revision guard stamped
/// automatically).
pub fn build_set_schematic_bus(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    bus: &Bus,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicBus {
            sheet_id,
            bus_id: bus.uuid,
            bus: serde_json::to_value(bus)?,
        })
        .primary_object(bus.uuid)
        .finish()
}

/// Build the batch that deletes an existing bus (revision guard stamped
/// automatically). Referential checks (e.g. remaining bus entries) are the
/// caller's responsibility, matching the CLI's historical validation seam.
pub fn build_delete_schematic_bus(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    bus: &Bus,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicBus {
            sheet_id,
            bus_id: bus.uuid,
            bus: serde_json::to_value(bus)?,
        })
        .primary_object(bus.uuid)
        .finish()
}

/// Build the batch that places a bus entry on `sheet_id`.
pub fn build_create_schematic_bus_entry(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    bus_entry: &BusEntry,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicBusEntry {
            sheet_id,
            bus_entry_id: bus_entry.uuid,
            bus_entry: serde_json::to_value(bus_entry)?,
        })
        .primary_object(bus_entry.uuid)
        .finish()
}

/// Build the batch that deletes an existing bus entry (revision guard
/// stamped automatically).
pub fn build_delete_schematic_bus_entry(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    bus_entry: &BusEntry,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicBusEntry {
            sheet_id,
            bus_entry_id: bus_entry.uuid,
            bus_entry: serde_json::to_value(bus_entry)?,
        })
        .primary_object(bus_entry.uuid)
        .finish()
}

/// Build the batch that places a no-connect marker on `sheet_id`.
pub fn build_create_schematic_noconnect(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    noconnect: &NoConnectMarker,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicNoConnect {
            sheet_id,
            noconnect_id: noconnect.uuid,
            noconnect: serde_json::to_value(noconnect)?,
        })
        .primary_object(noconnect.uuid)
        .finish()
}

/// Build the batch that deletes an existing no-connect marker.
pub fn build_delete_schematic_noconnect(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    noconnect: &NoConnectMarker,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicNoConnect {
            sheet_id,
            noconnect_id: noconnect.uuid,
            noconnect: serde_json::to_value(noconnect)?,
        })
        .primary_object(noconnect.uuid)
        .finish()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::Point;
    use crate::schematic::LabelKind;
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn fixture_sheet_id(model: &DesignModel) -> Uuid {
        Uuid::new_v5(&model.project.project_id, b"sheet")
    }

    #[test]
    fn create_label_matches_historical_batch_shape() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_connectivity_label");
        let sheet_id = fixture_sheet_id(&model);
        let label = NetLabel {
            uuid: Uuid::new_v4(),
            kind: LabelKind::Global,
            name: "NET1".to_string(),
            position: Point { x: 10, y: 20 },
        };

        let prepared = build_create_schematic_label(
            &model,
            test_provenance("place schematic label"),
            sheet_id,
            &label,
        )
        .expect("label create should build");

        assert_eq!(prepared.primary_object_id, Some(label.uuid));
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicLabel {
                sheet_id,
                label_id: label.uuid,
                label: serde_json::to_value(&label).expect("label should serialize"),
            }]
        );
    }

    #[test]
    fn set_and_delete_label_guard_the_label_object() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_connectivity_label_set");
        let sheet_id = fixture_sheet_id(&model);
        let mut label = NetLabel {
            uuid: Uuid::new_v4(),
            kind: LabelKind::Local,
            name: "NET1".to_string(),
            position: Point { x: 0, y: 0 },
        };
        let prepared = build_create_schematic_label(
            &model,
            test_provenance("place schematic label"),
            sheet_id,
            &label,
        )
        .expect("label create should build");
        commit_prepared(&mut model, &root, prepared).expect("label create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        label.name = "NET2".to_string();
        let prepared = build_set_schematic_label(
            &model,
            test_provenance("rename schematic label"),
            sheet_id,
            &label,
        )
        .expect("label set should build");
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: label.uuid,
                expected_object_revision: ObjectRevision(0),
            }
        );
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::SetSchematicLabel { label_id, .. } if *label_id == label.uuid
        ));

        let prepared = build_delete_schematic_label(
            &model,
            test_provenance("delete schematic label"),
            sheet_id,
            &label,
        )
        .expect("label delete should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == label.uuid
        ));
    }

    #[test]
    fn wire_and_junction_creates_are_unguarded_single_ops() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_connectivity_wire");
        let sheet_id = fixture_sheet_id(&model);
        let wire = SchematicWire {
            uuid: Uuid::new_v4(),
            from: Point { x: 0, y: 0 },
            to: Point { x: 10, y: 0 },
        };
        let junction = Junction {
            uuid: Uuid::new_v4(),
            position: Point { x: 10, y: 0 },
        };

        let prepared = build_create_schematic_wire(
            &model,
            test_provenance("draw schematic wire"),
            sheet_id,
            &wire,
        )
        .expect("wire create should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicWire {
                sheet_id,
                wire_id: wire.uuid,
                wire: serde_json::to_value(&wire).expect("wire should serialize"),
            }]
        );

        let prepared = build_create_schematic_junction(
            &model,
            test_provenance("place schematic junction"),
            sheet_id,
            &junction,
        )
        .expect("junction create should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicJunction {
                sheet_id,
                junction_id: junction.uuid,
                junction: serde_json::to_value(&junction).expect("junction should serialize"),
            }]
        );
    }

    #[test]
    fn bus_entry_delete_guards_the_entry_object() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_connectivity_bus_entry");
        let sheet_id = fixture_sheet_id(&model);
        let bus = Bus {
            uuid: Uuid::new_v4(),
            name: "DATA".to_string(),
            members: vec!["D0".to_string(), "D1".to_string()],
            segments: Vec::new(),
        };
        let prepared = build_create_schematic_bus(
            &model,
            test_provenance("create schematic bus"),
            sheet_id,
            &bus,
        )
        .expect("bus create should build");
        commit_prepared(&mut model, &root, prepared).expect("bus create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        let bus_entry = BusEntry {
            uuid: Uuid::new_v4(),
            bus: bus.uuid,
            wire: None,
            position: Point { x: 5, y: 5 },
            size: Point::zero(),
        };
        let prepared = build_create_schematic_bus_entry(
            &model,
            test_provenance("place schematic bus entry"),
            sheet_id,
            &bus_entry,
        )
        .expect("bus entry create should build");
        let mut model = model;
        commit_prepared(&mut model, &root, prepared).expect("bus entry create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        let prepared = build_delete_schematic_bus_entry(
            &model,
            test_provenance("delete schematic bus entry"),
            sheet_id,
            &bus_entry,
        )
        .expect("bus entry delete should build");
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: bus_entry.uuid,
                expected_object_revision: ObjectRevision(0),
            }
        );
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::DeleteSchematicBusEntry { bus_entry_id, .. }
                if *bus_entry_id == bus_entry.uuid
        ));
    }

    #[test]
    fn port_and_noconnect_builders_author_expected_operations() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_connectivity_port");
        let sheet_id = fixture_sheet_id(&model);
        let port = HierarchicalPort {
            uuid: Uuid::new_v4(),
            name: "CLK".to_string(),
            direction: crate::schematic::PortDirection::Input,
            position: Point { x: 1, y: 1 },
        };
        let noconnect = NoConnectMarker {
            uuid: Uuid::new_v4(),
            symbol: Uuid::new_v4(),
            pin: Uuid::new_v4(),
            position: Point { x: 2, y: 2 },
        };

        let prepared = build_create_schematic_port(
            &model,
            test_provenance("place schematic port"),
            sheet_id,
            &port,
        )
        .expect("port create should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicPort {
                sheet_id,
                port_id: port.uuid,
                port: serde_json::to_value(&port).expect("port should serialize"),
            }]
        );

        let prepared = build_create_schematic_noconnect(
            &model,
            test_provenance("place schematic noconnect"),
            sheet_id,
            &noconnect,
        )
        .expect("noconnect create should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicNoConnect {
                sheet_id,
                noconnect_id: noconnect.uuid,
                noconnect: serde_json::to_value(&noconnect).expect("noconnect should serialize"),
            }]
        );
    }
}
