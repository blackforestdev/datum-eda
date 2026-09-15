use super::*;
use crate::ir::geometry::Point;

#[path = "mod_tests_ipc_footprint.rs"]
mod ipc_footprint;
#[path = "mod_tests_library_graph.rs"]
mod library_graph;
#[path = "mod_tests_library_graph_default_pin_pad_map.rs"]
mod library_graph_default_pin_pad_map;

fn sample_pin() -> (Uuid, Pin) {
    let id = Uuid::nil();
    (
        id,
        Pin {
            uuid: id,
            name: "1".into(),
            direction: PinDirection::Input,
            swap_group: 0,
            alternates: vec![AlternateName {
                name: "A".into(),
                kind: "legacy".into(),
            }],
        },
    )
}

fn sample_unit() -> Unit {
    let (pin_id, pin) = sample_pin();
    let mut pins = HashMap::new();
    pins.insert(pin_id, pin);

    Unit {
        uuid: Uuid::from_u128(1),
        name: "TL072".into(),
        manufacturer: "TI".into(),
        pins,
        tags: HashSet::from(["opamp".to_string(), "dual".to_string()]),
    }
}

fn sample_package() -> Package {
    let pad_id = Uuid::from_u128(10);
    let mut pads = HashMap::new();
    pads.insert(
        pad_id,
        Pad {
            uuid: pad_id,
            name: "1".into(),
            position: Point::new(0, 0),
            padstack: Uuid::from_u128(20),
            layer: 1,
        },
    );

    Package {
        uuid: Uuid::from_u128(2),
        name: "SOIC-8".into(),
        package_family: Some("SOIC".into()),
        package_code: Some("SOIC-8".into()),
        mounting_type: Some("smd".into()),
        body_dimensions: Some(PackageBodyDimensions {
            x_nm: Some(4_900_000),
            y_nm: Some(3_900_000),
            z_nm: Some(1_750_000),
        }),
        terminals: HashMap::new(),
        pads,
        courtyard: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 500_000),
            Point::new(0, 500_000),
        ]),
        silkscreen: vec![],
        models_3d: vec![ModelRef {
            path: "models/soic8.step".into(),
            format: ModelFormat::Step,
            transform: Transform3D {
                translation_nm: Point3D {
                    x: 10,
                    y: 20,
                    z: 30,
                },
                rotation_tenths_deg: Euler3D {
                    roll_tenths_deg: 0,
                    pitch_tenths_deg: 0,
                    yaw_tenths_deg: 900,
                },
                scale: serde_json::Number::from_f64(1.0).unwrap(),
            },
            provenance: None,
        }],
        body_height_nm: Some(1_750_000),
        body_height_mounted_nm: Some(1_850_000),
        tags: HashSet::from(["soic".to_string()]),
    }
}

fn sample_footprint() -> Footprint {
    let pad_id = Uuid::from_u128(11);
    let mut pads = HashMap::new();
    pads.insert(
        pad_id,
        Pad {
            uuid: pad_id,
            name: "1".into(),
            position: Point::new(0, 0),
            padstack: Uuid::from_u128(20),
            layer: 1,
        },
    );

    Footprint {
        uuid: Uuid::from_u128(21),
        name: "SOIC-8-density-b".into(),
        package: Uuid::from_u128(2),
        pads,
        courtyard: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 500_000),
            Point::new(0, 500_000),
        ]),
        silkscreen: vec![],
        fab: vec![],
        assembly: vec![],
        mechanical: vec![],
        models_3d: vec![],
        standards_basis: Some("IPC-7351 density B".into()),
        ipc_basis: None,
        process_aperture_policy: Some("explicit".into()),
        tags: HashSet::from(["soic".to_string()]),
    }
}

fn sample_entity() -> Entity {
    let gate_id = Uuid::from_u128(3);
    let mut gates = HashMap::new();
    gates.insert(
        gate_id,
        Gate {
            uuid: gate_id,
            name: "A".into(),
            unit: Uuid::from_u128(1),
            symbol: Uuid::from_u128(4),
        },
    );

    Entity {
        uuid: Uuid::from_u128(5),
        name: "TL072".into(),
        prefix: "U".into(),
        manufacturer: "TI".into(),
        gates,
        tags: HashSet::from(["opamp".to_string()]),
    }
}

