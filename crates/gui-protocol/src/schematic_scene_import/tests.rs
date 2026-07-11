use super::*;
use uuid::Uuid;

#[test]
fn simple_kicad_schematic_projects_to_visible_review_scene() {
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic should load as a review scene");

    assert_eq!(state.scene.kind, "schematic_review_scene");
    assert_eq!(state.scene.board_name, "Sub");
    assert!(
        state
            .scene
            .layers
            .iter()
            .any(|layer| layer.kind == "schematic" && layer.visible_by_default)
    );
    assert!(
        state
            .scene
            .board_graphics
            .iter()
            .any(|graphic| graphic.object_id.starts_with("schematic-wire:")),
        "schematic wires should be visible through review-scene graphics"
    );
    assert!(
        state
            .scene
            .board_graphics
            .iter()
            .any(|graphic| graphic.object_id.starts_with("schematic-symbol:")),
        "schematic symbols should project an IEC rectangular body"
    );
}

#[test]
fn symbols_project_pins_terminals_and_annotation_text() {
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic should load as a review scene");
    let scene = &state.scene;

    // 2. Pin lines + terminal markers now project alongside the body, so the
    //    symbol reads as more than a bare box and wires meet real pins.
    assert!(
        scene
            .board_graphics
            .iter()
            .any(|g| g.object_id.starts_with("schematic-symbol-pin:")),
        "each symbol pin should project a pin line from body edge to terminal"
    );
    assert!(
        scene
            .board_graphics
            .iter()
            .any(|g| g.object_id.starts_with("schematic-symbol-pin-terminal:")),
        "each symbol pin should project a terminal marker at its wire attach point"
    );

    // 3/4. Refdes/value and pin name/number now render as real glyph text
    //      geometry (not discarded labels).
    assert!(
        scene
            .board_texts
            .iter()
            .any(|t| t.object_kind == "board_text"),
        "symbol annotation text (refdes/value/pin names) should project as real text"
    );
    assert!(
        !scene.board_text_geometries.is_empty(),
        "projected schematic text must carry renderable glyph geometry"
    );
    assert!(
        !scene.glyph_mesh_assets.is_empty(),
        "projected schematic text must carry glyph mesh assets for the world renderer"
    );
    // Text geometry must land on the per-role schematic text layers (P2.2c),
    // not the old single silk layer, so the renderer can colour refdes/value/
    // pin-name/pin-number distinctly.
    let text_layers: std::collections::BTreeSet<&str> = scene
        .board_text_geometries
        .iter()
        .map(|g| g.layer_id.as_str())
        .collect();
    assert!(
        !text_layers.contains("L44") && !text_layers.contains("L37"),
        "schematic text must no longer sit on the frame or the old F.SilkS layer"
    );
    assert!(
        text_layers.contains(&layer_id_string(SCHEMATIC_REFDES_TEXT_LAYER_INT).as_str())
            && text_layers
                .contains(&layer_id_string(SCHEMATIC_VALUE_TEXT_LAYER_INT).as_str()),
        "refdes and value text must carry their own per-role layers, got {text_layers:?}"
    );
}

#[test]
fn schematic_elements_carry_per_net_role_layers() {
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic should load as a review scene");
    let scene = &state.scene;

    // No schematic geometry may land on the old monochrome silk layer any
    // more; each net-role gets its own `Schematic.*` layer so the renderer
    // can resolve prototype token colours (green wires, grey symbols).
    assert!(
        scene
            .board_graphics
            .iter()
            .all(|g| g.layer_id != "L37"),
        "no schematic graphic may remain on the retired F.SilkS layer"
    );

    let layer_of = |prefix: &str| -> &str {
        scene
            .board_graphics
            .iter()
            .find(|g| g.object_id.starts_with(prefix))
            .unwrap_or_else(|| panic!("expected a {prefix} graphic"))
            .layer_id
            .as_str()
    };
    assert_eq!(
        layer_of("schematic-wire:"),
        SCHEMATIC_WIRE_LAYER,
        "wires must sit on the wire (green) layer"
    );
    assert_eq!(
        layer_of("schematic-symbol:"),
        SCHEMATIC_SYMBOL_LAYER,
        "symbol bodies must sit on the symbol (grey) layer"
    );

    // The scene layer table must register each role with its `Schematic.*`
    // name so the renderer's schematic colour path can key off it.
    let names: std::collections::BTreeSet<&str> =
        scene.layers.iter().map(|l| l.name.as_str()).collect();
    for expected in [
        "Schematic.Wire",
        "Schematic.Symbol",
        "Schematic.Junction",
        "Schematic.RefDes",
        "Schematic.Value",
        "Schematic.PinName",
        "Schematic.PinNumber",
    ] {
        assert!(
            names.contains(expected),
            "scene must register schematic role layer {expected}, got {names:?}"
        );
    }
    assert!(
        !names.contains("F.SilkS"),
        "the retired monochrome F.SilkS schematic layer must be gone"
    );
}

