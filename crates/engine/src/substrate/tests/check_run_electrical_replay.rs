//! ERC persistence must retain its authored input after native topology history.
use super::*;
use crate::api::native_write::{
    WriteProvenance,
    artifacts::build_set_check_run,
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_connectivity::{build_create_schematic_wire, build_delete_schematic_wire},
};
use crate::{ir::geometry::Point, schematic::SchematicWire};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Test,
        "ERC electrical replay regression",
    )
}
fn run(model: &DesignModel) -> CheckRun {
    CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id: Uuid::new_v4(),
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        profile_id: "erc".into(),
        status: "ok".into(),
        summary: serde_json::json!({"status":"ok"}),
        finding_count: 0,
        findings: vec![],
        proposal_refs: vec![],
        proposal_links: vec![],
        profile_basis: Default::default(),
        coverage: vec![],
        raw_report: serde_json::json!({"domain":"erc"}),
    }
}
fn assert_electrical_registrations(model: &DesignModel) {
    assert!(!model.electrical_identities.is_empty());
    for (id, record) in &model.electrical_identities {
        assert_eq!(
            model.objects[id].object_revision, record.object_revision,
            "reloading typed electrical identity {id} must not count historical diffs again"
        );
    }
    assert_eq!(
        model
            .objects
            .values()
            .filter(|o| o.domain == "electrical_identity")
            .count(),
        model.electrical_identities.len(),
        "deleted electrical identities leave no registrations"
    );
}

#[test]
fn check_run_after_native_topology_undo_redo_reopen_preserves_authored_basis() {
    let root = temp_project_root("check_run_native_electrical_history");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "ERC history".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = Uuid::new_v4();
    for (verb, params) in [
        (
            "datum.schematic.create_sheet",
            serde_json::json!({"sheet":sheet,"name":"Main"}),
        ),
        (
            "datum.schematic.place_label",
            serde_json::json!({"sheet":sheet,"name":"LEFT","x_nm":0,"y_nm":0}),
        ),
        (
            "datum.schematic.place_label",
            serde_json::json!({"sheet":sheet,"name":"RIGHT","x_nm":100,"y_nm":0}),
        ),
    ] {
        let write = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    let bridge = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(100, 0),
    };
    let write = build_create_schematic_wire(&model, provenance(), sheet, &bridge).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_electrical_registrations(&model);
    let joined = model.electrical_identities.clone();
    let write = build_delete_schematic_wire(&model, provenance(), sheet, &bridge).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_electrical_registrations(&model);
    let split = model.electrical_identities.clone();
    assert_ne!(joined, split);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_electrical_registrations(&model);
    assert_eq!(model.electrical_identities, joined);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_electrical_registrations(&model);
    assert_eq!(model.electrical_identities, split);
    let basis = model.model_revision.clone();
    let objects = model.objects.clone();
    let sources = model.source_shards.clone();
    let check = run(&model);
    let prepared = build_set_check_run(&model, provenance(), &check).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(model.model_revision, basis);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, basis);
    assert_eq!(reopened.objects, objects);
    assert_eq!(reopened.electrical_identities, split);
    assert_eq!(reopened.check_runs[&check.check_run_id], check);
    assert_eq!(
        reopened
            .source_shards
            .iter()
            .filter(|s| s.kind != SourceShardKind::CheckRun)
            .collect::<Vec<_>>(),
        sources.iter().collect::<Vec<_>>()
    );
    // Evidence preservation must not relax the GUI worker's stale-input guard.
    let mut stale = build_set_check_run(&reopened, provenance(), &run(&reopened)).unwrap();
    stale.batch.expected_model_revision = Some(ModelRevision("stale".into()));
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_run_retains_accepted_legacy_revision_but_mixed_design_batch_recomputes() {
    let root = temp_project_root("check_run_legacy_accepted_basis");
    let project = Uuid::new_v4();
    write_minimal_project(&root, project, Uuid::new_v4());
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    // Older journals retain accepted hashes made with inflated registrations.
    // Saving check evidence must not silently migrate that historical basis.
    let legacy = ModelRevision("a".repeat(64));
    model.model_revision = legacy.clone();
    let check = run(&model);
    let prepared = build_set_check_run(&model, provenance(), &check).unwrap();
    let mut report = model.commit_without_direct_policy(prepared.batch).unwrap();
    super::super::commit::refresh_journaled_report_revision(&mut model, &mut report, None).unwrap();
    assert_eq!(model.model_revision, legacy);
    assert_eq!(report.transaction.after_model_revision, legacy);
    let explicit = ModelRevision("b".repeat(64));
    super::super::commit::refresh_journaled_report_revision(
        &mut model,
        &mut report,
        Some(explicit.clone()),
    )
    .unwrap();
    assert_eq!(
        model.model_revision, explicit,
        "explicit history override retains precedence"
    );
    let mut mixed = build_set_check_run(&model, provenance(), &run(&model)).unwrap();
    mixed
        .batch
        .operations
        .push(Operation::BumpObjectRevision { object_id: project });
    let mut report = model.commit_without_direct_policy(mixed.batch).unwrap();
    super::super::commit::refresh_journaled_report_revision(&mut model, &mut report, None).unwrap();
    assert_ne!(model.model_revision, explicit);
    assert_eq!(
        model.model_revision,
        super::super::compute_model_revision(&project, &model.source_shards, &model.objects)
    );
    std::fs::remove_dir_all(root).unwrap();
}