fn sample_part() -> Part {
    let mut pad_map = HashMap::new();
    pad_map.insert(
        Uuid::from_u128(10),
        PadMapEntry {
            gate: Uuid::from_u128(3),
            pin: Uuid::nil(),
        },
    );

    Part {
        uuid: Uuid::from_u128(6),
        entity: Uuid::from_u128(5),
        package: Uuid::from_u128(2),
        default_footprint: Some(Uuid::from_u128(21)),
        default_pin_pad_map: Some(Uuid::from_u128(22)),
        pad_map,
        mpn: "TL072CDR".into(),
        manufacturer: "TI".into(),
        manufacturer_jep106: Some(0x29),
        value: "TL072".into(),
        description: "Dual JFET op amp".into(),
        datasheet: "https://example.invalid/tl072.pdf".into(),
        parametric: HashMap::from([
            ("channels".to_string(), "2".to_string()),
            ("package".to_string(), "SOIC-8".to_string()),
        ]),
        orderable_mpns: vec!["TL072CDR".into()],
        packaging_options: Vec::new(),
        tags: HashSet::from(["opamp".to_string()]),
        lifecycle: Lifecycle::Active,
        base: None,
        behavioural_models: Vec::new(),
        thermal: None,
        supply_chain_offers: Some(vec![SupplyOffer {
            distributor: "Digi-Key".into(),
            price_breaks: vec![SupplyPriceBreak {
                qty: 1,
                price: serde_json::Number::from_f64(1.23).unwrap(),
                currency: "USD".into(),
            }],
            stock: Some(100),
            lead_time_weeks: None,
            link: "https://example.invalid/dk".into(),
        }]),
        last_supply_chain_check: Some("2026-06-22T12:00:00Z".into()),
    }
}

fn sample_pin_pad_map() -> PinPadMap {
    PinPadMap {
        uuid: Uuid::from_u128(22),
        part: Uuid::from_u128(6),
        footprint: Some(Uuid::from_u128(21)),
        mappings: HashMap::from([(
            Uuid::from_u128(11),
            PadMapEntry {
                gate: Uuid::from_u128(3),
                pin: Uuid::nil(),
            },
        )]),
        tags: HashSet::from(["soic".to_string()]),
    }
}

fn sample_pool() -> Pool {
    let mut pool = Pool::default();
    pool.units.insert(Uuid::from_u128(1), sample_unit());
    pool.symbols.insert(
        Uuid::from_u128(4),
        Symbol {
            uuid: Uuid::from_u128(4),
            name: "OpAmpA".into(),
            unit: Uuid::from_u128(1),
            fields: vec![LibrarySymbolField {
                key: "Value".into(),
                value: "TL072".into(),
                position: Some(Point::new(10, 20)),
                visible: true,
            }],
            default_refdes_prefix: Some("U".into()),
            style_profile_assertions: vec!["datum.symbol.style.default".into()],
            standards_basis: Some("Decision 008A".into()),
            check_state: Some(LibraryCheckState {
                status: LibraryCheckStatus::Passed,
                checked_at: Some("2026-06-22T12:00:00Z".into()),
                checked_by: Some("fixture".into()),
                notes: Vec::new(),
            }),
            provenance: Some(LibraryObjectProvenance {
                source: Some("fixture".into()),
                source_hash: Some("sha256:test".into()),
                reviewed_by: None,
                reviewed_at: None,
            }),
            drawings: Vec::new(),
            pin_anchors: vec![SymbolPinAnchor {
                pin: Uuid::nil(),
                position: Point::new(0, 0),
                style: SymbolPinStyle {
                    orientation: SymbolPinOrientation::Left,
                    length_nm: Some(2_540_000),
                    decoration: SymbolPinDecoration::Inverted,
                },
            }],
        },
    );
    pool.entities.insert(Uuid::from_u128(5), sample_entity());
    pool.padstacks.insert(
        Uuid::from_u128(20),
        Padstack {
            uuid: Uuid::from_u128(20),
            name: "round-0.5mm".into(),
            aperture: Some(PadstackAperture::Circle {
                diameter_nm: 500_000,
            }),
            drill_nm: Some(300_000),
            plated: Some(true),
            layer_span: PadstackLayerSpan::Through,
            mask_policy: PadstackMaskPolicy::Exposed,
            paste_policy: PadstackPastePolicy::None,
            annular_ring_nm: Some(100_000),
            thermal: Some(PadstackThermal {
                spoke_count: Some(4),
                spoke_width_nm: Some(200_000),
                gap_nm: Some(150_000),
            }),
            antipad: Some(PadstackAntipad {
                clearance_nm: Some(250_000),
                aperture: None,
            }),
        },
    );
    pool.packages.insert(Uuid::from_u128(2), sample_package());
    pool.footprints
        .insert(Uuid::from_u128(21), sample_footprint());
    pool.pin_pad_maps
        .insert(Uuid::from_u128(22), sample_pin_pad_map());
    pool.parts.insert(Uuid::from_u128(6), sample_part());
    pool
}

#[test]
fn unit_round_trip() {
    let unit = sample_unit();
    let json = serde_json::to_string(&unit).unwrap();
    let decoded: Unit = serde_json::from_str(&json).unwrap();
    assert_eq!(unit, decoded);
}