#[test]
fn schematic_projection_emits_no_sheet_frame() {
    // P2.2f: the schematic pane has no sheet border. The projection must not
    // emit the former gold `Edge.Cuts` (`L44`) padded-bounds outline, and the
    // scene layer table must no longer register that frame layer — while the
    // fit-to-bounds envelope (scene.bounds) stays a real, non-degenerate rect
    // covering the padded sheet extent.
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic should load as a review scene");
    let scene = &state.scene;

    assert!(
        scene
            .outline
            .iter()
            .all(|outline| outline.path.is_empty()),
        "schematic scene must emit no rendered sheet-frame outline path"
    );
    assert!(
        scene.layers.iter().all(|layer| layer.name != "Edge.Cuts"),
        "schematic scene must no longer register the Edge.Cuts frame layer"
    );
    assert!(
        scene
            .board_graphics
            .iter()
            .all(|g| g.layer_id != "L44"),
        "no schematic graphic may sit on the retired L44 frame layer"
    );
    assert!(
        scene.bounds.max_x > scene.bounds.min_x && scene.bounds.max_y > scene.bounds.min_y,
        "the schematic fit-to-bounds envelope must remain a real rect, got {:?}",
        scene.bounds
    );
}

#[test]
fn buses_project_gold_lines_and_diagonal_entries() {
    // P2.2e: a bus projects as a gold thick polyline on its own Schematic.Bus
    // layer, and its bus entries as diagonal green stubs. The bus-demo fixture
    // carries one bus (DATA) with two entries.
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/bus-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("bus-demo schematic should load as a review scene");
    let scene = &state.scene;

    let bus = scene
        .board_graphics
        .iter()
        .find(|g| g.object_id.starts_with("schematic-bus:"))
        .expect("bus should project a gold polyline");
    assert_eq!(
        bus.layer_id, SCHEMATIC_BUS_LAYER,
        "bus must sit on the Schematic.Bus (gold) layer"
    );
    assert!(
        bus.path.len() >= 2 && bus.width_nm == Some(BUS_STROKE_NM),
        "bus must carry its imported geometry at the thick bus stroke, got {bus:?}"
    );

    let entries: Vec<_> = scene
        .board_graphics
        .iter()
        .filter(|g| g.object_id.starts_with("schematic-bus-entry:"))
        .collect();
    assert_eq!(entries.len(), 2, "both bus entries should project");
    for entry in &entries {
        assert_eq!(
            entry.layer_id, SCHEMATIC_WIRE_LAYER,
            "bus entry stubs read as green member wires"
        );
        // The stub is diagonal: start != end in both axes (non-zero imported size).
        assert!(
            entry.path.len() == 2
                && entry.path[0].x != entry.path[1].x
                && entry.path[0].y != entry.path[1].y,
            "bus entry must be a diagonal stub, got {entry:?}"
        );
    }

    let names: std::collections::BTreeSet<&str> =
        scene.layers.iter().map(|l| l.name.as_str()).collect();
    assert!(
        names.contains("Schematic.Bus"),
        "scene must register the Schematic.Bus layer, got {names:?}"
    );
}

