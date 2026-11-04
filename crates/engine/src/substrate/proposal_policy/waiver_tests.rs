//! Actual canonical refusal and accepted-disposition history, not policy matching alone.
use crate::api::native_write::{WriteProvenance, waivers::*};
use crate::schematic::CheckDomain;
use crate::substrate::{
    tests::{temp_project_root, write_minimal_project},
    *,
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture() -> (Fixture, DesignModel, Uuid) {
    let root = Fixture(temp_project_root("waiver_proposal_policy"));
    let project = Uuid::new_v4();
    write_minimal_project(&root.0, project, Uuid::new_v4());
    let path = root.0.join("schematic/schematic.json");
    let mut sheet: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    sheet["deviations"] = serde_json::json!([]);
    std::fs::write(
        path,
        format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(&sheet).unwrap()
        ),
    )
    .unwrap();
    let model = ProjectResolver::new(&root.0).resolve().unwrap();
    (root, model, Uuid::new_v5(&project, b"schematic"))
}
fn sources(root: &Path) -> Vec<Vec<u8>> {
    [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .iter()
    .map(|p| std::fs::read(root.join(p)).unwrap())
    .collect()
}
fn batch(
    model: &DesignModel,
    schematic: Uuid,
    source: CommitSource,
    deviation: bool,
) -> (Uuid, OperationBatch) {
    let provenance = WriteProvenance::new("reviewer", source, "explicitly accepted proposal");
    if deviation {
        let prepared = build_create_schematic_deviation(
            model,
            provenance,
            &CreateSchematicDeviationRequest {
                schematic_id: schematic,
                domain: CheckDomain::ERC,
                fingerprint: "erc:finding".into(),
                rationale: "reviewed exception".into(),
                accepted_by: Some("reviewer".into()),
            },
        )
        .unwrap();
        (prepared.deviation_id, prepared.write.batch)
    } else {
        let prepared = build_create_schematic_waiver(
            model,
            provenance,
            &CreateSchematicWaiverRequest {
                schematic_id: schematic,
                domain: CheckDomain::ERC,
                fingerprint: "erc:finding".into(),
                rationale: "reviewed exception".into(),
                created_by: Some("reviewer".into()),
            },
        )
        .unwrap();
        (prepared.waiver_id, prepared.write.batch)
    }
}

#[test]
fn every_source_refuses_direct_disposition_atomically_in_memory_and_on_disk() {
    for source in [
        CommitSource::Manual,
        CommitSource::Cli,
        CommitSource::Tool,
        CommitSource::Assistant,
        CommitSource::Test,
    ] {
        for deviation in [false, true] {
            let (root, mut model, schematic) = fixture();
            let (id, mut batch) = batch(&model, schematic, source, deviation);
            batch.operations.insert(
                0,
                Operation::SetProjectName {
                    project_id: model.project.project_id,
                    name: "must not apply".into(),
                },
            );
            let before = sources(&root.0);
            let revision = model.model_revision.clone();
            let project = serde_json::to_value(&model.project).unwrap();
            let journal = model.journal.clone();
            for journaled in [false, true] {
                let error = if journaled {
                    model.commit_journaled(&root.0, batch.clone())
                } else {
                    model.commit(batch.clone())
                }
                .unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("proposal_required_for_check_disposition"),
                    "{error}"
                );
                assert_eq!(model.model_revision, revision);
                assert_eq!(serde_json::to_value(&model.project).unwrap(), project);
                assert_eq!(model.journal, journal);
                assert!(!model.objects.contains_key(&id));
                assert_eq!(sources(&root.0), before);
                assert!(!transaction_journal_path(&root.0).exists());
            }
        }
    }
}

#[test]
fn accepted_dispositions_keep_preview_cancel_and_compensating_history_after_reopen() {
    for (source, proposal_source) in [
        (CommitSource::Manual, ProposalSource::Manual),
        (CommitSource::Tool, ProposalSource::Tool),
        (CommitSource::Assistant, ProposalSource::Assistant),
    ] {
        for deviation in [false, true] {
            let (root, mut model, schematic) = fixture();
            let before = sources(&root.0);
            let (id, batch) = batch(&model, schematic, source, deviation);
            let request = ProposalCreateRequest {
                proposal_id: None,
                batch,
                rationale: "reviewed exception".into(),
                source: proposal_source,
                checks_run: vec![],
                finding_fingerprints: vec!["erc:finding".into()],
            };
            let cancelled =
                create_draft_proposal_from_batch(&mut model, &root.0, request.clone()).unwrap();
            preview_proposal_diff(&model, cancelled.proposal_id).unwrap();
            preview_proposal_diff_journaled(&model, &root.0, cancelled.proposal_id).unwrap();
            assert_eq!(sources(&root.0), before);
            assert!(!model.objects.contains_key(&id));
            assert!(apply_accepted_proposal(&mut model, &root.0, cancelled.proposal_id).is_err());
            review_proposal_status(
                &mut model,
                &root.0,
                cancelled.proposal_id,
                ProposalStatus::Rejected,
            )
            .unwrap();
            assert_eq!(sources(&root.0), before);
            assert!(apply_accepted_proposal(&mut model, &root.0, cancelled.proposal_id).is_err());
            let accepted = create_draft_proposal_from_batch(&mut model, &root.0, request).unwrap();
            review_proposal_status(
                &mut model,
                &root.0,
                accepted.proposal_id,
                ProposalStatus::Accepted,
            )
            .unwrap();
            let report =
                apply_accepted_proposal(&mut model, &root.0, accepted.proposal_id).unwrap();
            assert_eq!(
                model.proposals[&accepted.proposal_id].applied_transaction_id,
                Some(report.transaction.transaction_id)
            );
            let after = sources(&root.0);
            assert_ne!(after, before);
            model = ProjectResolver::new(&root.0).resolve().unwrap();
            assert!(model.objects.contains_key(&id));
            let history = CommitProvenance {
                actor: "reviewer".into(),
                source,
                reason: "history".into(),
            };
            let count = model.journal.len();
            model.commit_journal_undo(&root.0, history.clone()).unwrap();
            assert_eq!(model.journal.len(), count + 1);
            assert_eq!(sources(&root.0), before);
            model = ProjectResolver::new(&root.0).resolve().unwrap();
            model.commit_journal_redo(&root.0, history).unwrap();
            assert_eq!(model.journal.len(), count + 2);
            assert_eq!(sources(&root.0), after);
            let reopened = ProjectResolver::new(&root.0).resolve().unwrap();
            assert!(reopened.objects.contains_key(&id));
        }
    }
}

#[test]
fn disposition_preview_retains_unjournaled_board_admission() {
    let (root, mut model, schematic) = fixture();
    let (_, batch) = batch(&model, schematic, CommitSource::Manual, false);
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        &root.0,
        ProposalCreateRequest {
            proposal_id: None,
            batch,
            rationale: "preview".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    model
        .proposals
        .get_mut(&proposal.proposal_id)
        .unwrap()
        .batch
        .operations = vec![Operation::AdoptBoardSourceProfile {
        board_id: Uuid::new_v4(),
        preimage: "{}".into(),
    }];
    let before = sources(&root.0);
    let journal = model.journal.clone();
    let error = preview_proposal_diff(&model, proposal.proposal_id).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires the canonical journaled transaction")
    );
    assert_eq!(sources(&root.0), before);
    assert_eq!(model.journal, journal);
}
