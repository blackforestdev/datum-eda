//! Normal Part commands enforce Map ownership using the existing real pin/pad fixture.
use super::main_tests_project_pool_part_reference_validation::{Fixture, payload, write};
use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
struct Cleanup(Fixture);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0.root);
    }
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, result);
            } else if !p.ends_with("writer.lock") {
                result.insert(
                    p.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
fn history(root: &Path, verb: &str) {
    execute(
        Cli::try_parse_from([
            "datum-eda",
            "--format",
            "json",
            "project",
            verb,
            root.to_str().unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
}
#[test]
fn public_part_map_owner_refuses_foreign_create_set_with_every_persisted_file_exact() {
    let cleanup = Cleanup(Fixture::new("part-map-owner-refusal"));
    let f = &cleanup.0;
    let original = payload(&f.root, f.part);
    let other = Uuid::new_v4();
    let mut second = original.clone();
    second["uuid"] = json!(other);
    write(&f.root, "create-pool-library-object", other, &second).unwrap();
    for (verb, id) in [
        ("create-pool-library-object", Uuid::new_v4()),
        ("set-pool-library-object", other),
    ] {
        let mut value = original.clone();
        value["uuid"] = json!(id);
        value["default_footprint"] = json!(f.footprint);
        value["default_pin_pad_map"] = json!(f.map);
        // The shared normal-entry helper owns this unregistered input file.
        // Prime it before the snapshot so its identical transport write does
        // not obscure equality of every persisted Project file on refusal.
        std::fs::write(
            f.root.join("part-input.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let before = snapshot(&f.root);
        let error = write(&f.root, verb, id, &value).unwrap_err();
        assert!(
            format!("{error:#}").contains(&format!("belongs to part {}, not {id}", f.part)),
            "{error:#}"
        );
        assert_eq!(snapshot(&f.root), before);
    }
}
#[test]
fn public_part_map_owner_matching_set_optional_create_and_history_preserve_metadata() {
    let cleanup = Cleanup(Fixture::new("part-map-owner-history"));
    let f = &cleanup.0;
    let original = payload(&f.root, f.part);
    let map_bytes =
        std::fs::read(f.root.join(format!("pool/pin_pad_maps/{}.json", f.map))).unwrap();
    let mut changed = original.clone();
    changed["default_pin_pad_map"] = json!(f.map);
    changed["vendor"] = json!({"literal":[3,1,3]});
    write(&f.root, "set-pool-library-object", f.part, &changed).unwrap();
    assert_eq!(payload(&f.root, f.part), changed);
    history(&f.root, "undo");
    assert_eq!(payload(&f.root, f.part), original);
    history(&f.root, "redo");
    assert_eq!(payload(&f.root, f.part), changed);
    changed["default_pin_pad_map"] = Value::Null;
    write(&f.root, "set-pool-library-object", f.part, &changed).unwrap();
    assert_eq!(payload(&f.root, f.part), changed);
    changed
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    write(&f.root, "set-pool-library-object", f.part, &changed).unwrap();
    assert_eq!(payload(&f.root, f.part), changed);
    // A matching Map requires its Part to exist first. New Part Create is
    // proved only for the supported omitted and explicit-null defaults.
    let mut created = Vec::new();
    for null in [false, true] {
        let id = Uuid::new_v4();
        let mut value = changed.clone();
        value["uuid"] = json!(id);
        if null {
            value["default_pin_pad_map"] = Value::Null;
        }
        write(&f.root, "create-pool-library-object", id, &value).unwrap();
        assert_eq!(payload(&f.root, id), value);
        created.push(id);
    }
    assert_eq!(
        std::fs::read(f.root.join(format!("pool/pin_pad_maps/{}.json", f.map))).unwrap(),
        map_bytes
    );
    let cold = eda_engine::substrate::ProjectResolver::new(&f.root)
        .resolve()
        .unwrap();
    assert!(created.iter().all(|id| cold.objects.contains_key(id)));
    assert_eq!(payload(&f.root, f.part), changed);
}
