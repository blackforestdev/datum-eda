use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::{Arc, LayerId, Point, Polygon};

/// Pool-domain types.
///
/// This is the M0 foundation layer only: canonical pool objects, simple pool
/// container types, and serialization tests. Indexing, search, and import
/// population are added in later M0 slices.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub uuid: Uuid,
    pub name: String,
    pub direction: PinDirection,
    pub swap_group: u32,
    pub alternates: Vec<AlternateName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinDirection {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    TriState,
    NoConnect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternateName {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub uuid: Uuid,
    pub name: String,
    pub manufacturer: String,
    pub pins: HashMap<Uuid, Pin>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub uuid: Uuid,
    pub name: String,
    pub unit: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gate {
    pub uuid: Uuid,
    pub name: String,
    pub unit: Uuid,
    pub symbol: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub uuid: Uuid,
    pub name: String,
    pub prefix: String,
    pub manufacturer: String,
    pub gates: HashMap<Uuid, Gate>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padstack {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pad {
    pub uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub padstack: Uuid,
    pub layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub models_3d: Vec<ModelRef>,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PadMapEntry {
    pub gate: Uuid,
    pub pin: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub uuid: Uuid,
    pub entity: Uuid,
    pub package: Uuid,
    pub pad_map: HashMap<Uuid, PadMapEntry>,
    pub mpn: String,
    pub manufacturer: String,
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub parametric: HashMap<String, String>,
    pub orderable_mpns: Vec<String>,
    pub tags: HashSet<String>,
    pub lifecycle: Lifecycle,
    pub base: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lifecycle {
    Active,
    Nrnd,
    Eol,
    Obsolete,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub path: String,
    pub transform: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Primitive {
    Line {
        from: Point,
        to: Point,
        width: i64,
    },
    Rect {
        min: Point,
        max: Point,
        width: i64,
    },
    Circle {
        center: Point,
        radius: i64,
        width: i64,
    },
    Polygon {
        polygon: Polygon,
        width: i64,
    },
    Arc {
        arc: Arc,
        width: i64,
    },
    Text {
        text: String,
        position: Point,
        rotation: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Pool {
    pub units: HashMap<Uuid, Unit>,
    pub symbols: HashMap<Uuid, Symbol>,
    pub entities: HashMap<Uuid, Entity>,
    pub padstacks: HashMap<Uuid, Padstack>,
    pub packages: HashMap<Uuid, Package>,
    pub parts: HashMap<Uuid, Part>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartSummary {
    pub uuid: Uuid,
    pub mpn: String,
    pub manufacturer: String,
    pub value: String,
    pub package: String,
}

pub struct PoolIndex {
    conn: Connection,
}

impl PoolIndex {
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS parts (
                uuid TEXT PRIMARY KEY,
                mpn TEXT NOT NULL,
                manufacturer TEXT NOT NULL,
                value TEXT NOT NULL,
                description TEXT NOT NULL,
                package_name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS part_tags (
                part_uuid TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (part_uuid, tag)
            );

            CREATE TABLE IF NOT EXISTS part_parametric (
                part_uuid TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (part_uuid, key)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn insert_part(&self, part: &Part, package_name: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO parts
                (uuid, mpn, manufacturer, value, description, package_name)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                part.uuid.to_string(),
                part.mpn,
                part.manufacturer,
                part.value,
                part.description,
                package_name
            ],
        )?;

        self.conn.execute(
            "DELETE FROM part_tags WHERE part_uuid = ?1",
            params![part.uuid.to_string()],
        )?;
        for tag in &part.tags {
            self.conn.execute(
                "INSERT INTO part_tags (part_uuid, tag) VALUES (?1, ?2)",
                params![part.uuid.to_string(), tag],
            )?;
        }

        self.conn.execute(
            "DELETE FROM part_parametric WHERE part_uuid = ?1",
            params![part.uuid.to_string()],
        )?;
        for (key, value) in &part.parametric {
            self.conn.execute(
                "INSERT INTO part_parametric (part_uuid, key, value) VALUES (?1, ?2, ?3)",
                params![part.uuid.to_string(), key, value],
            )?;
        }

        Ok(())
    }

    pub fn rebuild_from_pool(&self, pool: &Pool) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM part_tags", [])?;
        self.conn.execute("DELETE FROM part_parametric", [])?;
        self.conn.execute("DELETE FROM parts", [])?;

        for part in pool.parts.values() {
            let package_name = pool
                .packages
                .get(&part.package)
                .map(|p| p.name.as_str())
                .unwrap_or("");
            self.insert_part(part, package_name)?;
        }

        Ok(())
    }

    pub fn search_keyword(&self, query: &str) -> Result<Vec<PartSummary>, rusqlite::Error> {
        let pattern = format!("%{}%", query.to_lowercase());
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT p.uuid, p.mpn, p.manufacturer, p.value, p.package_name
            FROM parts p
            LEFT JOIN part_tags t ON t.part_uuid = p.uuid
            WHERE lower(p.mpn) LIKE ?1
               OR lower(p.manufacturer) LIKE ?1
               OR lower(p.value) LIKE ?1
               OR lower(p.description) LIKE ?1
               OR lower(p.package_name) LIKE ?1
               OR lower(t.tag) LIKE ?1
            ORDER BY p.manufacturer, p.mpn
            "#,
        )?;

        let rows = stmt.query_map(params![pattern], |row| {
            Ok(PartSummary {
                uuid: Uuid::parse_str(&row.get::<_, String>(0)?).expect("stored UUID must parse"),
                mpn: row.get(1)?,
                manufacturer: row.get(2)?,
                value: row.get(3)?,
                package: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    pub fn search_parametric(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<PartSummary>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT p.uuid, p.mpn, p.manufacturer, p.value, p.package_name
            FROM parts p
            JOIN part_parametric pp ON pp.part_uuid = p.uuid
            WHERE pp.key = ?1 AND pp.value = ?2
            ORDER BY p.manufacturer, p.mpn
            "#,
        )?;

        let rows = stmt.query_map(params![key, value], |row| {
            Ok(PartSummary {
                uuid: Uuid::parse_str(&row.get::<_, String>(0)?).expect("stored UUID must parse"),
                mpn: row.get(1)?,
                manufacturer: row.get(2)?,
                value: row.get(3)?,
                package: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
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
}
