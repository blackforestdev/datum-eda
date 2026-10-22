//! One declared singleton Net for native lifetime observation, not acceptance.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, WriteProvenance, electrical, schematic_connectivity},
    schematic::NetLabel,
    substrate::{
        CommitSource, ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence,
        ElectricalSelectionSnapshot, NetRelationshipIntent, ObjectRevision, ProjectResolver,
    },
};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-global-net-lifetime-fixture",
        CommitSource::Test,
        "existing semantic recipe: isolate root WireD and declare schematic-only correspondence",
    )
}

pub(super) fn build(root: &Path) -> Result<()> {
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh output required"
    );
    // Reuse the complete canonical core/semantic recipe, not a private writer.
    super::run("semantic-scope", root)?;
    let project = root.join("variants/semantic-scope");
    let mut model = ProjectResolver::new(&project).resolve()?;
    let label_id = Uuid::parse_str("73000000-0000-4000-8000-00000000000e")?;
    let wire_id = Uuid::parse_str("73000000-0000-4000-8000-00000000000d")?;
    let sheet_path = project.join("schematic/selection.json");
    let before_sheet: serde_json::Value = serde_json::from_slice(&std::fs::read(&sheet_path)?)?;
    let sheet_id = Uuid::parse_str(before_sheet["uuid"].as_str().context("sheet identity")?)?;
    let label: NetLabel =
        serde_json::from_value(before_sheet["labels"][label_id.to_string()].clone())?;
    let write = schematic_connectivity::build_delete_schematic_label(
        &model,
        provenance(),
        sheet_id,
        &label,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    let after_sheet: serde_json::Value = serde_json::from_slice(&std::fs::read(&sheet_path)?)?;
    let mut expected_sheet = before_sheet;
    expected_sheet["labels"]
        .as_object_mut()
        .unwrap()
        .remove(&label_id.to_string());
    ensure!(
        after_sheet == expected_sheet,
        "only root LabelE source is deleted"
    );
    let origin = ElectricalOccurrence {
        class: "wires".into(),
        source_id: wire_id,
        instance_path: vec![],
    };
    // Automatic topology composition allocated this actual final-group identity.
    let nets: Vec<_> = model
        .electrical_identities
        .values()
        .filter_map(|record| {
            matches!(&record.identity, ElectricalIdentity::Net { anchor, retired: false, .. }
            if anchor == &origin)
            .then_some(record.id)
        })
        .collect();
    ensure!(nets.len() == 1, "unique allocated root WireD Net");
    let logical = nets[0];
    let relationship = Uuid::parse_str("7e000000-0000-4000-8000-000000000001")?;
    let write = electrical::build_create_electrical_identity(
        &model,
        provenance(),
        ElectricalIdentityRecord {
            id: relationship,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: logical,
                board_net: None,
                intent: NetRelationshipIntent::SchematicOnly,
                evidence: vec![],
            },
        },
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    let revision = model.model_revision.clone();
    let expected = BTreeSet::from([origin.clone()]);
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|shard| Ok((shard.path.clone(), std::fs::read(&shard.path)?)))
        .collect::<Result<_>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(&project);
    before.push((journal.clone(), std::fs::read(journal)?));
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|error| anyhow::anyhow!("capture: {error:?}"))?;
    let global = snapshot
        .global_net(&revision, logical)
        .map_err(|error| anyhow::anyhow!("declared Global Net: {error:?}"))?;
    let from_origin = snapshot
        .net_for_origin(&revision, &origin)
        .map_err(|error| anyhow::anyhow!("native origin binding: {error:?}"))?;
    ensure!(
        global.subject_id == logical && global.owned == expected,
        "complete Global Net must be exactly root WireD"
    );
    ensure!(
        from_origin == global,
        "ordinary origin route resolves same complete declaration"
    );
    let reopened = ProjectResolver::new(&project).resolve()?;
    ensure!(
        reopened.model_revision == revision && model.model_revision == revision,
        "query/reopen preserve actual revision"
    );
    let fresh = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|error| anyhow::anyhow!("reopened capture: {error:?}"))?;
    ensure!(
        fresh
            .global_net(&revision, logical)
            .map_err(|error| anyhow::anyhow!("reopened Global Net: {error:?}"))?
            == global,
        "reopen preserves full singleton membership and correspondence"
    );
    ensure!(
        fresh
            .net_for_origin(&revision, &origin)
            .map_err(|error| anyhow::anyhow!("reopened origin: {error:?}"))?
            == global,
        "reopen preserves public acquisition binding"
    );
    for (path, bytes) in before {
        ensure!(
            std::fs::read(&path)? == bytes,
            "queries changed {}",
            path.display()
        );
    }
    let manifest = serde_json::json!({
        "schema_version":1,"family":"global-net-lifetime",
        "recipe":"existing core+semantic; canonical LabelE deletion and explicit SchematicOnly declaration",
        "project_root":project,"project_id":model.project.project_id,"sheet_id":sheet_id,
        "model_revision":revision,"logical_net_id":logical,"relationship_id":relationship,
        "expected_subject":{"kind":"global_net","identity":logical},
        "expected_members":[{"class":"wire","id":wire_id}],
        "expected_scope":null,"expected_focus":null,
        "origin":{"class":"wire","id":wire_id,"instance_path":[]},
        "source_geometry":after_sheet["wires"][wire_id.to_string()],
        "setup":{"deleted_label":label_id,"declared_intent":"schematic_only",
            "mutation_owner":"existing native_write builders and canonical commit/journal"},
        "engine_observation":{"complete_singleton_equal":true,"origin_binding_equal":true,
            "reopened_equal":true,"source_journal_queries_unchanged":true},
        "native_input":"pending actual source/input/observer review and native acquisition/delete-wire/Undo",
        "acceptance":"not established by fixture generation; C04/C05/C06 remain separate"
    });
    std::fs::write(
        project.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&manifest)?,
    )?;
    println!("{}", project.join("s5a-fixture.json").display());
    Ok(())
}
