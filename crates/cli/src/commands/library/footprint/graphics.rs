//! Manual Footprint graphic lifecycle and legacy proposal input adapters.
use super::super::library::{
    NativeProjectPoolLibraryObjectMutationView, definition_validation_error,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::super::payload::read_project_pool_object_payload;
use super::ensure_pool_object_exists;
use crate::*;
use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};
use eda_engine::api::native_write::library_footprint::{
    FootprintGraphicInput, FootprintGraphicTarget, footprint_graphic_primitive,
    footprint_with_graphic,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::ProjectResolver;
use std::path::Path;
use uuid::Uuid;

fn parse_target(value: &str) -> Result<FootprintGraphicTarget> {
    match value {
        "silkscreen" => Ok(FootprintGraphicTarget::Silkscreen),
        "fab" => Ok(FootprintGraphicTarget::Fab),
        _ => bail!("unsupported footprint graphic target {value}; expected silkscreen or fab"),
    }
}
fn commit_graphic(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    target: FootprintGraphicTarget,
    input: FootprintGraphicInput<'_>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    // Preserve geometry-before-path/reference diagnostic order.
    let primitive =
        footprint_graphic_primitive(target, input).map_err(definition_validation_error)?;
    validate_project_local_pool_path(pool_path)?;
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    eda_engine::api::native_write::library_definition::validate_pool_definition_reference(
        &model,
        footprint_id,
        "footprints",
        "footprint",
    )
    .map_err(definition_validation_error)?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let object = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .with_context(|| format!("failed to materialize pool library object {relative_path}"))?;
    // Match the historical CLI identity diagnostics on this same model basis.
    let raw = object
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("pool library object missing uuid"))?;
    let payload_id =
        Uuid::parse_str(raw).with_context(|| format!("invalid pool library object uuid {raw}"))?;
    if payload_id != footprint_id {
        bail!("pool library object uuid {payload_id} does not match --object {footprint_id}");
    }
    let object =
        footprint_with_graphic(object, target, primitive).map_err(definition_validation_error)?;
    let (shape, action) = match (target, input) {
        (FootprintGraphicTarget::Silkscreen, FootprintGraphicInput::Line { .. }) => {
            ("line", "add_footprint_silkscreen_line")
        }
        (FootprintGraphicTarget::Silkscreen, FootprintGraphicInput::Rect { .. }) => {
            ("rect", "add_footprint_silkscreen_rect")
        }
        (FootprintGraphicTarget::Silkscreen, FootprintGraphicInput::Circle { .. }) => {
            ("circle", "add_footprint_silkscreen_circle")
        }
        (FootprintGraphicTarget::Silkscreen, FootprintGraphicInput::Polygon { .. }) => {
            ("polygon", "add_footprint_silkscreen_polygon")
        }
        (FootprintGraphicTarget::Fab, FootprintGraphicInput::Line { .. }) => {
            ("line", "add_footprint_fab_line")
        }
        (FootprintGraphicTarget::Fab, FootprintGraphicInput::Rect { .. }) => {
            ("rect", "add_footprint_fab_rect")
        }
        (FootprintGraphicTarget::Fab, FootprintGraphicInput::Circle { .. }) => {
            ("circle", "add_footprint_fab_circle")
        }
        (FootprintGraphicTarget::Fab, FootprintGraphicInput::Polygon { .. }) => {
            ("polygon", "add_footprint_fab_polygon")
        }
    };
    let field = match target {
        FootprintGraphicTarget::Silkscreen => "silkscreen",
        FootprintGraphicTarget::Fab => "fab",
    };
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            crate::cli_commit_source()?,
            format!("add native pool footprint {footprint_id} {field} {shape}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                relative_path.clone(),
            ),
            object,
        }],
    )?;
    commit_prepared(&mut model, root, prepared)?;
    pool_library_mutation_view(
        root,
        action,
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

// Existing proposal callers retain their historical signature and silkscreen profile.
pub(crate) fn footprint_object_with_appended_silkscreen(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    primitive: serde_json::Value,
) -> Result<(String, serde_json::Value)> {
    validate_project_local_pool_path(pool_path)?;
    ensure_pool_object_exists(root, footprint_id, "footprints", "footprint")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let object = footprint_with_graphic(object, FootprintGraphicTarget::Silkscreen, primitive)
        .map_err(definition_validation_error)?;
    Ok((relative_path, object))
}

pub(crate) fn footprint_silkscreen_line_primitive(
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    footprint_graphic_primitive(
        FootprintGraphicTarget::Silkscreen,
        FootprintGraphicInput::Line {
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            width_nm,
        },
    )
    .map_err(definition_validation_error)
}

pub(crate) fn footprint_silkscreen_rect_primitive(
    min_x_nm: i64,
    min_y_nm: i64,
    max_x_nm: i64,
    max_y_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    footprint_graphic_primitive(
        FootprintGraphicTarget::Silkscreen,
        FootprintGraphicInput::Rect {
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
            width_nm,
        },
    )
    .map_err(definition_validation_error)
}

pub(crate) fn footprint_silkscreen_circle_primitive(
    center_x_nm: i64,
    center_y_nm: i64,
    radius_nm: i64,
    width_nm: i64,
) -> Result<serde_json::Value> {
    footprint_graphic_primitive(
        FootprintGraphicTarget::Silkscreen,
        FootprintGraphicInput::Circle {
            center_x_nm,
            center_y_nm,
            radius_nm,
            width_nm,
        },
    )
    .map_err(definition_validation_error)
}

pub(crate) fn footprint_silkscreen_polygon_primitive(
    vertices: &str,
    closed: bool,
    width_nm: i64,
) -> Result<serde_json::Value> {
    footprint_graphic_primitive(
        FootprintGraphicTarget::Silkscreen,
        FootprintGraphicInput::Polygon {
            vertices,
            closed,
            width_nm,
        },
    )
    .map_err(definition_validation_error)
}

impl ProjectAddPoolFootprintSilkscreenLineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            target,
            footprint_uuid,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            width_nm,
        } = self;
        Ok((
            render_output(
                format,
                &commit_graphic(
                    &path,
                    &pool,
                    footprint_uuid,
                    parse_target(&target)?,
                    FootprintGraphicInput::Line {
                        from_x_nm,
                        from_y_nm,
                        to_x_nm,
                        to_y_nm,
                        width_nm,
                    },
                )?,
            ),
            0,
        ))
    }
}