#[test]
fn library_pin_electrical_type_preserves_legacy_direction_json() {
    let pin = Pin {
        uuid: Uuid::from_u128(30),
        name: "OC".into(),
        direction: PinDirection::OpenCollector,
        swap_group: 0,
        alternates: Vec::new(),
    };

    let json = serde_json::to_value(&pin).unwrap();
    assert_eq!(json["direction"], "OpenCollector");

    let decoded: Pin = serde_json::from_value(json).unwrap();
    assert_eq!(decoded.direction, LibraryPinElectricalType::OpenCollector);
}

#[test]
fn legacy_symbol_json_defaults_production_schema_fields() {
    let symbol_id = Uuid::from_u128(31);
    let unit_id = Uuid::from_u128(32);
    let pin_id = Uuid::from_u128(33);
    let decoded: Symbol = serde_json::from_value(serde_json::json!({
        "uuid": symbol_id,
        "name": "LegacySymbol",
        "unit": unit_id,
        "pin_anchors": [{
            "pin": pin_id,
            "position": { "x": 100, "y": 200 }
        }]
    }))
    .expect("legacy symbol schema should remain valid");

    assert!(decoded.fields.is_empty());
    assert_eq!(decoded.default_refdes_prefix, None);
    assert!(decoded.style_profile_assertions.is_empty());
    assert_eq!(decoded.standards_basis, None);
    assert_eq!(decoded.check_state, None);
    assert_eq!(decoded.provenance, None);
    assert!(decoded.drawings.is_empty());
    assert_eq!(
        decoded.pin_anchors[0].style.orientation,
        SymbolPinOrientation::Right
    );
    assert_eq!(decoded.pin_anchors[0].style.length_nm, None);
    assert_eq!(
        decoded.pin_anchors[0].style.decoration,
        SymbolPinDecoration::None
    );
}

#[test]
fn symbol_pin_anchor_style_accepts_converged_schema_and_legacy_fields() {
    let symbol_id = Uuid::from_u128(35);
    let unit_id = Uuid::from_u128(36);
    let first_pin_id = Uuid::from_u128(37);
    let second_pin_id = Uuid::from_u128(38);
    let decoded: Symbol = serde_json::from_value(serde_json::json!({
        "uuid": symbol_id,
        "name": "StyledSymbol",
        "unit": unit_id,
        "pin_anchors": [
            {
                "pin": first_pin_id,
                "position": { "x": 100, "y": 200 },
                "style": {
                    "orientation": "Left",
                    "length_nm": 2540000,
                    "decoration": "inverted"
                }
            },
            {
                "pin": second_pin_id,
                "position": { "x": 300, "y": 400 },
                "orientation": "Up",
                "length_nm": 1270000,
                "decoration": "clock"
            }
        ]
    }))
    .expect("symbol pin anchor style should decode");

    assert_eq!(
        decoded.pin_anchors[0].style.orientation,
        SymbolPinOrientation::Left
    );
    assert_eq!(decoded.pin_anchors[0].style.length_nm, Some(2_540_000));
    assert_eq!(
        decoded.pin_anchors[0].style.decoration,
        SymbolPinDecoration::Inverted
    );
    assert_eq!(
        decoded.pin_anchors[1].style.orientation,
        SymbolPinOrientation::Up
    );
    assert_eq!(decoded.pin_anchors[1].style.length_nm, Some(1_270_000));
    assert_eq!(
        decoded.pin_anchors[1].style.decoration,
        SymbolPinDecoration::Clock
    );

    let encoded = serde_json::to_value(&decoded.pin_anchors[0]).expect("anchor should encode");
    assert_eq!(encoded["style"]["orientation"], "Left");
    assert_eq!(encoded["style"]["length_nm"], 2540000);
    assert_eq!(encoded["style"]["decoration"], "inverted");
    assert!(encoded.get("orientation").is_none());
}

#[test]
fn legacy_padstack_json_defaults_production_fields() {
    let padstack_id = Uuid::from_u128(34);
    let decoded: Padstack = serde_json::from_value(serde_json::json!({
        "uuid": padstack_id,
        "name": "legacy",
        "aperture": {
            "circle": {
                "diameter_nm": 500_000
            }
        },
        "drill_nm": 300_000
    }))
    .expect("legacy padstack schema should remain valid");

    assert_eq!(decoded.plated, None);
    assert_eq!(decoded.layer_span, PadstackLayerSpan::PadLayer);
    assert_eq!(decoded.mask_policy, PadstackMaskPolicy::Inherit);
    assert_eq!(decoded.paste_policy, PadstackPastePolicy::Inherit);
    assert_eq!(decoded.annular_ring_nm, None);
    assert_eq!(decoded.thermal, None);
    assert_eq!(decoded.antipad, None);
}

