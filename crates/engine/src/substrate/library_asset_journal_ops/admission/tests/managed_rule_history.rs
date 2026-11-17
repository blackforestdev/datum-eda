//! Generated CAD is regression data; real-project GUI proof remains separate.
use super::*;
use serde_json::json;

fn retained_design(f: &Fixture) -> BTreeMap<PathBuf, Vec<u8>> {
    files(&f.root)
        .into_iter()
        .filter(|(path, _)| {
            [
                "pool",
                "library",
                "board",
                "schematic",
                ".datum/component_instances",
                ".datum/electrical_identities",
            ]
            .iter()
            .any(|prefix| path.starts_with(prefix))
        })
        .collect()
}

fn cold_with_same_binding(f: &mut Fixture, retained: &BTreeMap<PathBuf, Vec<u8>>) {
    let cold = ProjectResolver::new(&f.root)
        .resolve()
        .expect("ordinary managed-project cold open after a rule edit");
    assert_eq!(cold.model_revision, f.model.model_revision);
    assert_eq!(cold.objects, f.model.objects);
    assert_eq!(cold.component_instances, f.model.component_instances);
    assert_eq!(cold.electrical_identities, f.model.electrical_identities);
    assert_eq!(
        cold.journal_cursor.applied_transaction_count,
        cold.journal.len()
    );
    f.model = cold;
    assert_eq!(&retained_design(f), retained);
}

fn rules(f: &Fixture) -> Value {
    f.model
        .materialized_source_shard_value(SourceShardKind::RulesRoot)
        .unwrap()["rules"]
        .clone()
}

#[test]
fn normal_explicit_rule_keeps_managed_project_cold_and_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::normal_explicit_rule_keeps_managed_project_cold_and_history"
    );
    if isolated(NAME) {
        return;
    }
    let mut f = Fixture::new(true);
    f.baseline();
    let retained = retained_design(&f);
    let before = rules(&f);
    let id = Uuid::new_v4();
    write(
        &f.root,
        &mut f.model,
        "datum.project.create_rule",
        json!({"rule": {
            "uuid": id,
            "name": "Explicit clearance",
            "scope": "All",
            "priority": 1,
            "enabled": true,
            "rule_type": "clearance_copper",
            "params": {"min_nm": 200000}
        }}),
    );
    let created = rules(&f);
    assert_eq!(
        created.as_array().unwrap().len(),
        before.as_array().unwrap().len() + 1
    );
    assert!(
        created
            .as_array()
            .unwrap()
            .iter()
            .any(|rule| rule["uuid"] == json!(id))
    );
    cold_with_same_binding(&mut f, &retained);
    assert_eq!(rules(&f), created);
    f.model
        .commit_journal_undo(&f.root, public_provenance().into())
        .unwrap();
    cold_with_same_binding(&mut f, &retained);
    assert_eq!(rules(&f), before);
    f.model
        .commit_journal_redo(&f.root, public_provenance().into())
        .unwrap();
    cold_with_same_binding(&mut f, &retained);
    assert_eq!(rules(&f), created);
    std::fs::remove_dir_all(&f.root).unwrap();
}
