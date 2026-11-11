//! Public manual admission preserves canonical source/error/history boundaries.
use super::*;
use crate::api::native_write::{
    WriteProvenance, commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
};
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Part pair fixture")
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if !path.ends_with("writer.lock") {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}
// Existing canonical source-ownership refusals retain failed staging payloads.
// The new pair-refusal tests above use the full snapshot without this exclusion.
fn canonical_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = snapshot(root);
    files.retain(|path, _| !path.starts_with(".datum/stage"));
    files
}
pub(super) struct Fixture {
    pub(super) root: PathBuf,
    pub(super) model: DesignModel,
    pub(super) part: Uuid,
    pub(super) package: Uuid,
    pub(super) other: Uuid,
    pub(super) good: Uuid,
    pub(super) bad: Uuid,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    pub(super) fn new() -> Self {
        let root = crate::api::native_write::test_support::temp_project_root("part-pair");
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Part pair".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let (unit, symbol, entity, gate, package, other, good, bad, part) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let mut fixture = Self {
            root,
            model,
            part,
            package,
            other,
            good,
            bad,
        };
        let objects = [
            ("units", unit, library::pool_unit_payload(unit, "Logic", "")),
            (
                "symbols",
                symbol,
                library::pool_symbol_payload(symbol, unit, "Logic"),
            ),
            (
                "entities",
                entity,
                library::pool_entity_payload(entity, gate, unit, symbol, "Logic", "U", "", "A"),
            ),
            (
                "packages",
                package,
                library::pool_package_payload(package, "Body", json!({})),
            ),
            (
                "packages",
                other,
                library::pool_package_payload(other, "Other", json!({})),
            ),
            (
                "footprints",
                good,
                crate::api::native_write::library_footprint::pool_footprint_payload(
                    good, package, "Matching",
                ),
            ),
            (
                "footprints",
                bad,
                crate::api::native_write::library_footprint::pool_footprint_payload(
                    bad, other, "Foreign",
                ),
            ),
            (
                "parts",
                part,
                library::pool_part_payload(
                    part,
                    entity,
                    package,
                    "PAIR",
                    "Maker",
                    "Value",
                    "Description",
                    "",
                    "Unknown",
                ),
            ),
        ];
        for (kind, id, object) in objects {
            let prepared = library::build_pool_library_write(
                &fixture.model,
                provenance(),
                Some("pool"),
                vec![Spec::Create {
                    target: Target::new("pool", kind, id),
                    object,
                }],
            )
            .unwrap();
            commit_prepared(&mut fixture.model, &fixture.root, prepared).unwrap();
        }
        fixture
    }
    pub(super) fn raw(&self) -> Value {
        library::pool_library_object_payload(
            &self.model,
            &format!("pool/parts/{}.json", self.part),
            self.part,
        )
        .unwrap()
    }
    fn check(&self, create: bool, object: &Value) -> Result<(), EngineError> {
        let id = object["uuid"].as_str().unwrap();
        let path = format!("pool/parts/{id}.json");
        if create {
            validate_part_creation_references(&self.model, &path, object)
        } else {
            validate_part_replacement_references(&self.model, &path, object)
        }
    }
}
#[test]
fn part_pair_create_set_refuse_foreign_defaults_and_keep_exact_sources() {
    let f = Fixture::new();
    let before = snapshot(&f.root);
    for create in [false, true] {
        let mut object = f.raw();
        if create {
            object["uuid"] = json!(Uuid::new_v4());
        }
        object["default_footprint"] = json!(f.bad);
        let error = f.check(create, &object).unwrap_err().to_string();
        assert!(
            error.contains(&format!(
                "belongs to package {}, not part package {}",
                f.other, f.package
            )),
            "{error}"
        );
        assert_eq!(snapshot(&f.root), before);
        object["default_footprint"] = json!(f.good);
        object["vendor"] = json!({"literal":[3,1,3]});
        f.check(create, &object).unwrap();
        object["default_footprint"] = Value::Null;
        f.check(create, &object).unwrap();
        object.as_object_mut().unwrap().remove("default_footprint");
        f.check(create, &object).unwrap();
        assert_eq!(snapshot(&f.root), before);
    }
}
#[test]
fn part_pair_keeps_reference_error_order_and_refuses_unavailable_source() {
    let mut f = Fixture::new();
    let before = snapshot(&f.root);
    for id in [Uuid::new_v4(), f.package] {
        let mut object = f.raw();
        object["default_footprint"] = json!(id);
        let error = f.check(false, &object).unwrap_err().to_string();
        assert!(
            error.contains(&format!("missing pool footprint {id} for part {}", f.part)),
            "{error}"
        );
    }
    let mut object = f.raw();
    object["default_footprint"] = json!(f.good);
    f.model.objects.get_mut(&f.good).unwrap().source_shard_id = Uuid::new_v4();
    assert!(
        f.check(false, &object)
            .unwrap_err()
            .to_string()
            .contains("missing source shard")
    );
    assert_eq!(snapshot(&f.root), before);
}
#[test]
fn part_pair_retains_source_blocker_and_shape_precedence() {
    let mut f = Fixture::new();
    let before = canonical_snapshot(&f.root);
    let mut object = f.raw();
    object["default_footprint"] = json!(f.bad);
    object["vendor"] = json!({"uuid":f.package});
    // Same-owner edits remain supported; foreign UUID ownership remains the
    // canonical writer's error rather than being masked by the new pair check.
    f.check(false, &object).unwrap();
    let p = library::build_pool_library_write(
        &f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "parts", f.part),
            object,
        }],
    )
    .unwrap();
    let error = commit_prepared(&mut f.model, &f.root, p)
        .unwrap_err()
        .to_string();
    assert!(error.contains("foreign source UUID collision"), "{error}");
    assert_eq!(canonical_snapshot(&f.root), before);
    let mut object = f.raw();
    object["schema_version"] = json!(2);
    object["default_footprint"] = json!(f.bad);
    f.check(false, &object).unwrap();
    object["schema_version"] = json!(1);
    object["lifecycle"] = json!("InvalidLifecycle");
    f.check(false, &object).unwrap();
    assert_eq!(canonical_snapshot(&f.root), before);
}
#[test]
fn part_pair_does_not_extend_raw_canonical_writes_and_stale_commit_stays_atomic() {
    let mut f = Fixture::new();
    let mut object = f.raw();
    object["default_footprint"] = json!(f.good);
    f.check(false, &object).unwrap();
    let prepared = library::build_pool_library_write(
        &f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "parts", f.part),
            object: object.clone(),
        }],
    )
    .unwrap();
    // Raw canonical Manual writes retain their contract; this new admission
    // rule belongs only to the already validated public CLI commands.
    object["default_footprint"] = json!(f.bad);
    let raw = library::build_pool_library_write(
        &f.model,
        provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "parts", f.part),
            object,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, raw).unwrap();
    let before = snapshot(&f.root);
    assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
    assert_eq!(snapshot(&f.root), before);
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&cold, &format!("pool/parts/{}.json", f.part), f.part)
            .unwrap()["default_footprint"],
        json!(f.bad)
    );
}
