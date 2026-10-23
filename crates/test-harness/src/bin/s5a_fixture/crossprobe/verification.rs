//! Independent expected source sets and complete canonical read-only snapshots.
use super::*;
use crate::source_history::{preserve_stage, source_bytes};
use eda_engine::substrate::{
    ElectricalQueryFailure, ModelRevision, NetCorrespondenceStatus, net_correspondence_status,
};
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    root: &Path,
    project: &Path,
    model: &mut DesignModel,
    placed: &placement::Placement,
    logical: Uuid,
    wire: &SchematicWire,
    label: &NetLabel,
    tracks: &[Track; 3],
) -> Result<()> {
    let expected: BTreeSet<_> = [
        occurrence("pins", placed.symbol.pins[0].uuid),
        occurrence("wires", wire.uuid),
        occurrence("labels", label.uuid),
        occurrence("pads", placed.pad),
        occurrence("tracks", tracks[0].uuid),
        occurrence("tracks", tracks[1].uuid),
    ]
    .into_iter()
    .collect();
    let complete = verify_complete(project, model, logical, &expected)?;
    let captured = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| anyhow::anyhow!("physical Run capture: {e:?}"))?;
    let physical = captured
        .board_run(
            captured.revision(),
            &occurrence("tracks", tracks[0].uuid),
            None,
        )
        .map_err(|e| anyhow::anyhow!("physical Run: {e:?}"))?;
    ensure!(
        physical.members
            == BTreeSet::from([
                occurrence("pads", placed.pad),
                occurrence("tracks", tracks[0].uuid),
            ]),
        "exact connected Pad/Track Run excludes disconnected and foreign Tracks"
    );
    let ci = &model.component_instances[&placed.instance];
    ensure!(
        ci.placed_symbol_refs == vec![placed.symbol.uuid]
            && ci.placed_package_refs == vec![placed.package],
        "exact persisted related-only mapping"
    );
    ensure!(
        !model
            .component_instances
            .values()
            .any(|ci| ci.placed_symbol_refs.contains(&placed.unlinked)),
        "unlinked negative stays unbound"
    );
    let mut stages = vec![preserve_stage(root, project, "complete", model)?];
    // Retain a real Pending source variant, not an in-memory fake status.
    let original = model.electrical_identities[&id(40)].clone();
    let mut pending = original.clone();
    if let ElectricalIdentity::NetRelationship { intent, .. } = &mut pending.identity {
        *intent = NetRelationshipIntent::Pending;
    }
    let write = electrical::build_set_electrical_identity(model, provenance(), pending)?;
    native_write::commit_prepared(model, project, write)?;
    let before = source_bytes(project, project)?;
    let reopened = ProjectResolver::new(project).resolve()?;
    let captured = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("Pending capture: {e:?}"))?;
    for origin in &expected {
        let result = captured.net_for_origin(captured.revision(), origin);
        let refused = if origin.class == "pads" {
            matches!(
                result,
                Err(ElectricalQueryFailure::UnavailableAssignment { .. })
            )
        } else {
            matches!(
                result,
                Err(ElectricalQueryFailure::UnavailableBinding {
                    status: NetCorrespondenceStatus::Pending,
                    ..
                })
            )
        };
        ensure!(refused, "Pending exact origin refusal: {origin:?}");
    }
    ensure!(
        source_bytes(project, project)? == before,
        "Pending queries/reopen preserve all source/history"
    );
    stages.push(preserve_stage(root, project, "pending", model)?);
    // Every independently persisted variant resumes from its resolved durable
    // tip. Never reuse a commit-returned object's revision cache as a new guard.
    let pending_revision_basis = serde_json::json!({
        "commit_returned_object_revision":model.objects[&id(40)].object_revision,
        "commit_returned_record_revision":model.electrical_identities[&id(40)].object_revision,
        "resolved_object_revision":reopened.objects[&id(40)].object_revision,
        "resolved_record_revision":reopened.electrical_identities[&id(40)].object_revision
    });
    println!("Pending restore source basis: {pending_revision_basis}");
    *model = reopened;
    let write = electrical::build_set_electrical_identity(model, provenance(), original)?;
    native_write::commit_prepared(model, project, write)?;
    let final_complete = verify_complete(project, model, logical, &expected)?;
    let files = source_bytes(project, project)?;
    let report = serde_json::json!({
        "schema_version":1, "kind":"crossprobe_preparation", "project_root":project,
        "project_id":model.project.project_id, "model_revision":model.model_revision,
        "logical_net":logical, "expected_members":expected,
        "foreign_same_name_track":occurrence("tracks",tracks[2].uuid),
        "disconnected_same_net_track":occurrence("tracks",tracks[1].uuid),
        "related_only":{"symbol":placed.symbol.uuid,"footprint":placed.package,"component_instance":placed.instance},
        "connected_board_run":{
            "project_id":physical.project_id,"revision":physical.revision,
            "origin":physical.origin,"board_net":physical.board_net,"members":physical.members,
            "copper_layers":physical.copper_layers.iter().map(|(source,layers)|
                serde_json::json!({"source":source,"layers":layers})).collect::<Vec<_>>(),
            "contact_evidence":physical.contact_evidence
        },
        "unlinked_symbol":placed.unlinked, "complete_setup":complete,"final_complete":final_complete,
        "stages":stages,
        "pending_restore_revision_basis":pending_revision_basis,
        "source_inventory_bytes":files.iter().map(|(p,b)|(p.to_string_lossy().into_owned(), b.len())).collect::<std::collections::BTreeMap<_,_>>(),
        "negative_scope":"Real Pending source stage, absent semantic identity, stale expected revision and unlinked Symbol; no Mismatch/source-deletion or GUI input PASS asserted",
        "status":"Fixture/source-query preparation only; ordinary GUI input, cross-pane rendering/Inspector, accessibility, native replay and acceptance remain pending"
    });
    std::fs::write(
        root.join("crossprobe-fixture.json"),
        eda_test_harness::canonical_json(&report)?,
    )?;
    println!("{}", root.join("crossprobe-fixture.json").display());
    Ok(())
}

