//! Existing PM057 same-binding allowlist across more than the physical journal tip.
use super::*;
fn assert_electrical_epochs(model: &DesignModel) {
    for (id, record) in &model.electrical_identities {
        assert_eq!(model.objects[id].object_revision, record.object_revision);
    }
}
#[test]
fn journal_multistep_materialization_keeps_current_ci_pad_net_references() {
    let (mut f, mut request, expected, expected_run) = setup();
    let ci = request.component_instance.unwrap();
    assert_electrical_epochs(&f.model);
    let mut snapshots = vec![authored_bytes(&f.root, &f.model)];
    for _ in 0..2 {
        let owner =
            source::capture_root(&f.model, request.placement.package, T::PoolPackage).unwrap();
        for pair in &mut request.correspondences {
            pair.target_library_pad =
                source::capture_pad(&f.model, &owner, pair.target_library_pad.object_id).unwrap();
        }
        let write =
            build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        assert_electrical_epochs(&f.model);
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_electrical_epochs(&f.model);
        check(&f.model, &request, f.logical, &expected, &expected_run);
        snapshots.push(authored_bytes(&f.root, &f.model));
    }
    let prefix = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    for index in (0..2).rev() {
        let electrical = next_electrical_revisions(&f.model, ci);
        let epochs = next_bound_epochs(&f.model, &request, &expected);
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        assert_history_snapshot(
            &snapshots[index],
            &authored_bytes(&f.root, &f.model),
            &f.model,
            ci,
            &electrical,
        );
        assert_electrical_epochs(&f.model);
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_electrical_epochs(&f.model);
        assert_bound_epochs(&f.model, &epochs);
        check(&f.model, &request, f.logical, &expected, &expected_run);
    }
    for index in 1..=2 {
        let electrical = next_electrical_revisions(&f.model, ci);
        let epochs = next_bound_epochs(&f.model, &request, &expected);
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        assert_history_snapshot(
            &snapshots[index],
            &authored_bytes(&f.root, &f.model),
            &f.model,
            ci,
            &electrical,
        );
        assert_electrical_epochs(&f.model);
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_electrical_epochs(&f.model);
        assert_bound_epochs(&f.model, &epochs);
        check(&f.model, &request, f.logical, &expected, &expected_run);
    }
    assert!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root))
            .unwrap()
            .starts_with(&prefix)
    );
    tamper::refuse(&f.root, &f.model);
    std::fs::remove_dir_all(f.root).unwrap();
}
