//! XP-C01 preparation: explicit native electrical correspondence, never GUI proof.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, electrical},
    board::{Net, Track},
    ir::geometry::Point,
    schematic::{LabelKind, NetLabel, SchematicWire},
    substrate::{
        CommitSource, DesignModel, ElectricalIdentity, ElectricalIdentityRecord,
        ElectricalOccurrence, ElectricalSelectionSnapshot, NetCorrespondence,
        NetRelationshipIntent, ObjectRevision, Operation, ProjectResolver, RevisionedRef,
    },
};
use std::path::Path;
use uuid::Uuid;
#[path = "crossprobe/placement.rs"]
mod placement;
#[path = "crossprobe/verification.rs"]
mod verification;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x78000000000040008000000000000000 + n)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "crossprobe-xpc01-fixture",
        CommitSource::Test,
        "explicit native cross-domain correspondence preparation",
    )
}
fn occurrence(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn reference(model: &DesignModel, object_id: Uuid) -> Result<RevisionedRef> {
    Ok(RevisionedRef {
        object_id,
        object_revision: model
            .objects
            .get(&object_id)
            .context("pinned source object")?
            .object_revision,
    })
}

pub(super) fn build(root: &Path) -> Result<()> {
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh owned output required"
    );
    let project = root.join("project");
    super::library::prepare(&project)?;
    let mut model = ProjectResolver::new(&project).resolve()?;
    let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None)
            .map_err(|e| anyhow::anyhow!("existing Project Units migration: {e:?}"))?
    else {
        anyhow::bail!("fresh scaffold must require canonical Working Units initialization");
    };
    let write = native_write::project::build_initialize_project_display_units(
        &model,
        provenance(),
        profile,
        &receipt,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    let placed = placement::build(&mut model, &project)?;
    let pin = &placed.symbol.pins[0];
    let endpoint = Point::new(pin.position.x + 8_000_000, pin.position.y);
    let wire = SchematicWire {
        uuid: id(30),
        from: pin.position,
        to: endpoint,
    };
    let label = NetLabel {
        uuid: id(31),
        kind: LabelKind::Global,
        name: "XP_SIGNAL".into(),
        position: endpoint,
    };
    let board_net = Net::new(id(32), "XP_SIGNAL", Uuid::nil());
    let foreign = Net::new(id(33), "XP_SIGNAL", Uuid::nil());
    let tracks = [
        Track::straight(
            id(34),
            board_net.uuid,
            placed.pad_position,
            Point::new(2_000_000, placed.pad_position.y),
            300_000,
            1,
        ),
        Track::straight(
            id(35),
            board_net.uuid,
            Point::new(18_000_000, 0),
            Point::new(21_000_000, 0),
            300_000,
            1,
        ),
        Track::straight(
            id(36),
            foreign.uuid,
            Point::new(8_000_000, 3_000_000),
            Point::new(15_000_000, 3_000_000),
            300_000,
            1,
        ),
    ];
    let mut operations = vec![
        Operation::CreateSchematicWire {
            sheet_id: placed.sheet,
            wire_id: wire.uuid,
            wire: serde_json::to_value(&wire)?,
        },
        Operation::CreateSchematicLabel {
            sheet_id: placed.sheet,
            label_id: label.uuid,
            label: serde_json::to_value(&label)?,
        },
    ];
    for net in [&board_net, &foreign] {
        operations.push(Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(net)?,
        });
    }
    for track in &tracks {
        operations.push(Operation::CreateBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track)?,
        });
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(operations)
        .finish()?;
    native_write::commit_prepared(&mut model, &project, write)?;
    let write = electrical::build_adopt_schematic_net_identities(&model, provenance())?;
    native_write::commit_prepared(&mut model, &project, write)?;
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let pin_origin = occurrence("pins", pin.uuid);
    let logical = snapshot
        .schematic_run(snapshot.revision(), &pin_origin)
        .map_err(|e| anyhow::anyhow!("logical owner: {e:?}"))?
        .subject_id;
    let relation = ElectricalIdentityRecord {
        id: id(40),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: logical,
            board_net: Some(reference(&model, board_net.uuid)?),
            intent: NetRelationshipIntent::Implemented,
            evidence: vec![NetCorrespondence {
                component_instance: reference(&model, placed.instance)?,
                pin_pad_map: reference(&model, placed.map)?,
                schematic_terminal: pin_origin,
                board_pad: reference(&model, placed.pad)?,
                library_pad: Some(reference(&model, placed.library_pad)?),
                library_pin: Some(reference(
                    &model,
                    pin.library_pin.context("source library pin")?,
                )?),
            }],
        },
    };
    let write = electrical::build_adopt_pin_correspondence(
        &model,
        provenance(),
        relation,
        &[electrical::PlacedPinAdoption {
            sheet_id: placed.sheet,
            symbol_id: placed.symbol.uuid,
            placed_pin_id: pin.uuid,
            library_pin_id: pin.library_pin.unwrap(),
        }],
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    verification::check(
        root, &project, &mut model, &placed, logical, &wire, &label, &tracks,
    )
}
