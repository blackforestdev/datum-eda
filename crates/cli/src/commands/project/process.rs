//! Thin manual PM061 authoring/inspection adapters over canonical engine primitives.
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::Path;

use crate::args::project_process::{ProcessCommands, ProjectProcessArgs};
use crate::{OutputFormat, render_output};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared, process};
use eda_engine::substrate::{ProjectResolver, SourceShardKind};

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&std::fs::read(path)?)
        .with_context(|| format!("invalid process input {}", path.display()))
}

impl ProjectProcessArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let mut model = ProjectResolver::new(&self.path).resolve()?;
        let provenance = WriteProvenance::new(
            "datum-eda-cli",
            crate::cli_commit_source()?,
            "Explicit native process authoring",
        );
        let prepared = match self.action {
            ProcessCommands::PinPad {
                package,
                footprint,
                pad,
            } => {
                use eda_engine::api::native_write::board_components::pad_materialization::{
                    PadLandPatternSource, capture_component_library_pad,
                };
                let pin = capture_component_library_pad(
                    &model,
                    package,
                    &PadLandPatternSource::Footprint(footprint),
                    pad,
                )?;
                return Ok((
                    render_output(
                        format,
                        &json!({"target_library_pad": pin, "model_revision": model.model_revision}),
                    ),
                    0,
                ));
            }
            ProcessCommands::Profiles => {
                return Ok((
                    render_output(
                        format,
                        &json!({"process_profiles": process::profiles(&model)?, "model_revision": model.model_revision}),
                    ),
                    0,
                ));
            }
            ProcessCommands::SetProfile { profile_json } => process::build_set_process_profile(
                &model,
                provenance,
                serde_json::from_value(read(&profile_json)?)?,
            )?,
            ProcessCommands::DeleteProfile { profile } => {
                process::build_delete_process_profile(&model, provenance, profile)?
            }
            ProcessCommands::SelectBasis {
                footprint,
                basis_json,
                profile,
            } => {
                let selector = match profile {
                    Some(id) => process::pinned_profile_selector(&model, id)?,
                    None => serde_json::from_value(read(
                        &basis_json.expect("clap requires basis or profile"),
                    )?)?,
                };
                process::build_select_process_basis(&model, provenance, footprint, selector)?
            }
            ProcessCommands::Adopt { capability_json } => process::build_adopt_process_capability(
                &model,
                &self.path,
                provenance,
                serde_json::from_value(read(&capability_json)?)?,
            )?,
            ProcessCommands::SetAssociations { capability_json } => {
                process::build_set_process_associations(
                    &model,
                    provenance,
                    serde_json::from_value(read(&capability_json)?)?,
                )?
            }
            ProcessCommands::Inspect {
                footprint,
                padstack,
            } => {
                return Ok((
                    render_output(
                        format,
                        &json!({"footprint_id": footprint, "padstack_id": padstack,
                    "effective_process": process::inspect_effective_process(&model, footprint, padstack)?,
                    "model_revision": model.model_revision}),
                    ),
                    0,
                ));
            }
            ProcessCommands::Placements { component } => {
                let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
                let projection =
                    eda_engine::substrate::component_pad_source::component_pad_projection(
                        &model, &board,
                    )?;
                let mut placements = Vec::new();
                if let Some(groups) = board.get("component_pads").and_then(Value::as_object) {
                    for (placement, pads) in groups {
                        if component.is_some_and(|id| id.to_string() != *placement) {
                            continue;
                        }
                        for raw in pads.as_array().into_iter().flatten() {
                            let pad: eda_engine::substrate::component_pad_source::ComponentPadSource = serde_json::from_value(raw.clone())?;
                            let pose: eda_engine::board::PlacedPackage =
                                serde_json::from_value(board["packages"][placement].clone())?;
                            let inspection = eda_engine::substrate::component_pad_source::process::inspection::inspect(
                                &model, &board, &pad, &pose);
                            let (inspection, inspection_error) = match inspection {
                                Ok(value) => (Some(value), None),
                                Err(error) => (None, Some(error.to_string())),
                            };
                            placements.push(json!({"placement_id": placement, "pad_id": pad.uuid,
                                "source_binding": pad.source_binding, "status": projection.statuses.get(&pad.uuid),
                                "frozen_process": projection.process_pads.get(&pad.uuid), "unavailable": projection.unavailable.get(&pad.uuid),
                                "process_inspection": inspection, "process_inspection_error": inspection_error}));
                        }
                    }
                }
                return Ok((
                    render_output(
                        format,
                        &json!({"placements": placements, "model_revision": model.model_revision}),
                    ),
                    0,
                ));
            }
        };
        let report = commit_prepared(&mut model, &self.path, prepared)?;
        Ok((render_output(format, &report), 0))
    }
}
