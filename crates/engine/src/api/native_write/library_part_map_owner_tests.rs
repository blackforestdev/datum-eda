//! Map ownership admission reuses canonical Part fixtures and reference readers.
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
    WriteProvenance::new("test", CommitSource::Manual, "Part Map ownership fixture")
}
struct Maps {
    f: Fixture,
    own: Uuid,
    foreign: Uuid,
    other_part: Uuid,
}
impl Maps {
    fn new() -> Self {
        let mut f = Fixture::new();
        let (own, foreign, other_part) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let mut other = f.raw();
        other["uuid"] = json!(other_part);
        for (kind, id, object) in [
            ("parts", other_part, other),
            (
                "pin_pad_maps",
                own,
                pin_pad_map_payload(own, f.part, Some(f.good), &[]),
            ),
            (
                "pin_pad_maps",
                foreign,
                pin_pad_map_payload(foreign, other_part, Some(f.good), &[]),
            ),
        ] {
            let p = library::build_pool_library_write(
                &f.model,
                provenance(),
                Some("pool"),
                vec![Spec::Create {
                    target: Target::new("pool", kind, id),
                    object,
                }],
            )
            .unwrap();
            commit_prepared(&mut f.model, &f.root, p).unwrap();
        }
        Self {
            f,
            own,
            foreign,
            other_part,
        }
    }
    fn check(&self, create: bool, value: &Value) -> Result<(), EngineError> {
        let path = format!("pool/parts/{}.json", value["uuid"].as_str().unwrap());
        if create {
            validate_part_creation_references(&self.f.model, &path, value)
        } else {
            validate_part_replacement_references(&self.f.model, &path, value)
        }
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
fn part_map_owner_refuses_foreign_create_set_and_retains_supported_defaults() {
    let m = Maps::new();
    let before = m.witness();
    for create in [false, true] {
        let mut value = m.f.raw();
        if create {
            value["uuid"] = json!(Uuid::new_v4());
        }
        value["default_pin_pad_map"] = json!(m.foreign);
        let error = m.check(create, &value).unwrap_err().to_string();
        assert!(
            error.contains(&format!(
                "belongs to part {}, not {}",
                m.other_part,
                value["uuid"].as_str().unwrap()
            )),
            "{error}"
        );
        value["default_pin_pad_map"] = Value::Null;
        m.check(create, &value).unwrap();
        value.as_object_mut().unwrap().remove("default_pin_pad_map");
        m.check(create, &value).unwrap();
        assert_eq!(m.witness(), before);
    }
    // A Map can be made for an existing Part and then selected by Set. No
    // circular matching-Map/new-Part Create sequence is invented here.
    let mut value = m.f.raw();
    value["default_pin_pad_map"] = json!(m.own);
    value["vendor"] = json!([3, 1, 3]);
    m.check(false, &value).unwrap();
    assert_eq!(m.witness(), before);
}
#[test]
fn part_map_owner_preserves_reference_and_footprint_error_precedence() {
    let mut m = Maps::new();
    let before = m.witness();
    for id in [Uuid::new_v4(), m.f.package] {
        let mut value = m.f.raw();
        value["default_pin_pad_map"] = json!(id);
        let error = m.check(false, &value).unwrap_err().to_string();
        assert!(
            error.contains(&format!(
                "missing pool pin_pad_map {id} for part {}",
                m.f.part
            )),
            "{error}"
        );
    }
    let mut value = m.f.raw();
    value["default_pin_pad_map"] = json!(m.foreign);
    value["default_footprint"] = json!(m.f.bad);
    let error = m.check(false, &value).unwrap_err().to_string();
    assert!(error.contains("belongs to package"), "{error}");
    value["default_footprint"] = json!(m.f.good);
    value["default_pin_pad_map"] = json!(m.own);
    m.f.model.objects.get_mut(&m.own).unwrap().source_shard_id = Uuid::new_v4();
    assert!(
        m.check(false, &value)
            .unwrap_err()
            .to_string()
            .contains("missing source shard")
    );
    assert_eq!(m.witness(), before);
}
#[test]
fn part_map_owner_reference_edit_stales_prepared_part_without_moving_its_revision() {
    let mut m = Maps::new();
    let mut value = m.f.raw();
    value["default_pin_pad_map"] = json!(m.own);
    m.check(false, &value).unwrap();
    let prepared = library::build_pool_library_write(
        &m.f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "parts", m.f.part),
            object: value,
        }],
    )
    .unwrap();
    let old_part_revision = m.f.model.objects[&m.f.part].object_revision;
    let mut map = library::pool_library_object_payload(
        &m.f.model,
        &format!("pool/pin_pad_maps/{}.json", m.own),
        m.own,
    )
    .unwrap();
    map["part"] = json!(m.other_part);
    let p = library::build_pool_library_write(
        &m.f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "pin_pad_maps", m.own),
            object: map,
        }],
    )
    .unwrap();
    // The raw canonical Manual API retains its existing policy; this admission
    // check belongs to the public CLI. The whole-model batch guard protects the
    // reference read even though the candidate Part itself has not changed.
    commit_prepared(&mut m.f.model, &m.f.root, p).unwrap();
    assert_eq!(
        m.f.model.objects[&m.f.part].object_revision,
        old_part_revision
    );
    let before = m.witness();
    assert!(commit_prepared(&mut m.f.model, &m.f.root, prepared).is_err());
    assert_eq!(m.witness(), before);
}
