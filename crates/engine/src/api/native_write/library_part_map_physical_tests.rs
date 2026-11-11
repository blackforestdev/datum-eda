//! Map physical links use existing canonical fixtures; public CLI proof uses real pads.
use super::tests::Fixture;
use super::*;
use crate::api::native_write::{
    WriteProvenance, commit_prepared,
    library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    library_pin_pad_map::pin_pad_map_payload,
};
use crate::substrate::CommitSource;
use serde_json::{Value, json};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Part Map physical fixture")
}
struct Maps {
    f: Fixture,
    map: Uuid,
}
impl Maps {
    fn new() -> Self {
        let f = Fixture::new();
        let mut m = Self {
            f,
            map: Uuid::new_v4(),
        };
        let object = pin_pad_map_payload(m.map, m.f.part, Some(m.f.good), &[]);
        m.commit("pin_pad_maps", m.map, object, true);
        m
    }
    fn commit(&mut self, kind: &str, id: Uuid, object: Value, create: bool) {
        let target = Target::new("pool", kind, id);
        let op = if create {
            Spec::Create { target, object }
        } else {
            Spec::Set { target, object }
        };
        let p =
            library::build_pool_library_write(&self.f.model, provenance(), Some("pool"), vec![op])
                .unwrap();
        commit_prepared(&mut self.f.model, &self.f.root, p).unwrap();
    }
    fn value(&self) -> Value {
        let mut value = self.f.raw();
        value["default_pin_pad_map"] = json!(self.map);
        value
    }
    fn check(&self, value: &Value) -> Result<(), EngineError> {
        validate_part_replacement_references(
            &self.f.model,
            &format!("pool/parts/{}.json", self.f.part),
            value,
        )
    }
    fn map_payload(&self) -> Value {
        library::pool_library_object_payload(
            &self.f.model,
            &format!("pool/pin_pad_maps/{}.json", self.map),
            self.map,
        )
        .unwrap()
    }
    fn witness(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        (
            std::fs::read(self.f.root.join(format!("pool/parts/{}.json", self.f.part))).unwrap(),
            std::fs::read(self.f.root.join(".datum/journal/transactions.jsonl")).unwrap(),
            std::fs::read(self.f.root.join(".datum/journal/cursor.json")).unwrap(),
        )
    }
}
#[test]
fn part_map_physical_refuses_conflicts_and_keeps_optional_links() {
    let mut m = Maps::new();
    let alternate = Uuid::new_v4();
    m.commit(
        "footprints",
        alternate,
        crate::api::native_write::library_footprint::pool_footprint_payload(
            alternate,
            m.f.package,
            "Alternate",
        ),
        true,
    );
    let before = m.witness();
    let mut value = m.value();
    value["default_footprint"] = json!(alternate);
    let error = m.check(&value).unwrap_err().to_string();
    assert!(
        error.contains(&format!(
            "targets footprint {}, not default_footprint {alternate}",
            m.f.good
        )),
        "{error}"
    );
    value.as_object_mut().unwrap().remove("default_footprint");
    value["package"] = json!(m.f.other);
    let error = m.check(&value).unwrap_err().to_string();
    assert!(
        error.contains(&format!(
            "belongs to package {}, not part package {}",
            m.f.package, m.f.other
        )),
        "{error}"
    );
    assert_eq!(m.witness(), before);
    value["package"] = json!(m.f.package);
    value["vendor"] = json!({"literal":[3,1,3]});
    m.check(&value).unwrap();
    value["default_footprint"] = Value::Null;
    m.check(&value).unwrap();
    value["default_footprint"] = json!(m.f.good);
    m.check(&value).unwrap();
    // Optional Map footprint stays optional even with an explicit Part default.
    for null in [false, true] {
        let mut map = m.map_payload();
        map.as_object_mut().unwrap().remove("footprint");
        if null {
            map["footprint"] = Value::Null;
        }
        m.commit("pin_pad_maps", m.map, map, false);
        m.check(&value).unwrap();
    }
}
#[test]
fn part_map_physical_retains_owner_reference_and_source_error_precedence() {
    let mut m = Maps::new();
    let mut value = m.value();
    value["package"] = json!(m.f.other);
    value["default_footprint"] = json!(m.f.good);
    assert!(
        m.check(&value)
            .unwrap_err()
            .to_string()
            .contains("belongs to package")
    );
    value.as_object_mut().unwrap().remove("default_footprint");
    let candidate = Uuid::new_v4();
    value["uuid"] = json!(candidate);
    let error = validate_part_creation_references(
        &m.f.model,
        &format!("pool/parts/{candidate}.json"),
        &value,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("belongs to part"), "{error}");
    value["uuid"] = json!(m.f.part);
    for id in [Uuid::new_v4(), m.f.package] {
        let mut map = m.map_payload();
        map["footprint"] = json!(id);
        m.commit("pin_pad_maps", m.map, map, false);
        let before = m.witness();
        let error = m.check(&value).unwrap_err().to_string();
        assert!(
            error.contains(&format!("missing pool footprint {id}")),
            "{error}"
        );
        assert_eq!(m.witness(), before);
    }
    let mut map = m.map_payload();
    map["footprint"] = json!(m.f.good);
    m.commit("pin_pad_maps", m.map, map, false);
    m.f.model
        .objects
        .get_mut(&m.f.good)
        .unwrap()
        .source_shard_id = Uuid::new_v4();
    assert!(
        m.check(&value)
            .unwrap_err()
            .to_string()
            .contains("missing source shard")
    );
}
#[test]
fn part_map_physical_reference_change_stales_prepared_part() {
    let mut m = Maps::new();
    let value = m.value();
    m.check(&value).unwrap();
    let p = library::build_pool_library_write(
        &m.f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "parts", m.f.part),
            object: value,
        }],
    )
    .unwrap();
    let revision = m.f.model.objects[&m.f.part].object_revision;
    let mut map = m.map_payload();
    map["footprint"] = json!(m.f.bad);
    // Existing raw canonical writes retain their policy. Whole-model revision
    // protects a prepared Part batch when only its referenced Map changes.
    m.commit("pin_pad_maps", m.map, map, false);
    assert_eq!(m.f.model.objects[&m.f.part].object_revision, revision);
    let before = m.witness();
    assert!(commit_prepared(&mut m.f.model, &m.f.root, p).is_err());
    assert_eq!(m.witness(), before);
}
