//! Persisted wire/Bus/hierarchy source cases through existing canonical owners.
use anyhow::{Context, Result};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, electrical, schematic_sheets},
    ir::geometry::Point,
    schematic::{
        Bus, BusEntry, HierarchicalPort, Junction, LabelKind, NetLabel, PortDirection,
        SchematicWire, SheetDefinition, SheetInstance,
    },
    substrate::{
        BusInterfaceEquivalence, CommitSource, ElectricalIdentity, ElectricalIdentityRecord,
        ElectricalOccurrence, ElectricalSelectionSnapshot, NetRelationshipIntent, ObjectRevision,
        Operation, ProjectResolver,
    },
};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;
#[path = "semantic/checks.rs"]
mod checks;
#[path = "semantic/fork.rs"]
mod fork;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x73000000000040008000000000000000 + n)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-semantic-fixture",
        CommitSource::Test,
        "existing native semantic Bus and occurrence sources",
    )
}
fn occurrence(class: &str, n: u128, path: &[Uuid]) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id(n),
        instance_path: path.into(),
    }
}
fn reps(path: &[Uuid], members: &[u128]) -> BTreeSet<ElectricalOccurrence> {
    members
        .iter()
        .map(|&n| occurrence("buses", n, path))
        .collect()
}
pub(super) fn build(root: &Path) -> Result<serde_json::Value> {
    let path = root.join("variants/semantic-scope");
    super::lifecycle::copy_project(root, &path)?;
    let mut model = ProjectResolver::new(&path).resolve()?;
    let snap = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let sheet = snap
        .schematic_source()
        .context("native source")?
        .sheets
        .values()
        .next()
        .unwrap()
        .uuid;
    let mut ops = vec![];
    for (n, a, b) in [
        (1, 0, 20_000_000),
        (2, 20_000_000, 40_000_000),
        (3, 70_000_000, 90_000_000),
        (4, 40_000_000, 60_000_000),
    ] {
        let bus = Bus {
            uuid: id(n),
            name: "same declaration name".into(),
            members: vec![],
            segments: vec![Point::new(a, 10_000_000), Point::new(b, 10_000_000)],
        };
        ops.push(Operation::CreateSchematicBus {
            sheet_id: sheet,
            bus_id: bus.uuid,
            bus: serde_json::to_value(bus)?,
        });
    }
    let entry = BusEntry {
        uuid: id(5),
        bus: id(2),
        wire: None,
        position: Point::new(30_000_000, 10_000_000),
        size: Point::new(5_000_000, 5_000_000),
    };
    ops.push(Operation::CreateSchematicBusEntry {
        sheet_id: sheet,
        bus_entry_id: entry.uuid,
        bus_entry: serde_json::to_value(entry)?,
    });
    for (n, a, b) in [
        (8, 0, 10_000_000),
        (9, 10_000_000, 20_000_000),
        (13, 50_000_000, 60_000_000),
    ] {
        let wire = SchematicWire {
            uuid: id(n),
            from: Point::new(a, 0),
            to: Point::new(b, 0),
        };
        ops.push(Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        });
    }
    for (n, x) in [(10, 20_000_000), (14, 60_000_000)] {
        let label = NetLabel {
            uuid: id(n),
            kind: LabelKind::Global,
            name: "same global label".into(),
            position: Point::new(x, 0),
        };
        ops.push(Operation::CreateSchematicLabel {
            sheet_id: sheet,
            label_id: label.uuid,
            label: serde_json::to_value(label)?,
        });
    }
    let junction = Junction {
        uuid: id(11),
        position: Point::new(10_000_000, 0),
    };
    ops.push(Operation::CreateSchematicJunction {
        sheet_id: sheet,
        junction_id: junction.uuid,
        junction: serde_json::to_value(junction)?,
    });
    let port = HierarchicalPort {
        uuid: id(12),
        name: "port".into(),
        direction: PortDirection::Bidirectional,
        position: Point::new(20_000_000, 0),
    };
    ops.push(Operation::CreateSchematicPort {
        sheet_id: sheet,
        port_id: port.uuid,
        port: serde_json::to_value(port)?,
    });
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .finish()?;
    native_write::commit_prepared(&mut model, &path, write)?;
    let schematic = model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::SchematicRoot)?;
    let schematic_id = Uuid::parse_str(schematic["uuid"].as_str().unwrap())?;
    let child = fork::create(&mut model, &path, schematic_id, sheet)?;
    let definition = SheetDefinition {
        uuid: id(30),
        root_sheet: child,
        name: "same definition".into(),
    };
    let write = schematic_sheets::build_create_schematic_definition(
        &model,
        provenance(),
        schematic_id,
        definition.uuid,
        "definitions/semantic.json",
        serde_json::to_value(definition)?,
    )?;
    native_write::commit_prepared(&mut model, &path, write)?;
    for n in [31, 32] {
        let instance = SheetInstance {
            uuid: id(n),
            definition: id(30),
            parent_sheet: None,
            name: "same occurrence name".into(),
            position: Point::zero(),
            ports: vec![],
        };
        let write = schematic_sheets::build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic_id,
            instance.uuid,
            serde_json::to_value(instance)?,
        )?;
        native_write::commit_prepared(&mut model, &path, write)?;
    }
    let write = electrical::build_adopt_schematic_net_identities(&model, provenance())?;
    native_write::commit_prepared(&mut model, &path, write)?;
    let snap = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let local = snap
        .schematic_run(&model.model_revision, &occurrence("wires", 8, &[]))
        .map_err(|e| anyhow::anyhow!("actual logical owner: {e:?}"))?;
    let net = local.subject_id;
    let mut primary = reps(&[], &[1, 2, 3]);
    primary.extend(reps(&[id(31)], &[101, 102, 103]));
    let mut foreign = reps(&[], &[4]);
    foreign.extend(reps(&[id(31)], &[104]));
    let mut records = vec![ElectricalIdentityRecord {
        id: id(40),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: net,
            board_net: None,
            intent: NetRelationshipIntent::SchematicOnly,
            evidence: vec![],
        },
    }];
    for (n, representations) in [
        (41, primary),
        (42, foreign),
        (43, reps(&[id(32)], &[101, 102, 103])),
        (44, reps(&[id(32)], &[104])),
    ] {
        records.push(ElectricalIdentityRecord {
            id: id(n),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Bus {
                name: "same declaration name".into(),
                scalar_nets: BTreeSet::from([net]),
                representations,
                retired: false,
            },
        });
    }
    for (n, right_bus, right_occurrence, equivalence) in [
        (
            45,
            id(41),
            occurrence("buses", 101, &[id(31)]),
            BusInterfaceEquivalence::Identity,
        ),
        (
            46,
            id(43),
            occurrence("buses", 101, &[id(32)]),
            BusInterfaceEquivalence::Related,
        ),
    ] {
        records.push(ElectricalIdentityRecord {
            id: id(n),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::BusInterface {
                left_bus: id(41),
                right_bus,
                left_occurrence: occurrence("buses", 1, &[]),
                right_occurrence,
                equivalence,
                scalar_mapping: BTreeSet::from([(net, net)]),
            },
        });
    }
    for record in records {
        let write = electrical::build_create_electrical_identity(&model, provenance(), record)?;
        native_write::commit_prepared(&mut model, &path, write)?;
    }
    checks::check(&path, net)
}
