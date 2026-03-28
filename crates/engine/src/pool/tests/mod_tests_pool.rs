use super::*;
use crate::ir::geometry::Point;

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
        pads,
        courtyard: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 500_000),
            Point::new(0, 500_000),
        ]),
        silkscreen: vec![],
        models_3d: vec![],
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
        pad_map,
        mpn: "TL072CDR".into(),
        manufacturer: "TI".into(),
        value: "TL072".into(),
        description: "Dual JFET op amp".into(),
        datasheet: "https://example.invalid/tl072.pdf".into(),
        parametric: HashMap::from([
            ("channels".to_string(), "2".to_string()),
            ("package".to_string(), "SOIC-8".to_string()),
        ]),
        orderable_mpns: vec!["TL072CDR".into()],
        tags: HashSet::from(["opamp".to_string()]),
        lifecycle: Lifecycle::Active,
        base: None,
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
        },
    );
    pool.entities.insert(Uuid::from_u128(5), sample_entity());
    pool.padstacks.insert(
        Uuid::from_u128(20),
        Padstack {
            uuid: Uuid::from_u128(20),
            name: "round-0.5mm".into(),
            aperture: Some(PadstackAperture::Circle { diameter_nm: 500_000 }),
            drill_nm: Some(300_000),
        },
    );
    pool.packages.insert(Uuid::from_u128(2), sample_package());
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
fn package_round_trip() {
    let package = sample_package();
    let json = serde_json::to_string(&package).unwrap();
    let decoded: Package = serde_json::from_str(&json).unwrap();
    assert_eq!(package, decoded);
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
