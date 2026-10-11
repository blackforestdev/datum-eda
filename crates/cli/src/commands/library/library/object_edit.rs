//! Generic object replacement owns payload admission and the canonical edit lifecycle.
use std::path::Path;

use crate::{OutputFormat, ProjectSetPoolLibraryObjectArgs, render_output};
use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};
use eda_engine::api::native_write::library_definition::{
    footprint_with_pads, padstack_with_aperture, symbol_with_drawings, symbol_with_fields,
    validate_footprint_replacement_references, validate_part_replacement_references,
    validate_symbol_replacement_references,
};
use eda_engine::api::native_write::library_footprint::{
    FootprintGraphicTarget, footprint_with_graphics,
};
use eda_engine::api::native_write::library_pin_pad_map::validate_pin_pad_map_replacement_references;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use uuid::Uuid;

use super::super::payload::{read_pool_library_object_payload, read_project_pool_object_payload};
use super::{
    NativeProjectPoolLibraryObjectMutationView, definition_validation_error,
    pool_library_mutation_view, pool_library_relative_path, validate_pool_library_object_kind,
    validate_project_local_pool_path,
};

pub(crate) fn set_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source: &Path,
    section: Option<&str>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    let (mut model, object) = if let Some(section) = section {
        let required_kind = match section {
            "fields" | "drawings" => "symbols",
            "silkscreen" | "fab" | "pads" => "footprints",
            "aperture" => "padstacks",
            _ => bail!(
                "unsupported pool library section {section}; expected fields, drawings, silkscreen, fab, aperture or pads"
            ),
        };
        if object_kind != required_kind {
            bail!("--section {section} requires --kind {required_kind}");
        }
        let model = ProjectResolver::new(root)
            .resolve()
            .with_context(|| format!("failed to resolve native project {}", root.display()))?;
        let object = eda_engine::api::native_write::library::pool_library_object_payload(
            &model,
            &relative_path,
            object_id,
        )
        .map_err(definition_validation_error)?;
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(source).with_context(|| {
                format!("failed to read pool library {section} {}", source.display())
            })?)
            .with_context(|| {
                format!(
                    "failed to parse pool library {section} {}",
                    source.display()
                )
            })?;
        let object = match section {
            "fields" => symbol_with_fields(object, value),
            "drawings" => symbol_with_drawings(object, value),
            "silkscreen" => {
                footprint_with_graphics(object, FootprintGraphicTarget::Silkscreen, value)
            }
            "fab" => footprint_with_graphics(object, FootprintGraphicTarget::Fab, value),
            "aperture" => padstack_with_aperture(object, value),
            _ => footprint_with_pads(object, value),
        }
        .map_err(definition_validation_error)?;
        (model, object)
    } else {
        // Preserve the whole-object mode's existing input and diagnostic order.
        read_project_pool_object_payload(root, &relative_path, object_id)?;
        let object = read_pool_library_object_payload(source, object_id)?;
        let model = ProjectResolver::new(root)
            .resolve()
            .with_context(|| format!("failed to resolve native project {}", root.display()))?;
        (model, object)
    };
    let source = crate::cli_commit_source()?;
    if source == CommitSource::Cli {
        match object_kind {
            "footprints" => validate_footprint_replacement_references(&model, &object)
                .map_err(definition_validation_error)?,
            "parts" => validate_part_replacement_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            "pin_pad_maps" => {
                validate_pin_pad_map_replacement_references(&model, &relative_path, &object)
                    .map_err(definition_validation_error)?
            }
            "symbols" => validate_symbol_replacement_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            _ => {}
        }
    }
    // Validate and prepare against the same resolved revision; the canonical
    // commit guard rejects an intervening edit rather than accepting stale links.
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            source,
            format!("set native pool library object {object_id}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object,
        }],
    )?;
    commit_prepared(&mut model, root, prepared)?;
    pool_library_mutation_view(
        root,
        "set",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}

impl ProjectSetPoolLibraryObjectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            kind,
            object_uuid,
            from_json,
            section,
        } = self;
        Ok((
            render_output(
                format,
                &set_native_project_pool_library_object(
                    &path,
                    &pool,
                    &kind,
                    object_uuid,
                    &from_json,
                    section.as_deref(),
                )?,
            ),
            0,
        ))
    }
}
