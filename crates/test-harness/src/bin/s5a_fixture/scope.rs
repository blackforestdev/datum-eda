//! Two real Projects deliberately reuse source-object UUIDs; Project identity
//! and independently authored quantity contexts remain the selection boundary.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{self, WriteProvenance, project},
    ir::units::{
        LengthPrecisionChoice, LengthUnit, LengthUnitChoice, PreFeatureProjectUnitsMigration,
        migrate_pre_feature_project_units, profile_to_descriptor_values,
    },
    substrate::{CommitSource, ProjectResolver},
};
use std::path::Path;
fn configure(root: &Path, unit: LengthUnit) -> Result<serde_json::Value> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        migrate_pre_feature_project_units(None).map_err(|e| anyhow::anyhow!("migration: {e:?}"))?
    else {
        anyhow::bail!("fresh fixture must obtain real migration receipt");
    };
    let provenance = || {
        WriteProvenance::new(
            "s5a-fixture",
            CommitSource::Test,
            "preconfigured native quantity context",
        )
    };
    let write =
        project::build_initialize_project_display_units(&model, provenance(), profile, &receipt)?;
    native_write::commit_prepared(&mut model, root, write)?;
    let mut configured = profile;
    configured.board.unit = LengthUnitChoice::Explicit(unit);
    configured.board.precision = LengthPrecisionChoice::DecimalPlaces(3);
    let write = project::build_set_project_display_units(&model, provenance(), configured)?;
    native_write::commit_prepared(&mut model, root, write)?;
    let reopened = ProjectResolver::new(root).resolve()?;
    ensure!(
        project::project_display_units(&reopened)? == configured,
        "reopen retains exact authored profile"
    );
    let files: Vec<_> = reopened
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(root);
    let bytes = std::fs::read(&journal)?;
    let authority = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("snapshot: {e:?}"))?;
    ensure!(
        authority.project_id() == reopened.project.project_id,
        "actual Project authority"
    );
    for (path, before) in files {
        ensure!(std::fs::read(path)? == before, "read-only source equality");
    }
    ensure!(
        std::fs::read(journal)? == bytes,
        "read-only journal equality"
    );
    Ok(
        serde_json::json!({"path":root,"project_id":reopened.project.project_id,"revision":reopened.model_revision.0,
        "profile":profile_to_descriptor_values(configured),"nonmutation":"complete source/journal bytes around snapshot and reopen"}),
    )
}
pub(super) fn build(root: &Path) -> Result<serde_json::Value> {
    let second = root.join("variants/second-project");
    std::fs::create_dir_all(root.join("variants"))?;
    let report = super::recipe::build_with_project_group(&second, 1, 0x24000000)?;
    let first = configure(root, LengthUnit::Millimeter)?;
    let second = configure(&second, LengthUnit::Mil)?;
    ensure!(
        first["project_id"] != second["project_id"],
        "separate real Projects"
    );
    Ok(
        serde_json::json!({"projects":[first,second],"second_engine_oracles":report,
        "same_authored_track_ids":["40000000-0000-4000-8000-000000000001","40000000-0000-4000-8000-000000000002"],
        "widths_nm":[5080000,5080001],"expected_common_width_displays":["5.080mm","200.000mil"],
        "native_input":"pending duplicate/mixed panes, scope focus, visibility and restored-consumer replay",
        "acceptance":"fixture/protocol readiness only; C04/C05/C06 pending"}),
    )
}