#[test]
fn legacy_model_attachment_json_defaults_review_fields() {
    let attachment_id = Uuid::from_u128(35);
    let model_id = Uuid::from_u128(36);
    let decoded: ModelAttachment = serde_json::from_value(serde_json::json!({
        "uuid": attachment_id,
        "model_uuid": model_id,
        "role": "Spice",
        "dialect": "Ngspice",
        "model_names": ["TL072"],
        "encrypted": false,
        "encryption_scheme": null,
        "provenance": {
            "source": "vendor",
            "vendor": "TI",
            "fetched_at": null,
            "sha256": "abc123"
        },
        "format_metadata": {
            "kind": "spice",
            "ngspice_validates": true
        }
    }))
    .expect("legacy model attachment schema should remain valid");

    assert_eq!(decoded.reviewed, None);
    assert!(decoded.notes.is_empty());
    assert_eq!(
        decoded
            .provenance
            .as_ref()
            .map(|provenance| &provenance.sha256),
        Some(&"abc123".to_string())
    );
}

#[test]
fn package_round_trip() {
    let package = sample_package();
    let json = serde_json::to_string(&package).unwrap();
    let decoded: Package = serde_json::from_str(&json).unwrap();
    assert_eq!(package, decoded);
}

#[test]
fn body_only_package_deserializes_without_land_pattern_fields() {
    let package_id = Uuid::from_u128(22);
    let terminal_id = Uuid::from_u128(23);
    let decoded: Package = serde_json::from_value(serde_json::json!({
        "uuid": package_id,
        "name": "SOIC-8 body",
        "package_family": "SOIC",
        "package_code": "SOIC-8",
        "mounting_type": "smd",
        "body_dimensions": {
            "x_nm": 4_900_000,
            "y_nm": 3_900_000,
            "z_nm": 1_750_000
        },
        "terminals": {
            terminal_id: {
                "uuid": terminal_id,
                "name": "1",
                "role": "lead"
            }
        }
    }))
    .expect("body-only package should be valid");

    assert_eq!(decoded.uuid, package_id);
    assert_eq!(decoded.package_family.as_deref(), Some("SOIC"));
    assert!(decoded.pads.is_empty());
    assert!(decoded.courtyard.vertices.is_empty());
    assert!(decoded.silkscreen.is_empty());
    assert!(decoded.terminals.contains_key(&terminal_id));
}

#[test]
fn footprint_round_trip() {
    let footprint = sample_footprint();
    let json = serde_json::to_string(&footprint).unwrap();
    let decoded: Footprint = serde_json::from_str(&json).unwrap();
    assert_eq!(footprint, decoded);
}

#[test]
fn pin_pad_map_round_trip() {
    let pin_pad_map = sample_pin_pad_map();
    let json = serde_json::to_string(&pin_pad_map).unwrap();
    let decoded: PinPadMap = serde_json::from_str(&json).unwrap();
    assert_eq!(pin_pad_map, decoded);
}

#[test]
fn part_round_trip() {
    let part = sample_part();
    let json = serde_json::to_string(&part).unwrap();
    let decoded: Part = serde_json::from_str(&json).unwrap();
    assert_eq!(part, decoded);
}

#[test]
fn pool_deterministic_serialization() {
    let pool = sample_pool();
    let a = crate::ir::serialization::to_json_deterministic(&pool).unwrap();
    let b = crate::ir::serialization::to_json_deterministic(&pool).unwrap();
    assert_eq!(a, b);
}

#[test]
fn pool_index_keyword_search() {
    let pool = sample_pool();
    let index = PoolIndex::open_in_memory().unwrap();
    index.rebuild_from_pool(&pool).unwrap();

    let results = index.search_keyword("opamp").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].manufacturer, "TI");
    assert_eq!(results[0].package, "SOIC-8");
}

#[test]
fn pool_index_parametric_search() {
    let pool = sample_pool();
    let index = PoolIndex::open_in_memory().unwrap();
    index.rebuild_from_pool(&pool).unwrap();

    let results = index.search_parametric("package", "SOIC-8").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].mpn, "TL072CDR");
}

#[test]
fn pool_index_keyword_search_returns_error_for_malformed_stored_uuid() {
    let index = PoolIndex::open_in_memory().unwrap();
    index
        .conn
        .execute(
            "INSERT INTO parts (uuid, mpn, manufacturer, value, description, package_uuid, package_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "not-a-uuid",
                "TL072CDR",
                "TI",
                "TL072",
                "Dual JFET op amp",
                Uuid::from_u128(2).to_string(),
                "SOIC-8"
            ],
        )
        .unwrap();

    let err = index
        .search_keyword("TL072")
        .expect_err("malformed stored UUID should return an error");
    assert!(matches!(
        err,
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, _)
    ));
}
