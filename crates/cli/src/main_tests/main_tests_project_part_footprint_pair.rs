//! Normal public Part Create/Set preserve project bytes on incompatible defaults.
use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn command(root: &Path, verb: &str, args: &[&str]) -> Result<String> {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    execute(Cli::try_parse_from(argv).unwrap())
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, result);
            } else if !p.ends_with("writer.lock") {
                result.insert(
                    p.strip_prefix(root).unwrap().into(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
// Existing canonical source-ownership refusals retain failed staging payloads.
// The new pair-refusal tests above use the full snapshot without this exclusion.
fn canonical_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = snapshot(root);
    files.retain(|path, _| !path.starts_with(".datum/stage"));
    files
}
struct Fixture {
    root: PathBuf,
    part: Uuid,
    package: Uuid,
    other: Uuid,
    good: Uuid,
    bad: Uuid,
    entity: Uuid,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_file(self.root.with_extension("input.json"));
    }
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-cli-part-pair-{}", Uuid::new_v4()));
        command(
            &root,
            "new",
            &["--name", "Part pair", "--units-source", "factory"],
        )
        .unwrap();
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
        command(
            &root,
            "create-pool-unit",
            &["--unit", &unit.to_string(), "--name", "Logic"],
        )
        .unwrap();
        command(
            &root,
            "create-pool-symbol",
            &[
                "--symbol",
                &symbol.to_string(),
                "--unit",
                &unit.to_string(),
                "--name",
                "Logic symbol",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-entity",
            &[
                "--entity",
                &entity.to_string(),
                "--gate",
                &gate.to_string(),
                "--unit",
                &unit.to_string(),
                "--symbol",
                &symbol.to_string(),
                "--name",
                "Logic entity",
                "--prefix",
                "U",
            ],
        )
        .unwrap();
        for (id, name) in [(package, "Body"), (other, "Other")] {
            command(
                &root,
                "create-pool-package",
                &["--package", &id.to_string(), "--name", name],
            )
            .unwrap();
        }
        for (id, pkg) in [(good, package), (bad, other)] {
            command(
                &root,
                "create-pool-footprint",
                &[
                    "--footprint",
                    &id.to_string(),
                    "--package",
                    &pkg.to_string(),
                    "--name",
                    "Lands",
                ],
            )
            .unwrap();
        }
        command(
            &root,
            "create-pool-part",
            &[
                "--part",
                &part.to_string(),
                "--entity",
                &entity.to_string(),
                "--package",
                &package.to_string(),
                "--mpn",
                "PAIR",
            ],
        )
        .unwrap();
        Self {
            root,
            part,
            package,
            other,
            good,
            bad,
            entity,
        }
    }
    fn read(&self, id: Uuid) -> Value {
        serde_json::from_slice(
            &std::fs::read(self.root.join(format!("pool/parts/{id}.json"))).unwrap(),
        )
        .unwrap()
    }
    fn write(&self, verb: &str, id: Uuid, value: &Value) -> Result<String> {
        let input = self.root.with_extension("input.json");
        std::fs::write(&input, serde_json::to_vec(value).unwrap()).unwrap();
        command(
            &self.root,
            verb,
            &[
                "--kind",
                "parts",
                "--object",
                &id.to_string(),
                "--from-json",
                input.to_str().unwrap(),
            ],
        )
    }
}
#[test]
fn public_part_pair_create_set_refuse_foreign_default_without_source_or_history_changes() {
    let f = Fixture::new();
    let initial = snapshot(&f.root);
    let fresh = Uuid::new_v4();
    for (verb, id) in [
        ("create-pool-library-object", fresh),
        ("set-pool-library-object", f.part),
    ] {
        let mut value = f.read(f.part);
        value["uuid"] = json!(id);
        value["default_footprint"] = json!(f.bad);
        let error = f.write(verb, id, &value).unwrap_err();
        assert!(
            format!("{error:#}").contains(&format!(
                "belongs to package {}, not part package {}",
                f.other, f.package
            )),
            "{error:#}"
        );
        assert_eq!(
            snapshot(&f.root),
            initial,
            "refusal preserves every persisted file and journal/cursor"
        );
    }
}
#[test]
fn public_part_pair_compatible_create_set_clear_and_cold_history_preserve_raw_metadata() {
    let f = Fixture::new();
    let id = Uuid::new_v4();
    let mut value = f.read(f.part);
    value["uuid"] = json!(id);
    value["vendor"] = json!({"values":[3,1,3],"label":"unchanged"});
    value["default_footprint"] = json!(f.good);
    f.write("create-pool-library-object", id, &value).unwrap();
    assert_eq!(f.read(id), value);
    let first = value.clone();
    value["mpn"] = json!("EDITED");
    value["default_footprint"] = Value::Null;
    f.write("set-pool-library-object", id, &value).unwrap();
    assert_eq!(f.read(id), value);
    command(&f.root, "undo", &[]).unwrap();
    assert_eq!(f.read(id), first);
    command(&f.root, "redo", &[]).unwrap();
    assert_eq!(f.read(id), value);
    value.as_object_mut().unwrap().remove("default_footprint");
    f.write("set-pool-library-object", id, &value).unwrap();
    assert_eq!(f.read(id), value);
    let second = Uuid::new_v4();
    value["uuid"] = json!(second);
    f.write("create-pool-library-object", second, &value)
        .unwrap();
    assert_eq!(f.read(second), value);
    let cold = eda_engine::substrate::ProjectResolver::new(&f.root)
        .resolve()
        .unwrap();
    assert!(cold.objects.contains_key(&id) && cold.objects.contains_key(&second));
    let queried =
        super::main_tests_project_pool_library::query_pool_object_payload(&f.root, "parts", second);
    assert_eq!(queried, value);
}
#[test]
fn public_part_pair_keeps_missing_wrong_kind_and_ownership_errors_ahead_of_pair_check() {
    let f = Fixture::new();
    let before = canonical_snapshot(&f.root);
    for (verb, id) in [
        ("create-pool-library-object", Uuid::new_v4()),
        ("set-pool-library-object", f.part),
    ] {
        for reference in [Uuid::new_v4(), f.package, f.entity] {
            let mut value = f.read(f.part);
            value["uuid"] = json!(id);
            value["default_footprint"] = json!(reference);
            let error = f.write(verb, id, &value).unwrap_err();
            assert_eq!(
                format!("{error:#}"),
                format!("missing pool footprint {reference} for part {id}")
            );
            assert_eq!(canonical_snapshot(&f.root), before);
        }
        let mut value = f.read(f.part);
        value["uuid"] = json!(id);
        value["default_footprint"] = json!(f.bad);
        value["extension"] = json!({"uuid":f.package});
        let error = f.write(verb, id, &value).unwrap_err();
        let expected = if verb.starts_with("create") {
            "source UUID already exists"
        } else {
            "foreign source UUID collision"
        };
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert_eq!(canonical_snapshot(&f.root), before);
    }
}
