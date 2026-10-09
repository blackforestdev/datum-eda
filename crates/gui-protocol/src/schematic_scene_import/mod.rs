mod projection;
use anyhow::{Context, Result};
use eda_engine::schematic::{Schematic, Sheet};
use eda_engine::text::{TextHAlign, TextVAlign};
pub(crate) use projection::project_schematic_sheet_source;
use std::path::{Path, PathBuf};

use super::*;

mod buses;
mod common;
mod drawings;
mod labels;
mod layers;
mod power;
mod symbols;

use buses::*;
use common::*;
use drawings::*;
use labels::*;
use layers::*;
use power::*;
use symbols::*;

/// Typed interaction role attached by schematic projection. It describes the
/// authored primitive independently from the active tool's selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchematicHitKind {
    Symbol,
    Pin,
    Wire,
    Bus,
    Label,
    Port,
    Junction,
    NoConnect,
    Drawing,
}

impl BoardGraphicPrimitive {
    pub fn schematic_hit_kind(&self) -> Option<SchematicHitKind> {
        match self.object_kind.as_str() {
            "schematic_symbol" => Some(SchematicHitKind::Symbol),
            "schematic_pin" => Some(SchematicHitKind::Pin),
            "schematic_wire" => Some(SchematicHitKind::Wire),
            "schematic_bus" => Some(SchematicHitKind::Bus),
            "schematic_label" => Some(SchematicHitKind::Label),
            "schematic_port" => Some(SchematicHitKind::Port),
            "schematic_junction" => Some(SchematicHitKind::Junction),
            "schematic_no_connect" => Some(SchematicHitKind::NoConnect),
            "schematic_drawing" => Some(SchematicHitKind::Drawing),
            _ => None,
        }
    }
}

