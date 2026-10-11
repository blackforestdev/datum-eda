//! Persisted prepared allocations are hypothetical until reviewed application.
use super::tests::{fixture, provenance};
use super::*;
use crate::substrate::{
    ProjectResolver, ProposalCreateRequest, ProposalSource, ProposalStatus,
    apply_accepted_proposal, create_draft_proposal_from_batch, preview_proposal_diff_journaled,
    review_proposal_status,
};

#[test]
fn process_prepared_allocations_reopen_preserves_design_basis_until_reviewed_apply() {
    use crate::api::native_write::{
        commit_prepared,
        library::{
            PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
            pool_library_object_payload,
        },
        process::build_adopt_process_capability,
    };
    use crate::pool::process::{ProcessFace, ProcessSourceCapability};
    let (root, mut model, request, library) = fixture();
    let raw_package = pool_library_object_payload(
        &model,
        &format!("pool/packages/{}.json", request.placement.package),
        request.placement.package,
    )
    .unwrap();
    let stack: Uuid =
        serde_json::from_value(raw_package["pads"][library.to_string()]["padstack"].clone())
            .unwrap();
    let mut raw_stack =
        pool_library_object_payload(&model, &format!("pool/padstacks/{stack}.json"), stack)
            .unwrap();
    raw_stack["mask_policy"] = serde_json::json!("tented");
    raw_stack["paste_policy"] = serde_json::json!("none");
    let authored = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "padstacks", stack),
            object: raw_stack,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, authored).unwrap();
    let capability = ProcessSourceCapability {
        version: 1,
        front: ProcessFace {
            copper_layer: 1,
            mask_layer: None,
            paste_layer: None,
        },
        back: ProcessFace {
            copper_layer: 2,
            mask_layer: None,
            paste_layer: None,
        },
    };
    let adopted = build_adopt_process_capability(&model, &root, provenance(), capability).unwrap();
    commit_prepared(&mut model, &root, adopted).unwrap();
    let prepared = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    let allocated = prepared
        .batch
        .operations
        .iter()
        .find_map(|op| match op {
            Operation::SetBoardPackagePackage { materialized, .. } => {
                materialized["component_pads"]
                    .as_array()
                    .and_then(|pads| pads.first())
                    .map(|p| serde_json::from_value::<Uuid>(p["uuid"].clone()).unwrap())
            }
            _ => None,
        })
        .unwrap();
    assert!(!model.objects.contains_key(&allocated));
    let basis = model.model_revision.clone();
    let objects = model.objects.clone();
    let diagnostics = serde_json::to_value(&model.diagnostics).unwrap();
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: prepared.batch,
            rationale: "Explicit complete allocation review".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, basis);
    assert_eq!(reopened.objects, objects);
    assert!(!reopened.objects.contains_key(&allocated));
    preview_proposal_diff_journaled(&reopened, &root, proposal.proposal_id).unwrap();
    review_proposal_status(
        &mut reopened,
        &root,
        proposal.proposal_id,
        ProposalStatus::Accepted,
    )
    .unwrap();
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, basis);
    assert_eq!(reopened.objects, objects);
    let applied = apply_accepted_proposal(&mut reopened, &root, proposal.proposal_id).unwrap();
    assert_eq!(applied.transaction.diff.created, vec![allocated]);
    let applied_objects = reopened.objects.clone();
    let applied_revision = reopened.model_revision.clone();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.objects, applied_objects);
    assert_eq!(reopened.model_revision, applied_revision);
    // This existing board-only fixture reports its unmatched schematic instance.
    // Reopening the applied process snapshot must introduce no new diagnostics.
    assert_eq!(
        serde_json::to_value(&reopened.diagnostics).unwrap(),
        diagnostics
    );
    assert_eq!(reopened.objects[&allocated].domain, "board");
    for (id, object) in &objects {
        assert_eq!(reopened.objects[id].source_shard_id, object.source_shard_id);
        assert_eq!(reopened.objects[id].domain, object.domain);
    }
    assert_ne!(reopened.model_revision, basis);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn process_bound_admission_rejects_omitted_or_null_pad_array_before_any_write() {
    use crate::api::native_write::commit_prepared;
    let (root, mut model, mut request, library) = fixture();
    let prepared = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let placed: Uuid = serde_json::from_value(
        board["component_pads"][request.placement.uuid.to_string()][0]["uuid"].clone(),
    )
    .unwrap();
    request.correspondences = vec![PlacedPadCorrespondence {
        placed_pad_id: placed,
        target_library_pad: capture_component_library_pad(
            &model,
            request.placement.package,
            &request.land_pattern,
            library,
        )
        .unwrap(),
    }];
    let prepared = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    let before_board = std::fs::read(root.join("board/board.json")).unwrap();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let before_journal = std::fs::read(&journal_path).unwrap();
    let before_objects = model.objects.clone();
    for null in [false, true] {
        let mut forged = prepared.clone();
        for operation in &mut forged.batch.operations {
            if let Operation::SetBoardPackagePackage { materialized, .. } = operation {
                if null {
                    materialized["component_pads"] = serde_json::Value::Null;
                } else {
                    materialized
                        .as_object_mut()
                        .unwrap()
                        .remove("component_pads");
                }
            }
        }
        let error = commit_prepared(&mut model, &root, forged)
            .unwrap_err()
            .to_string();
        let expected = if null {
            "invalid type: null, expected a sequence"
        } else {
            "explicit component_pads array"
        };
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            before_board
        );
        assert_eq!(std::fs::read(&journal_path).unwrap(), before_journal);
        assert_eq!(model.objects, before_objects);
    }
    let _ = std::fs::remove_dir_all(root);
}