impl ProjectAddPoolFootprintSilkscreenRectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            target,
            footprint_uuid,
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
            width_nm,
        } = self;
        Ok((
            render_output(
                format,
                &commit_graphic(
                    &path,
                    &pool,
                    footprint_uuid,
                    parse_target(&target)?,
                    FootprintGraphicInput::Rect {
                        min_x_nm,
                        min_y_nm,
                        max_x_nm,
                        max_y_nm,
                        width_nm,
                    },
                )?,
            ),
            0,
        ))
    }
}

impl ProjectAddPoolFootprintSilkscreenCircleArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            target,
            footprint_uuid,
            center_x_nm,
            center_y_nm,
            radius_nm,
            width_nm,
        } = self;
        Ok((
            render_output(
                format,
                &commit_graphic(
                    &path,
                    &pool,
                    footprint_uuid,
                    parse_target(&target)?,
                    FootprintGraphicInput::Circle {
                        center_x_nm,
                        center_y_nm,
                        radius_nm,
                        width_nm,
                    },
                )?,
            ),
            0,
        ))
    }
}

impl ProjectAddPoolFootprintSilkscreenPolygonArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            target,
            footprint_uuid,
            vertices,
            closed,
            width_nm,
        } = self;
        Ok((
            render_output(
                format,
                &commit_graphic(
                    &path,
                    &pool,
                    footprint_uuid,
                    parse_target(&target)?,
                    FootprintGraphicInput::Polygon {
                        vertices: &vertices,
                        closed,
                        width_nm,
                    },
                )?,
            ),
            0,
        ))
    }
}
