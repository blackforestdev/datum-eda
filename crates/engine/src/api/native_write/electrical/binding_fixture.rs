use super::super::{
    board_routing::{build_place_board_net, build_place_board_pad},
    commit_prepared,
    library::{
        PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
        pool_part_payload,
    },
    schematic_symbols::build_place_schematic_symbol,
};
use super::tests::{create, net, provenance};
use super::*;
use crate::ir::geometry::Point;
use crate::schematic::{
    HiddenPowerBehavior, PinElectricalType, PlacedSymbol, SymbolDisplayMode, SymbolPin,
};
use crate::substrate::{
    ElectricalOccurrence, NetCorrespondence, NetRelationshipIntent, ProjectResolver, RevisionedRef,
};
use serde_json::json;
use uuid::Uuid;
fn reference(id: Uuid) -> RevisionedRef {
    RevisionedRef {
        object_id: id,
        object_revision: ObjectRevision(0),
    }
}

pub(super) struct BindingFixture {
    pub root: std::path::PathBuf,
    pub model: DesignModel,
    pub sheet: Uuid,
    pub schematic: Uuid,
    pub symbol: PlacedSymbol,
    pub logical: Uuid,
    pub relationship: ElectricalIdentityRecord,
    pub library_pin: Uuid,
}

pub(super) fn fixture(name: &str) -> BindingFixture {
    let root = super::super::test_support::temp_project_root(name);
    let genesis = super::super::genesis::bootstrap_native_project(
        &root,
        super::super::genesis::GenesisSpec {
            project_name: "E1 binding proof".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = Uuid::new_v4();
    let prepared = super::super::schematic_sheets::build_create_schematic_sheet(&model, provenance(), genesis.schematic_uuid, sheet, "sheets/main.json", json!({"schema_version":1,"uuid":sheet,"name":"Main","symbols":{},"wires":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"junctions":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let package = Uuid::new_v4();
    let ids: Vec<_> = (0..9).map(|_| Uuid::new_v4()).collect();
    let (part, entity, unit, gate, library_package, library_pad, library_pin, map_id, placed_pin) = (
        ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7], ids[8],
    );
    let objects = vec![
        (
            "packages",
            library_package,
            json!({"uuid":library_package,"name":"PACKAGE","pads":{library_pad.to_string():{"uuid":library_pad,"name":"unrelated pad display","position":{"x":0,"y":0},"padstack":Uuid::new_v4(),"layer":1}}}),
        ),
        (
            "units",
            unit,
            json!({"uuid":unit,"name":"UNIT","manufacturer":"","tags":[],"pins":{library_pin.to_string():{"uuid":library_pin,"name":"unrelated pin display","direction":"Passive","swap_group":0,"alternates":[]}}}),
        ),
        (
            "entities",
            entity,
            json!({"uuid":entity,"name":"ENTITY","prefix":"U","manufacturer":"","tags":[],"gates":{gate.to_string():{"uuid":gate,"name":"G","unit":unit,"symbol":Uuid::new_v4()}}}),
        ),
        (
            "parts",
            part,
            pool_part_payload(
                part,
                entity,
                library_package,
                "MPN",
                "M",
                "V",
                "D",
                "DS",
                "Active",
            ),
        ),
        (
            "pin_pad_maps",
            map_id,
            json!({"uuid":map_id,"part":part,"mappings":{library_pad.to_string():{"gate":gate,"pin":library_pin}},"tags":[]}),
        ),
    ];
    let prepared = build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        objects
            .into_iter()
            .map(|(kind, id, mut object)| {
                object["schema_version"] = json!(1);
                PoolLibraryOperationSpec::Create {
                    target: PoolLibraryObjectTarget::new("pool", kind, id),
                    object,
                }
            })
            .collect(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let prepared = BatchComposer::compose(&model, provenance()).push_op(Operation::CreateBoardPackage { package_id: package, package: json!({"uuid":package,"part":part,"package":library_package,"reference":"U1","value":"","position":{"x":0,"y":0},"rotation":0,"layer":1,"locked":false}), materialized: json!({}) }).finish().unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: Some(part),
        entity: Some(entity),
        gate: Some(gate),
        lib_id: None,
        reference: "U1".into(),
        value: "".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: placed_pin,
            library_pin: None,
            number: "nonsense".into(),
            name: "nonsense".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point::new(0, 0),
        }],
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let prepared =
        build_place_schematic_symbol(&model, provenance(), sheet, &symbol, None).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let board_net = Uuid::new_v4();
    let net_data = crate::board::Net::new(board_net, "unrelated net display", Uuid::nil());
    let prepared = build_place_board_net(&model, provenance(), &net_data).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let pad:crate::board::PlacedPad=serde_json::from_value(json!({"uuid":Uuid::new_v4(),"package":package,"name":"nonsense","net":board_net,"position":{"x":0,"y":0},"layer":1})).unwrap();
    let prepared = build_place_board_pad(&model, provenance(), &pad).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let ci = Uuid::new_v4();
    let prepared=BatchComposer::compose(&model,provenance()).push_op(Operation::CreateComponentInstance { component_instance_id:ci,component_instance:json!({"uuid":ci,"object_revision":0,"part_ref":{"object_id":part,"object_revision":0},"placed_symbol_refs":[reference(symbol.uuid)],"placed_package_refs":[{"object_id":package,"object_revision":model.objects[&package].object_revision}],"placed_symbol_roles":{symbol.uuid.to_string():{"role":"primary"}},"placed_package_roles":{package.to_string():{"role":"primary"}},"library_bindings":{Uuid::new_v4().to_string():{"target_object_id":map_id,"pinned_object_revision":0,"binding_role":"pin_pad_map"}}}) }).finish().unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let logical = Uuid::new_v4();
    create(
        &mut model,
        &root,
        net(
            logical,
            ElectricalOccurrence {
                class: "pins".into(),
                source_id: placed_pin,
                instance_path: vec![],
            },
        ),
    );
    let relationship = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: logical,
            board_net: Some(reference(board_net)),
            intent: NetRelationshipIntent::Implemented,
            evidence: vec![NetCorrespondence {
                component_instance: reference(ci),
                pin_pad_map: reference(map_id),
                schematic_terminal: ElectricalOccurrence {
                    class: "pins".into(),
                    source_id: placed_pin,
                    instance_path: vec![],
                },
                board_pad: reference(pad.uuid),
                library_pad: Some(reference(library_pad)),
                library_pin: None,
            }],
        },
    };
    BindingFixture {
        root,
        model,
        sheet,
        schematic: genesis.schematic_uuid,
        symbol,
        logical,
        relationship,
        library_pin,
    }
}

// Exact authored shard bytes: refusal must preserve disk, not just live caches.
pub(super) fn authored_bytes(
    root: &std::path::Path,
    model: &DesignModel,
) -> std::collections::BTreeMap<String, Vec<u8>> {
    model
        .source_shards
        .iter()
        .map(|shard| {
            (
                shard.relative_path.clone(),
                std::fs::read(root.join(&shard.relative_path)).unwrap(),
            )
        })
        .collect()
}
