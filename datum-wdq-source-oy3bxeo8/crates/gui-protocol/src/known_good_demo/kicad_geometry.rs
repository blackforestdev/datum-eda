//! KiCad reference-geometry patcher for the known-good demo project.
//!
//! Extracted from `lib.rs` (behavior-preserving move, decision 022 source-health
//! burn-down) to give the gui-protocol root headroom for the S4 crosshair state.
//! This owns exactly one responsibility: importing the four demo footprints'
//! silkscreen/mechanical geometry from the system KiCad footprint library and
//! patching them into the freshly written known-good `board.json`. The demo
//! project skeleton itself is written by the parent `known_good_demo` module.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::Path;

use crate::PointNm;

pub(super) fn apply_kicad_reference_geometry(board_path: &Path) -> Result<()> {
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(board_path)
            .with_context(|| format!("failed to read {}", board_path.display()))?,
    )
    .context("failed to decode known-good board JSON for KiCad geometry patching")?;

    let specs = [
        (
            "00000000-0000-0000-0000-00000000c203",
            "U1",
            Path::new(
                "/usr/share/kicad/footprints/Package_SO.pretty/SOIC-8_3.9x4.9mm_P1.27mm.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c204",
            "J2",
            Path::new(
                "/usr/share/kicad/footprints/Connector_PinHeader_2.54mm.pretty/PinHeader_1x03_P2.54mm_Vertical.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c208",
            "R1",
            Path::new(
                "/usr/share/kicad/footprints/Resistor_SMD.pretty/R_0805_2012Metric.kicad_mod",
            ),
        ),
        (
            "00000000-0000-0000-0000-00000000c209",
            "TP1",
            Path::new(
                "/usr/share/kicad/footprints/TestPoint.pretty/TestPoint_Loop_D2.60mm_Drill1.4mm_Beaded.kicad_mod",
            ),
        ),
    ];

    for (component_uuid, reference, path) in specs {
        let geometry = load_kicad_demo_geometry(path, reference)?;
        replace_component_geometry(&mut board, component_uuid, &geometry)?;
    }

    super::write_json_file(board_path, &board)?;
    Ok(())
}

#[derive(Default)]
struct KicadDemoGeometry {
    silk_lines: Vec<Value>,
    silk_polylines: Vec<Value>,
    silk_circles: Vec<Value>,
    silk_polygons: Vec<Value>,
    silk_arcs: Vec<Value>,
    silk_texts: Vec<Value>,
    mechanical_lines: Vec<Value>,
    mechanical_polylines: Vec<Value>,
    mechanical_circles: Vec<Value>,
    mechanical_polygons: Vec<Value>,
    mechanical_arcs: Vec<Value>,
    mechanical_texts: Vec<Value>,
}

fn load_kicad_demo_geometry(path: &Path, reference: &str) -> Result<KicadDemoGeometry> {
    let (imported, _report) = eda_engine::import::kicad::import_footprint_document(path)
        .with_context(|| format!("failed to import KiCad footprint {}", path.display()))?;
    let mut out = KicadDemoGeometry::default();
    append_primitive_geometry(&mut out, &imported.package.silkscreen, true, reference);
    append_primitive_geometry(&mut out, &imported.mechanical, false, reference);
    if !imported.package.courtyard.vertices.is_empty() {
        out.mechanical_polygons.push(json!({
            "vertices": imported.package.courtyard.vertices.iter().map(|point| point_to_json(PointNm { x: point.x, y: point.y })).collect::<Vec<_>>(),
            "layer": 41
        }));
    }
    Ok(out)
}

fn replace_component_geometry(
    board: &mut Value,
    component_uuid: &str,
    geometry: &KicadDemoGeometry,
) -> Result<()> {
    replace_component_section(
        board,
        "component_silkscreen",
        component_uuid,
        &geometry.silk_lines,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_polylines",
        component_uuid,
        &geometry.silk_polylines,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_circles",
        component_uuid,
        &geometry.silk_circles,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_polygons",
        component_uuid,
        &geometry.silk_polygons,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_arcs",
        component_uuid,
        &geometry.silk_arcs,
    )?;
    replace_component_section(
        board,
        "component_silkscreen_texts",
        component_uuid,
        &geometry.silk_texts,
    )?;
    replace_component_section(
        board,
        "component_mechanical_lines",
        component_uuid,
        &geometry.mechanical_lines,
    )?;
    replace_component_section(
        board,
        "component_mechanical_polylines",
        component_uuid,
        &geometry.mechanical_polylines,
    )?;
    replace_component_section(
        board,
        "component_mechanical_circles",
        component_uuid,
        &geometry.mechanical_circles,
    )?;
    replace_component_section(
        board,
        "component_mechanical_polygons",
        component_uuid,
        &geometry.mechanical_polygons,
    )?;
    replace_component_section(
        board,
        "component_mechanical_arcs",
        component_uuid,
        &geometry.mechanical_arcs,
    )?;
    replace_component_section(
        board,
        "component_mechanical_texts",
        component_uuid,
        &geometry.mechanical_texts,
    )?;
    Ok(())
}

fn replace_component_section(
    board: &mut Value,
    key: &str,
    component_uuid: &str,
    values: &[Value],
) -> Result<()> {
    let section = board
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("known-good board missing object section {key}"))?;
    section.insert(component_uuid.to_string(), Value::Array(values.to_vec()));
    Ok(())
}

