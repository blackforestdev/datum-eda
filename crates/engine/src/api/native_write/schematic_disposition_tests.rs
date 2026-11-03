//! Complementary adapter refusals; CLI tests use the actual persisted ERC producer.
use super::*;
use crate::{
    api::native_write::{commit_prepared, electrical::topology_tests},
    substrate::{CheckRun, CommitSource},
};
use serde_json::json;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "disposition refusal fixture")
}
fn evidence(model: &DesignModel) -> CheckRun {
    serde_json::from_value(json!({
        "check_run_id":Uuid::new_v4(), "project_id":model.project.project_id,
        "model_revision":model.model_revision, "profile_id":"erc", "status":"fail",
        "summary":{}, "finding_count":1, "proposal_refs":[], "raw_report":{},
        "findings":[{"finding_id":Uuid::new_v4(),"index":0,"source":"erc",
            "code":"fixture", "severity":"warning", "fingerprint":"fixture:fingerprint",
            "domain":"erc", "rule_id":"fixture", "status":"active", "message":"Fixture",
            "payload":{}, "proposal_refs":[]}]
    }))
    .unwrap()
}

#[test]
fn native_erc_disposition_refuses_unqualified_findings_without_writes() {
    let (root, mut model, _, _) = topology_tests::fixture("erc-disposition-admission");
    let run = evidence(&model);
    let id = run.check_run_id;
    let fp = run.findings[0].fingerprint.clone();
    model.check_runs.insert(id, run.clone());
    let before = super::root(&model).unwrap();
    let len = model.journal.len();
    let try_build = |model: &DesignModel, id, fp: &str, reason: &str| {
        build(model, provenance(), id, fp, DispositionKind::Waiver, reason)
    };
    assert!(try_build(&model, Uuid::new_v4(), &fp, "reason").is_err());
    assert!(try_build(&model, id, "unknown", "reason").is_err());
    assert!(try_build(&model, id, &fp, " \t ").is_err());
    for case in 0..6 {
        let mut invalid = run.clone();
        match case {
            0 => invalid.project_id = Uuid::new_v4(),
            1 => invalid.model_revision.0.push_str("-stale"),
            2 => invalid.findings[0].domain = "drc".into(),
            3 => invalid.findings.push(invalid.findings[0].clone()),
            4 => invalid.findings[0].status = "waived".into(),
            _ => invalid.findings[0].deviation_refs.push(Uuid::new_v4()),
        }
        model.check_runs.insert(id, invalid);
        assert!(try_build(&model, id, &fp, "reason").is_err(), "case {case}");
    }
    model.check_runs.insert(id, run);
    assert!(try_build(&model, id, &fp, "reason").is_ok());
    assert_eq!(super::root(&model).unwrap(), before);
    assert_eq!(model.journal.len(), len);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_erc_disposition_checks_root_duplicates_despite_empty_finding_refs() {
    for kind in [DispositionKind::Waiver, DispositionKind::Deviation] {
        let (root, mut model, _, _) = topology_tests::fixture("erc-disposition-existing");
        let mut run = evidence(&model);
        let id = run.check_run_id;
        let fp = run.findings[0].fingerprint.clone();
        model.check_runs.insert(id, run.clone());
        let write = build(&model, provenance(), id, &fp, kind, "existing reason").unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        // Deliberately stale presentation annotations cannot authorize a replacement.
        run.model_revision = model.model_revision.clone();
        model.check_runs.insert(id, run);
        let before = super::root(&model).unwrap();
        let len = model.journal.len();
        for replacement in [DispositionKind::Waiver, DispositionKind::Deviation] {
            assert!(
                build(&model, provenance(), id, &fp, replacement, "replacement")
                    .unwrap_err()
                    .to_string()
                    .contains("already has")
            );
        }
        assert_eq!(super::root(&model).unwrap(), before);
        assert_eq!(model.journal.len(), len);
        std::fs::remove_dir_all(root).unwrap();
    }
}
