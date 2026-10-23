//! Existing pool materializers and canonical placed-object/component binding owners.
use super::*;
use eda_engine::{
    api::native_write::{
        board_components::{
            self, BoardPackagePlacement,
            pad_materialization::{
                ComponentPadMaterializationRequest, PadLandPatternSource,
                build_materialize_component_pad_sources,
            },
        },
        board_layout,
        schematic_library::materialize_pool_symbol_pins,
        schematic_symbols,
    },
    board::PlacedPackage,
    schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode},
    substrate::SourceShardKind,
};

pub(super) struct Placement {
    pub sheet: Uuid,
    pub symbol: PlacedSymbol,
    pub package: Uuid,
    pub instance: Uuid,
    pub pad: Uuid,
    pub pad_position: Point,
    pub library_pad: Uuid,
    pub map: Uuid,
    pub unlinked: Uuid,
}
fn pool(group: u128, n: u128) -> Uuid {
    Uuid::from_u128((group << 96) | 0x40008000000000000000 | n)
}
pub(super) fn build(model: &mut DesignModel, project: &Path) -> Result<Placement> {
    let sheet = pool(0x10000000, 4);
    let library_symbol = pool(0x30000000, 1).to_string();
    let mut pins = materialize_pool_symbol_pins(model, Some(&library_symbol))?;
    ensure!(pins.len() == 5, "complete baseline source pins");
    for (index, pin) in pins.iter_mut().enumerate() {
        pin.uuid = id(100 + index as u128);
    }
    ensure!(
        pins[0].library_pin == Some(pool(0x20000000, 0x101)),
        "independent anchor ordering"
    );
    let symbol = PlacedSymbol {
        uuid: id(1),
        part: Some(pool(0x90000000, 1)),
        entity: Some(pool(0x40000000, 1)),
        gate: Some(pool(0x40000000, 0x101)),
        lib_id: Some(library_symbol),
        reference: "UXP".into(),
        value: "native bound fixture".into(),
        fields: vec![],
        pins,
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let write =
        schematic_symbols::build_place_schematic_symbol(model, provenance(), sheet, &symbol, None)?;
    native_write::commit_prepared(model, project, write)?;
    let mut free = symbol.clone();
    free.uuid = id(5);
    free.reference = "UFREE".into();
    free.position.x = -10_000_000;
    for (index, pin) in free.pins.iter_mut().enumerate() {
        pin.uuid = id(110 + index as u128);
        pin.position.x -= 10_000_000;
    }
    let write =
        schematic_symbols::build_place_schematic_symbol(model, provenance(), sheet, &free, None)?;
    native_write::commit_prepared(model, project, write)?;
    let package = PlacedPackage {
        uuid: id(2),
        part: pool(0x90000000, 1),
        package: pool(0x50000000, 1),
        reference: "UXP".into(),
        value: "native bound fixture".into(),
        position: Point::new(10_000_000, 0),
        rotation: 0,
        layer: 1,
        locked: false,
    };
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let board_id = serde_json::from_value(board["uuid"].clone())?;
    let write = board_layout::source_profile::build_adopt_board_source_profile(
        model,
        provenance(),
        board_id,
    )?;
    native_write::commit_prepared(model, project, write)?;
    let map = pool(0xa0000000, 1);
    let write = board_components::build_place_board_package(
        model,
        provenance(),
        &BoardPackagePlacement {
            package: package.clone(),
            materialized: serde_json::json!({}),
        },
    )?;
    native_write::commit_prepared(model, project, write)?;
    // The generic bind helper pins only Part. This fixture also needs the exact
    // PinPadMap witness, authored through the same canonical typed operation as
    // the existing electrical binding fixture, never patched into source files.
    let instance = id(3);
    let write = BatchComposer::compose(model, provenance())
        .push_op(Operation::CreateComponentInstance {
            component_instance_id: instance,
            component_instance: serde_json::json!({
                "uuid":instance,"object_revision":0,
                "part_ref":reference(model, package.part)?,
                "placed_symbol_refs":[reference(model, symbol.uuid)?],
                "placed_package_refs":[reference(model, package.uuid)?],
                "placed_symbol_roles":{symbol.uuid.to_string():{"role":"primary"}},
                "placed_package_roles":{package.uuid.to_string():{"role":"primary"}},
                "library_bindings":{id(4).to_string():{
                    "target_object_id":map,"pinned_object_revision":model.objects[&map].object_revision,
                    "binding_role":"pin_pad_map"
                }}
            }),
        })
        .finish()?;
    native_write::commit_prepared(model, project, write)?;
    let request = ComponentPadMaterializationRequest {
        placement: package.clone(),
        land_pattern: PadLandPatternSource::Footprint(pool(0x60000000, 1)),
        component_instance: Some(instance),
        pin_pad_map: Some(map),
        correspondences: vec![],
        adopt_existing: false,
    };
    let write = build_materialize_component_pad_sources(model, provenance(), &request)?;
    native_write::commit_prepared(model, project, write)?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let library_pad = pool(0x80000000, 1);
    let pads = board["component_pads"][package.uuid.to_string()]
        .as_array()
        .context("source-qualified Pads")?;
    let matching: Vec<_> = pads
        .iter()
        .filter(|pad| {
            pad["source_binding"]["library_pad"]["object_id"] == serde_json::json!(library_pad)
        })
        .collect();
    // Use the explicit source binding below, never Pad display names or positions.
    let [pad] = matching.as_slice() else {
        anyhow::bail!("exact library Pad binding absent");
    };
    let pad_id = serde_json::from_value(pad["uuid"].clone())?;
    let local: Point = serde_json::from_value(pad["position"].clone())?;
    let pad_position = Point::new(local.x + package.position.x, local.y + package.position.y);
    Ok(Placement {
        sheet,
        symbol,
        package: package.uuid,
        instance,
        pad: pad_id,
        pad_position,
        library_pad,
        map,
        unlinked: free.uuid,
    })
}
