//! Canonical atomic creation, refusal and cold journal history.
use super::super::{
    electrical::topology_tests,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use super::*;
use crate::substrate::ProjectResolver;
use std::path::Path;
fn request(parent: Uuid) -> ChildSheetRequest {
    ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Blank child".into(),
        position: Point::new(12_000_000, 25_000_000),
    }
}
fn act(
    model: &mut DesignModel,
    root: &Path,
    action: Action,
) -> Result<proposal::NativeProposalResponse, EngineError> {
    let request = NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    };
    proposal::execute(model, &request)
}
#[test]
fn blank_child_proposal_cancel_atomic_history_and_cold_reopen() {
    let (root, mut model, _, parent) = topology_tests::fixture("blank_child_history");
    let before = source(&model).unwrap();
    let mut r = request(parent);
    let expected = preview(&model, &r).unwrap();
    let draft = act(&mut model, &root, Action::CreateChildSheet(r.clone())).unwrap();
    assert_eq!(draft.child_sheet, Some(expected));
    assert_eq!(source(&model).unwrap(), before);
    assert!(
        !root
            .join(format!("schematic/sheets/{}.json", r.sheet))
            .exists()
    );
    act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: r.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(source(&model).unwrap(), before);
    r.proposal_id = Uuid::new_v4();
    act(&mut model, &root, Action::CreateChildSheet(r.clone())).unwrap();
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: r.proposal_id,
        },
    )
    .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source(&model).unwrap();
    assert_eq!(after.sheets[&parent], before.sheets[&parent]);
    assert_eq!(after.sheet_definitions[&r.definition], definition(&r));
    assert_eq!(after.sheet_instances[&r.instance], instance(&r));
    assert!(after.sheets[&r.sheet].symbols.is_empty());
    let sheet_file = root.join(format!("schematic/sheets/{}.json", r.sheet));
    let def_file = root.join(format!("schematic/definitions/{}.json", r.definition));
    let sheet_bytes = std::fs::read(&sheet_file).unwrap();
    let def_bytes = std::fs::read(&def_file).unwrap();
    let provenance = WriteProvenance::new("test", CommitSource::Manual, "blank child history");
    for _ in 0..2 {
        model
            .commit_journal_undo(&root, provenance.clone().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model).unwrap(), before);
        assert!(!sheet_file.exists());
        assert!(!def_file.exists());
        model
            .commit_journal_redo(&root, provenance.clone().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(source(&model).unwrap(), after);
        assert_eq!(std::fs::read(&sheet_file).unwrap(), sheet_bytes);
        assert_eq!(std::fs::read(&def_file).unwrap(), def_bytes);
    }
    // The new empty qualified child can itself be a unique parent.
    let mut nested = request(r.sheet);
    nested.instance_path = vec![r.instance];
    assert!(preview(&model, &nested).is_ok());
    nested.instance_path.clear();
    assert!(preview(&model, &nested).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn blank_child_refuses_invalid_stale_and_tampered_batches() {
    let (root, mut model, _, parent) = topology_tests::fixture("blank_child_refusal");
    let before = source(&model).unwrap();
    let r = request(parent);
    for bad in [
        ChildSheetRequest {
            parent: Uuid::new_v4(),
            ..r.clone()
        },
        ChildSheetRequest {
            sheet: parent,
            ..r.clone()
        },
        ChildSheetRequest {
            definition: r.sheet,
            ..r.clone()
        },
        ChildSheetRequest {
            name: " \n".into(),
            ..r.clone()
        },
        ChildSheetRequest {
            instance_path: vec![Uuid::new_v4()],
            ..r.clone()
        },
    ] {
        assert!(act(&mut model, &root, Action::CreateChildSheet(bad)).is_err());
        assert_eq!(source(&model).unwrap(), before);
    }
    act(&mut model, &root, Action::CreateChildSheet(r.clone())).unwrap();
    let mut tampered = model.proposals[&r.proposal_id].clone();
    tampered
        .batch
        .operations
        .push(Operation::CreateSchematicSheetInstance {
            schematic_id: before.uuid,
            instance_id: Uuid::new_v4(),
            instance: json!({}),
        });
    assert!(validate_recorded(&tampered).is_err());
    let other = request(parent);
    act(&mut model, &root, Action::CreateChildSheet(other.clone())).unwrap();
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: other.proposal_id,
        },
    )
    .unwrap();
    let changed = source(&model).unwrap();
    assert!(
        act(
            &mut model,
            &root,
            Action::AcceptApply {
                proposal_id: r.proposal_id
            }
        )
        .is_err()
    );
    assert_eq!(source(&model).unwrap(), changed);
    assert!(
        !root
            .join(format!("schematic/sheets/{}.json", r.sheet))
            .exists()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn blank_child_accept_refuses_new_destination_without_overwriting_it() {
    for definition in [false, true] {
        let (root, mut model, _, parent) = topology_tests::fixture("blank_child_destination");
        let r = request(parent);
        let before = source(&model).unwrap();
        act(&mut model, &root, Action::CreateChildSheet(r.clone())).unwrap();
        let relative = if definition {
            format!("definitions/{}.json", r.definition)
        } else {
            format!("sheets/{}.json", r.sheet)
        };
        let path = root.join("schematic").join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"unrelated file created after review").unwrap();
        let tip = crate::revision::transaction_tip(&model.journal);
        assert!(
            act(
                &mut model,
                &root,
                Action::AcceptApply {
                    proposal_id: r.proposal_id
                }
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"unrelated file created after review"
        );
        assert_eq!(source(&model).unwrap(), before);
        assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
        assert_eq!(
            model.proposals[&r.proposal_id].status,
            crate::substrate::ProposalStatus::Draft
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn blank_child_refuses_repeated_parent_and_changed_review_payloads() {
    let (root, mut model, schematic, parent) = topology_tests::fixture("blank_child_repeated");
    let r = request(parent);
    act(&mut model, &root, Action::CreateChildSheet(r.clone())).unwrap();
    let original = model.proposals[&r.proposal_id].clone();
    for kind in 0..3 {
        let mut changed = original.clone();
        for op in &mut changed.batch.operations {
            match (kind, op) {
                (0, Operation::CreateSchematicSheet { sheet, .. }) => {
                    sheet["name"] = json!("Changed after review")
                }
                (1, Operation::CreateSchematicDefinition { definition, .. }) => {
                    definition["root_sheet"] = json!(parent)
                }
                (2, Operation::CreateSchematicSheetInstance { instance, .. }) => {
                    instance["parent_sheet"] = json!(Uuid::new_v4())
                }
                _ => {}
            }
        }
        model.proposals.insert(r.proposal_id, changed);
        assert!(
            act(
                &mut model,
                &root,
                Action::AcceptApply {
                    proposal_id: r.proposal_id
                }
            )
            .is_err()
        );
        assert!(!source(&model).unwrap().sheets.contains_key(&r.sheet));
    }
    model.proposals.insert(r.proposal_id, original);
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: r.proposal_id,
        },
    )
    .unwrap();
    let mut sibling = instance(&r);
    sibling.uuid = Uuid::new_v4();
    sibling.name = "Second occurrence".into();
    let write = schematic_sheets::build_create_schematic_sheet_instance(
        &model,
        WriteProvenance::new("test", CommitSource::Manual, "repeat existing source"),
        schematic,
        sibling.uuid,
        serde_json::to_value(sibling).unwrap(),
    )
    .unwrap();
    super::super::commit_prepared(&mut model, &root, write).unwrap();
    let mut nested = request(r.sheet);
    nested.instance_path = vec![r.instance];
    let before = source(&model).unwrap();
    assert!(act(&mut model, &root, Action::CreateChildSheet(nested)).is_err());
    assert_eq!(source(&model).unwrap(), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires DATUM_CHILD_SHEET_HISTORY_RUN stopped canonical run76"]
fn real_child_history_then_proposal_metadata_preserves_prepared_revision() {
    use sha2::{Digest, Sha256};
    let run = std::path::PathBuf::from(
        std::env::var_os("DATUM_CHILD_SHEET_HISTORY_RUN").expect("stopped real GUI project"),
    );
    let inventory: std::collections::BTreeMap<String, String> =
        serde_json::from_slice(&std::fs::read(run.join("stopped-files.json")).unwrap()).unwrap();
    let root = super::super::test_support::temp_project_root("real_child_metadata_history");
    for (relative, expected) in &inventory {
        let bytes = std::fs::read(run.join("project-one").join(relative)).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), *expected);
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, bytes).unwrap();
    }
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let initial = source(&model).unwrap();
    assert_eq!(initial.sheet_instances.len(), 1);
    let placed = initial.sheet_instances.values().next().unwrap().clone();
    let child = initial.sheet_definitions[&placed.definition].root_sheet;
    let provenance =
        WriteProvenance::new("test", CommitSource::Manual, "real child metadata history");
    model
        .commit_journal_undo(&root, provenance.clone().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        matches!(
            model.journal.last().unwrap().operations.as_slice(),
            [Operation::DeleteProposalMetadata { .. }]
        ),
        "first undo removes the retained failed Draft metadata"
    );
    assert_eq!(source(&model).unwrap(), initial);
    model
        .commit_journal_undo(&root, provenance.clone().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(source(&model).unwrap().sheet_instances.is_empty());
    model
        .commit_journal_redo(&root, provenance.clone().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model).unwrap(), initial);
    let prepared = model.model_revision.clone();
    let mut nested = request(child);
    nested.instance_path = vec![placed.uuid];
    let draft = act(&mut model, &root, Action::CreateChildSheet(nested.clone())).unwrap();
    assert_eq!(
        model.model_revision, prepared,
        "Draft metadata cannot migrate a history basis"
    );
    assert_eq!(draft.prepared_against, prepared.0);
    let metadata = model.journal.last().unwrap();
    assert_eq!(
        metadata.before_model_revision,
        metadata.after_model_revision
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.model_revision, prepared,
        "cold metadata replay retains the authored basis"
    );
    act(
        &mut model,
        &root,
        Action::Preview {
            proposal_id: nested.proposal_id,
        },
    )
    .unwrap();
    act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: nested.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(model.model_revision, prepared);
    assert_eq!(source(&model).unwrap(), initial);
    nested.proposal_id = Uuid::new_v4();
    act(&mut model, &root, Action::CreateChildSheet(nested.clone())).unwrap();
    let before_accept = model.journal.len();
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: nested.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(model.journal.len(), before_accept + 2);
    let accepted = &model.journal[before_accept];
    assert_eq!(accepted.before_model_revision, prepared);
    assert_eq!(accepted.after_model_revision, prepared);
    let design = &model.journal[before_accept + 1];
    assert_ne!(
        design.after_model_revision, prepared,
        "mixed design/Proposal batch must change authored revision"
    );
    assert!(
        design
            .operations
            .iter()
            .any(|op| matches!(op, Operation::CreateSchematicSheet { .. }))
    );
    assert!(
        design
            .operations
            .iter()
            .any(|op| matches!(op, Operation::SetProposalMetadata { .. }))
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source(&model).unwrap();
    assert_eq!(after.sheet_instances.len(), 2);
    assert_eq!(
        after.sheet_instances[&nested.instance].parent_sheet,
        Some(child)
    );
    assert_eq!(
        sheet_occurrences(&after).unwrap()[&nested.sheet],
        vec![vec![placed.uuid, nested.instance]]
    );
    for (id, sheet) in &initial.sheets {
        assert_eq!(after.sheets[id], *sheet);
    }
    model
        .commit_journal_undo(&root, provenance.clone().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model).unwrap(), initial);
    model.commit_journal_redo(&root, provenance.into()).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model).unwrap(), after);
    for (relative, expected) in &inventory {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(run.join("project-one").join(relative)).unwrap())
            ),
            *expected
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