#[test]
fn global_labels_project_the_pentagon_tag() {
    // P2.2e: a Global net label projects as a pointed pentagon tag on the
    // Schematic.GlobalLabel (blue) layer, not a generic annotation rect. The
    // simple-demo fixture carries a global label (VCC) plus a local label (SCL)
    // that stays a chip.
    let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let state = load_kicad_schematic_workspace_state(&schematic)
        .expect("simple-demo schematic should load as a review scene");
    let scene = &state.scene;

    let pentagon = scene
        .board_graphics
        .iter()
        .find(|g| {
            g.object_id.starts_with("schematic-label:")
                && g.layer_id == SCHEMATIC_GLOBAL_LABEL_LAYER
        })
        .expect("the global label should project onto the GlobalLabel layer");
    assert_eq!(
        pentagon.primitive_kind, "polyline",
        "the pentagon tag is a stroked (not filled) polyline"
    );
    assert_eq!(
        pentagon.path.len(),
        6,
        "the pentagon tag has five vertices plus the closing point, got {}",
        pentagon.path.len()
    );

    let names: std::collections::BTreeSet<&str> =
        scene.layers.iter().map(|l| l.name.as_str()).collect();
    assert!(
        names.contains("Schematic.GlobalLabel"),
        "scene must register the Schematic.GlobalLabel layer, got {names:?}"
    );
}

#[test]
fn power_symbols_project_rail_flag_and_ground_stack_geometry() {
    // P2.2e: power symbols (`power:*` lib_ids) project a rail flag (stem + one
    // bar) or a ground stack (stem + three shrinking bars) on the Schematic.Power
    // layer, not a generic IEC box. No repo KiCad fixture carries power symbols,
    // so the projection is exercised directly over the engine Sheet model (this
    // constructs engine structs, not fabricated KiCad s-expressions).
    use eda_engine::ir::geometry::Point;
    use eda_engine::schematic::{
        HiddenPowerBehavior, PlacedSymbol, Schematic, Sheet, SymbolDisplayMode,
    };
    use std::collections::HashMap;

    let power_symbol = |uuid: Uuid, lib: &str, value: &str, x: i64| PlacedSymbol {
        uuid,
        part: None,
        entity: None,
        gate: None,
        lib_id: Some(lib.to_string()),
        reference: "#PWR".to_string(),
        value: value.to_string(),
        fields: Vec::new(),
        pins: Vec::new(),
        position: Point::new(x, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::new(),
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    };
    let gnd_uuid = Uuid::from_u128(0x9001);
    let rail_uuid = Uuid::from_u128(0x9002);
    let mut symbols = HashMap::new();
    symbols.insert(gnd_uuid, power_symbol(gnd_uuid, "power:GND", "GND", 0));
    symbols.insert(rail_uuid, power_symbol(rail_uuid, "power:+3V3", "+3V3", 5_000_000));

    let sheet = Sheet {
        uuid: Uuid::from_u128(0x1),
        name: "Root".to_string(),
        frame: None,
        symbols,
        wires: HashMap::new(),
        junctions: HashMap::new(),
        labels: HashMap::new(),
        buses: HashMap::new(),
        bus_entries: HashMap::new(),
        ports: HashMap::new(),
        noconnects: HashMap::new(),
        texts: HashMap::new(),
        drawings: HashMap::new(),
    };
    let schematic = Schematic {
        uuid: Uuid::from_u128(0x2),
        sheets: HashMap::new(),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let mut graphics = Vec::new();
    let mut points = Vec::new();
    let mut text = SchematicTextSink::default();
    push_root_sheet_graphics(&sheet, &schematic, &mut graphics, &mut points, &mut text);

    let power_lines: Vec<_> = graphics
        .iter()
        .filter(|g| g.object_id.starts_with("schematic-power:"))
        .collect();
    assert!(
        !power_lines.is_empty(),
        "power symbols must project power geometry, not a generic box"
    );
    assert!(
        power_lines
            .iter()
            .all(|g| g.layer_id == SCHEMATIC_POWER_LAYER),
        "all power geometry must sit on the Schematic.Power layer"
    );
    // No power symbol may fall through to the generic IEC symbol body.
    assert!(
        !graphics
            .iter()
            .any(|g| g.object_id.starts_with("schematic-symbol:")),
        "power symbols must not project a generic symbol body"
    );

    // Ground stack = stem + three bars (4 lines); rail flag = stem + one bar (2).
    let gnd_lines = power_lines
        .iter()
        .filter(|g| g.object_id.contains(&gnd_uuid.to_string()))
        .count();
    let rail_lines = power_lines
        .iter()
        .filter(|g| g.object_id.contains(&rail_uuid.to_string()))
        .count();
    assert_eq!(gnd_lines, 4, "ground = stem + three shrinking bars");
    assert_eq!(rail_lines, 2, "positive rail = stem + one bar");
}
