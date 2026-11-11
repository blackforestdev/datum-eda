//! Public Part Set refuses correctly-owned Maps with incompatible physical links.
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
fn command(root: &Path, verb: &str, args: &[&str]) {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    execute(Cli::try_parse_from(argv).unwrap()).unwrap();
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, out)
            } else if !p.ends_with("writer.lock") {
                out.insert(
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
#[test]
fn public_part_map_physical_refuses_conflicts_with_all_persisted_bytes_exact() {
    let cleanup = Cleanup(Fixture::new("part-map-physical-refusal"));
    let f = &cleanup.0;
    let alternate = Uuid::new_v4();
    let package = Uuid::new_v4();
    command(
        &f.root,
        "create-pool-footprint",
        &[
            "--footprint",
            &alternate.to_string(),
            "--package",
            &f.package.to_string(),
            "--name",
            "Alternate",
        ],
    );
    command(
        &f.root,
        "create-pool-package",
        &["--package", &package.to_string(), "--name", "Other"],
    );
    let original = payload(&f.root, f.part);
    let mut footprint = original.clone();
    footprint["default_pin_pad_map"] = json!(f.map);
    footprint["default_footprint"] = json!(alternate);
    let mut foreign = original.clone();
    foreign["default_pin_pad_map"] = json!(f.map);
    foreign["package"] = json!(package);
    for (value, diagnostic) in [
        (footprint, "targets footprint"),
        (foreign, "belongs to package"),
    ] {
        std::fs::write(
            f.root.join("part-input.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let before = snapshot(&f.root);
        let error = write(&f.root, "set-pool-library-object", f.part, &value).unwrap_err();
        assert!(format!("{error:#}").contains(diagnostic), "{error:#}");
        assert_eq!(snapshot(&f.root), before);
    }
}
#[test]
fn public_part_map_physical_matching_optional_defaults_history_and_cold() {
    let cleanup = Cleanup(Fixture::new("part-map-physical-history"));
    let f = &cleanup.0;
    let original = payload(&f.root, f.part);
    let map_path = f.root.join(format!("pool/pin_pad_maps/{}.json", f.map));
    let map = std::fs::read(&map_path).unwrap();
    let mut value = original.clone();
    value["default_pin_pad_map"] = json!(f.map);
    value["default_footprint"] = json!(f.footprint);
    value["vendor"] = json!({"literal":[3,1,3]});
    write(&f.root, "set-pool-library-object", f.part, &value).unwrap();
    assert_eq!(payload(&f.root, f.part), value);
    command(&f.root, "undo", &[]);
    assert_eq!(payload(&f.root, f.part), original);
    command(&f.root, "redo", &[]);
    assert_eq!(payload(&f.root, f.part), value);
    for null in [false, true] {
        value.as_object_mut().unwrap().remove("default_footprint");
        if null {
            value["default_footprint"] = Value::Null;
        }
        write(&f.root, "set-pool-library-object", f.part, &value).unwrap();
        assert_eq!(payload(&f.root, f.part), value);
    }
    assert_eq!(std::fs::read(map_path).unwrap(), map);
    let cold = eda_engine::substrate::ProjectResolver::new(&f.root)
        .resolve()
        .unwrap();
    assert!(cold.objects.contains_key(&f.part));
    assert_eq!(payload(&f.root, f.part), value);
    // Matching Map creation needs an existing Part. Supported new-Part Create
    // remains the null/omitted Map case, without inventing circular creation.
    for null in [false, true] {
        let mut new = value.clone();
        let id = Uuid::new_v4();
        new["uuid"] = json!(id);
        new.as_object_mut().unwrap().remove("default_pin_pad_map");
        if null {
            new["default_pin_pad_map"] = Value::Null;
        }
        write(&f.root, "create-pool-library-object", id, &new).unwrap();
        assert_eq!(payload(&f.root, id), new);
    }
}