pub fn load_kicad_schematic_workspace_state(schematic_file: &Path) -> Result<ReviewWorkspaceState> {
    let source = schematic_file.canonicalize().with_context(|| {
        format!(
            "failed to resolve KiCad schematic {}",
            schematic_file.display()
        )
    })?;
    let (scene, schematic_path) = load_scene_from_kicad_schematic_import(&source)?;
    let request = LiveReviewRequest {
        project_root: source
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state = ReviewWorkspaceState::new(scene, empty_route_review_payload(&request));
    state.backing = Some(WorkspaceBacking {
        request,
        board_path: schematic_path,
    });
    Ok(state)
}

pub(crate) fn load_scene_from_kicad_schematic_import(
    schematic_file: &Path,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let import_started = std::time::Instant::now();
    let (schematic, report) = eda_engine::import::kicad::import_schematic_document(schematic_file)
        .map_err(|e| anyhow::anyhow!("import {}: {e}", schematic_file.display()))?;
    for warning in &report.warnings {
        eprintln!(
            "datum-import warning [{}]: {warning}",
            schematic_file.display()
        );
    }
    trace_protocol_timing(format!(
        "kicad schematic import {}ms warnings={}",
        import_started.elapsed().as_millis(),
        report.warnings.len()
    ));

    Ok((
        project_schematic_source(&schematic, schematic_file)?,
        schematic_file.to_path_buf(),
    ))
}

pub(crate) fn project_schematic_source(
    schematic: &Schematic,
    schematic_file: &Path,
) -> Result<BoardReviewSceneV1> {
    let root_sheet = root_sheet(schematic).context("imported schematic has no root sheet")?;
    project_schematic_sheet_source(schematic, root_sheet, schematic_file)
}

/// Attach explicit interaction metadata while the typed schematic model is still
/// available. Consumers never need to reverse-engineer identities or layer names.
fn tag_schematic_hit_kinds(sheet: &Sheet, graphics: &mut [BoardGraphicPrimitive]) {
    let mut kinds = std::collections::BTreeMap::new();
    for wire in sheet.wires.values() {
        kinds.insert(wire.uuid.to_string(), "schematic_wire");
    }
    for bus in sheet.buses.values() {
        kinds.insert(bus.uuid.to_string(), "schematic_bus");
    }
    for drawing in sheet.drawings.values() {
        kinds.insert(
            match drawing {
                eda_engine::schematic::SchematicPrimitive::Line { uuid, .. }
                | eda_engine::schematic::SchematicPrimitive::Rect { uuid, .. }
                | eda_engine::schematic::SchematicPrimitive::Circle { uuid, .. }
                | eda_engine::schematic::SchematicPrimitive::Arc { uuid, .. } => uuid.to_string(),
            },
            "schematic_drawing",
        );
    }
    for symbol in sheet.symbols.values() {
        kinds.insert(symbol.uuid.to_string(), "schematic_symbol");
        for pin in &symbol.pins {
            kinds.insert(pin.uuid.to_string(), "schematic_pin");
        }
    }
    for label in sheet.labels.values() {
        kinds.insert(label.uuid.to_string(), "schematic_label");
    }
    for port in sheet.ports.values() {
        kinds.insert(port.uuid.to_string(), "schematic_port");
    }
    for junction in sheet.junctions.values() {
        kinds.insert(junction.uuid.to_string(), "schematic_junction");
    }
    for marker in sheet.noconnects.values() {
        kinds.insert(marker.uuid.to_string(), "schematic_no_connect");
    }
    for graphic in graphics {
        if let Some(kind) = kinds.get(&graphic.source_object_uuid) {
            graphic.object_kind = (*kind).to_string();
        }
    }
}

fn root_sheet(schematic: &Schematic) -> Option<&Sheet> {
    schematic
        .sheet_definitions
        .values()
        .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        .and_then(|definition| schematic.sheets.get(&definition.root_sheet))
        .or_else(|| {
            schematic
                .sheets
                .values()
                .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        })
}

fn push_root_sheet_graphics(
    sheet: &Sheet,
    schematic: &Schematic,
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
) {
    for wire in sorted_values(&sheet.wires) {
        push_line_graphic(
            graphics,
            points,
            format!("schematic-wire:{}", wire.uuid),
            wire.uuid,
            wire.from,
            wire.to,
            SCHEMATIC_STROKE_NM,
            SCHEMATIC_WIRE_LAYER,
        );
    }
    // P2.2e: buses (gold thick polylines) and their diagonal green entry stubs.
    for bus in sorted_values(&sheet.buses) {
        push_bus_graphic(graphics, points, bus);
    }
    for entry in sorted_values(&sheet.bus_entries) {
        push_bus_entry_graphic(graphics, points, entry);
    }
    for drawing in sorted_values(&sheet.drawings) {
        push_drawing_graphic(graphics, points, text, drawing);
    }
    for symbol in sorted_values(&sheet.symbols) {
        // P2.2e: power symbols (KiCad `power:*` lib_ids) project a rail flag or a
        // ground stack instead of a generic IEC box.
        if is_power_symbol(symbol) {
            push_power_symbol_graphics(graphics, points, text, symbol);
        } else {
            push_symbol_graphics(graphics, points, text, symbol);
        }
    }
    for label in sorted_values(&sheet.labels) {
        push_net_label_graphic(graphics, points, text, label);
    }
    for port in sorted_values(&sheet.ports) {
        push_hierarchical_port_graphic(graphics, points, text, port);
    }
    for junction in sorted_values(&sheet.junctions) {
        push_circle_graphic(
            graphics,
            points,
            format!("schematic-junction:{}", junction.uuid),
            junction.uuid,
            junction.position,
            JUNCTION_RADIUS_NM,
            SCHEMATIC_JUNCTION_LAYER,
        );
    }
    for noconnect in sorted_values(&sheet.noconnects) {
        push_cross_graphic(
            graphics,
            points,
            format!("schematic-noconnect:{}", noconnect.uuid),
            noconnect.uuid,
            noconnect.position,
            NOCONNECT_HALF_NM,
            SCHEMATIC_NOCONNECT_LAYER,
        );
    }
    for schematic_text in sorted_values(&sheet.texts) {
        // Free schematic text is annotation, not a boxed object: emit it as real
        // text at its own position rather than a bare labelled rect.
        text.push_rotated(
            points,
            schematic_text.uuid,
            "schematic-text",
            &schematic_text.text,
            point_nm(schematic_text.position),
            LABEL_TEXT_HEIGHT_NM,
            TextHAlign::Left,
            TextVAlign::Center,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
            schematic_text.rotation,
        );
    }
    for instance in sorted_values(&schematic.sheet_instances) {
        if instance.parent_sheet != Some(sheet.uuid) {
            continue;
        }
        push_rect_graphic(
            graphics,
            points,
            text,
            format!("schematic-sheet-instance:{}", instance.uuid),
            instance.uuid,
            instance.position,
            SHEET_INSTANCE_HALF_WIDTH_NM,
            SHEET_INSTANCE_HALF_HEIGHT_NM,
            Some(instance.name.clone()),
            SCHEMATIC_ANNOTATION_LAYER,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        );
    }
}

#[cfg(test)]
mod tests;

pub(crate) mod native_library;