fn append_primitive_geometry(
    out: &mut KicadDemoGeometry,
    primitives: &[eda_engine::pool::Primitive],
    silkscreen: bool,
    reference: &str,
) {
    for primitive in primitives {
        match primitive {
            eda_engine::pool::Primitive::Line { from, to, width } => {
                target_lines(out, silkscreen).push(json!({
                    "from": point_to_json(PointNm { x: from.x, y: from.y }),
                    "to": point_to_json(PointNm { x: to.x, y: to.y }),
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Rect { min, max, width } => {
                target_polygons(out, silkscreen).push(json!({
                    "vertices": vec![
                        point_to_json(PointNm { x: min.x, y: min.y }),
                        point_to_json(PointNm { x: max.x, y: min.y }),
                        point_to_json(PointNm { x: max.x, y: max.y }),
                        point_to_json(PointNm { x: min.x, y: max.y }),
                    ],
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Circle {
                center,
                radius,
                width,
            } => {
                target_circles(out, silkscreen).push(json!({
                    "center": point_to_json(PointNm { x: center.x, y: center.y }),
                    "radius_nm": *radius,
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Polygon { polygon, width } => {
                target_polygons(out, silkscreen).push(json!({
                    "vertices": polygon.vertices.iter().map(|point| point_to_json(PointNm { x: point.x, y: point.y })).collect::<Vec<_>>(),
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Arc { arc, width } => {
                target_arcs(out, silkscreen).push(json!({
                    "center": point_to_json(PointNm { x: arc.center.x, y: arc.center.y }),
                    "radius_nm": arc.radius,
                    "start_angle": arc.start_angle,
                    "end_angle": arc.end_angle,
                    "width_nm": *width,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
            eda_engine::pool::Primitive::Text {
                text,
                position,
                rotation,
            } => {
                let normalized = normalize_reference_text(text, reference);
                if normalized != reference {
                    continue;
                }
                target_texts(out, silkscreen).push(json!({
                    "text": normalized,
                    "position": point_to_json(PointNm { x: position.x, y: position.y }),
                    "rotation": *rotation,
                    "height_nm": 1_000_000,
                    "stroke_width_nm": 150_000,
                    "layer": if silkscreen { 1 } else { 41 }
                }));
            }
        }
    }
}

fn normalize_reference_text(text: &str, reference: &str) -> String {
    if text.contains("REF")
        || text.contains("Reference")
        || text.contains('?')
        || text.contains("${REFERENCE}")
    {
        reference.to_string()
    } else {
        text.to_string()
    }
}

fn target_lines(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_lines
    } else {
        &mut out.mechanical_lines
    }
}

fn target_circles(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_circles
    } else {
        &mut out.mechanical_circles
    }
}

fn target_polygons(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_polygons
    } else {
        &mut out.mechanical_polygons
    }
}

fn target_arcs(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_arcs
    } else {
        &mut out.mechanical_arcs
    }
}

fn target_texts(out: &mut KicadDemoGeometry, silkscreen: bool) -> &mut Vec<Value> {
    if silkscreen {
        &mut out.silk_texts
    } else {
        &mut out.mechanical_texts
    }
}

fn point_to_json(point: PointNm) -> Value {
    json!({ "x": point.x, "y": point.y })
}
