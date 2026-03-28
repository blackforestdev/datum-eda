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
    #[serde(default)]
    pub aperture: Option<PadstackAperture>,
    #[serde(default)]
    pub drill_nm: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PadstackAperture {
    Circle { diameter_nm: i64 },
    Rect { width_nm: i64, height_nm: i64 },
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
    pub package_uuid: Uuid,
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
                package_uuid TEXT NOT NULL,
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
                (uuid, mpn, manufacturer, value, description, package_uuid, package_name)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                part.uuid.to_string(),
                part.mpn,
                part.manufacturer,
                part.value,
                part.description,
                part.package.to_string(),
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
            SELECT DISTINCT p.uuid, p.mpn, p.manufacturer, p.value, p.package_uuid, p.package_name
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
                package_uuid: Uuid::parse_str(&row.get::<_, String>(4)?)
                    .expect("stored package UUID must parse"),
                package: row.get(5)?,
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
            SELECT p.uuid, p.mpn, p.manufacturer, p.value, p.package_uuid, p.package_name
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
                package_uuid: Uuid::parse_str(&row.get::<_, String>(4)?)
                    .expect("stored package UUID must parse"),
                package: row.get(5)?,
            })
        })?;

        rows.collect()
    }
}

#[cfg(test)]
#[path = "tests/mod_tests_pool.rs"]
mod tests;