fn verify_complete(
    project: &Path,
    model: &DesignModel,
    logical: Uuid,
    expected: &BTreeSet<ElectricalOccurrence>,
) -> Result<serde_json::Value> {
    let before = source_bytes(project, project)?;
    let reopened = ProjectResolver::new(project).resolve()?;
    ensure!(
        reopened.model_revision == model.model_revision,
        "exact persisted revision"
    );
    ensure!(
        net_correspondence_status(&reopened, logical) == NetCorrespondenceStatus::Complete,
        "explicit Complete correspondence after reopen"
    );
    let captured = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("Complete capture: {e:?}"))?;
    let net = captured
        .global_net(captured.revision(), logical)
        .map_err(|e| anyhow::anyhow!("Complete Net: {e:?}"))?;
    ensure!(
        net.owned == *expected,
        "independent complete six-member cross-domain set"
    );
    for origin in expected {
        let result = captured
            .net_for_origin(captured.revision(), origin)
            .map_err(|e| anyhow::anyhow!("origin {origin:?}: {e:?}"))?;
        ensure!(
            result.subject_id == logical && result.owned == *expected,
            "same identity and complete members from both domains"
        );
    }
    ensure!(
        matches!(
            captured.global_net(&ModelRevision("wrong-revision".into()), logical),
            Err(ElectricalQueryFailure::StaleRevision)
        ),
        "stale basis refuses"
    );
    ensure!(
        matches!(
            captured.global_net(captured.revision(), id(99)),
            Err(ElectricalQueryFailure::AbsentIdentity { .. })
        ),
        "absent identity refuses"
    );
    ensure!(
        source_bytes(project, project)? == before,
        "all source/history paths and bytes preserved around queries/reopen"
    );
    Ok(
        serde_json::json!({"revision":reopened.model_revision,"source_basis":captured.source_basis(),"membership":net,"all_six_origins_same_identity":true,"nonmutation":true}),
    )
}
