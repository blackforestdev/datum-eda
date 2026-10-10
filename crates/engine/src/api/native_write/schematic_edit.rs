//! Native writer adapters over existing schematic builders and library binding.
use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};
use super::schematic_library::{
    materialize_pool_symbol_pins, resolve_pool_symbol_component_binding,
};
use super::schematic_symbols::{PlacedSymbolPartBinding, build_place_schematic_symbol};
use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::schematic::{HiddenPowerBehavior, PlacedSymbol, Sheet, SymbolDisplayMode};
use crate::substrate::{DesignModel, ElectricalSelectionSnapshot, Operation, SchematicMarkerKind};
use serde::Deserialize;
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.move_symbol",
        build: super::schematic_symbol_move::move_symbol,
    },
    NativeWriteVerb {
        id: "datum.schematic.place_symbol",
        build: place_symbol,
    },
    NativeWriteVerb {
        id: "datum.schematic.draw_wire",
        build: draw_wire,
    },
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceParams {
    sheet: Uuid,
    library_symbol: Uuid,
    reference: String,
    value: String,
    x_nm: i64,
    y_nm: i64,
    #[serde(default)]
    rotation_deg: i32,
    #[serde(default)]
    mirrored: bool,
}

fn schematic(model: &DesignModel) -> Result<crate::schematic::Schematic, EngineError> {
    ElectricalSelectionSnapshot::capture(model)
        .map_err(|error| {
            EngineError::Validation(format!("Schematic source unavailable: {error:?}"))
        })?
        .schematic_source()
        .cloned()
        .ok_or_else(|| EngineError::Validation("Project has no native schematic".into()))
}

fn place_symbol(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: PlaceParams = parse_verb_params("datum.schematic.place_symbol", params)?;
    let source = schematic(context.model)?;
    if !source.sheets.contains_key(&params.sheet) {
        return Err(EngineError::Validation("Native sheet not found".into()));
    }
    if params.reference.trim().is_empty()
        || source.sheets.values().any(|sheet| {
            sheet
                .symbols
                .values()
                .any(|symbol| symbol.reference == params.reference)
        })
    {
        return Err(EngineError::Validation(
            "Symbol reference must be nonempty and unique".into(),
        ));
    }
    let id = params.library_symbol.to_string();
    let resolved = resolve_pool_symbol_component_binding(context.model, Some(&id))?;
    let binding = resolved
        .binding
        .as_ref()
        .and_then(|binding| binding.part.as_ref().map(|part| (binding, part)))
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "Native symbol cannot be placed: {}. {}",
                resolved.status,
                resolved.diagnostics.join("; ")
            ))
        })?;
    let mut pins = materialize_pool_symbol_pins(context.model, Some(&id))?;
    // Placed schematic pins are absolute electrical/scene attachment points;
    // the reusable pool anchors are local. Existing cardinal poses are exact.
    for pin in &mut pins {
        pin.position = place_local_point(
            pin.position,
            Point {
                x: params.x_nm,
                y: params.y_nm,
            },
            params.rotation_deg,
            params.mirrored,
        )?;
    }
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: Some(binding.1.part_id),
        entity: Some(binding.0.entity_id),
        gate: Some(binding.0.gate_id),
        lib_id: Some(id.clone()),
        reference: params.reference,
        value: params.value,
        fields: Vec::new(),
        pins,
        position: Point {
            x: params.x_nm,
            y: params.y_nm,
        },
        rotation: params.rotation_deg,
        mirrored: params.mirrored,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::new(),
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let prepared = build_place_schematic_symbol(
        context.model,
        provenance,
        params.sheet,
        &symbol,
        Some(&PlacedSymbolPartBinding {
            pool_symbol_id: params.library_symbol,
            part_id: binding.1.part_id,
        }),
    )?;
    prepare_native_birth(context.model, prepared)
}

fn prepare_native_birth(
    model: &DesignModel,
    mut prepared: PreparedWrite,
) -> Result<PreparedWrite, EngineError> {
    prepared.batch = crate::substrate::bootstrap_empty_native_topology(model, prepared.batch)?;
    Ok(prepared)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireParams {
    sheet: Uuid,
    points: Vec<Point>,
}

fn draw_wire(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: WireParams = parse_verb_params("datum.schematic.draw_wire", params)?;
    let source = schematic(context.model)?;
    let sheet = source
        .sheets
        .get(&params.sheet)
        .ok_or_else(|| EngineError::Validation("Native sheet not found".into()))?;
    build_wire_gesture(context.model, provenance, sheet, &params.points)
}

/// Complete wire gesture with exact T contacts in one guarded history entry.
/// Existing crossings without an endpoint are not silently turned into junctions.
pub fn build_wire_gesture(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet: &Sheet,
    points: &[Point],
) -> Result<PreparedWrite, EngineError> {
    if points.len() < 2 || points.windows(2).any(|p| p[0] == p[1]) {
        return Err(EngineError::Validation(
            "Wire needs distinct consecutive vertices".into(),
        ));
    }
    let mut junctions = BTreeSet::new();
    let mut operations = Vec::new();
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        for wire in sheet.wires.values() {
            for endpoint in [a, b] {
                if strictly_on_segment(endpoint, wire.from, wire.to)? {
                    junctions.insert((endpoint.x, endpoint.y));
                }
            }
            for endpoint in [wire.from, wire.to] {
                if strictly_on_segment(endpoint, a, b)? {
                    junctions.insert((endpoint.x, endpoint.y));
                }
            }
        }
        for other in points.windows(2) {
            for endpoint in [other[0], other[1]] {
                if strictly_on_segment(endpoint, a, b)? {
                    junctions.insert((endpoint.x, endpoint.y));
                }
            }
        }
        let wire = crate::schematic::SchematicWire {
            uuid: Uuid::new_v4(),
            from: a,
            to: b,
        };
        operations.push(Operation::CreateSchematicWire {
            sheet_id: sheet.uuid,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        });
    }
    for (x, y) in junctions {
        let position = Point { x, y };
        if sheet.junctions.values().any(|j| j.position == position) {
            continue;
        }
        let marker_id = Uuid::new_v4();
        operations.push(Operation::PlaceSchematicMarker {
            sheet_id: sheet.uuid,
            marker_id,
            marker_kind: SchematicMarkerKind::Junction,
            marker: serde_json::to_value(crate::schematic::Junction {
                uuid: marker_id,
                position,
            })?,
        });
    }
    let prepared = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()?;
    prepare_native_birth(model, prepared)
}

fn strictly_on_segment(p: Point, a: Point, b: Point) -> Result<bool, EngineError> {
    if p == a
        || p == b
        || p.x < a.x.min(b.x)
        || p.x > a.x.max(b.x)
        || p.y < a.y.min(b.y)
        || p.y > a.y.max(b.y)
    {
        return Ok(false);
    }
    let dx = i128::from(b.x) - i128::from(a.x);
    let dy = i128::from(b.y) - i128::from(a.y);
    let px = i128::from(p.x) - i128::from(a.x);
    let py = i128::from(p.y) - i128::from(a.y);
    // Each difference is at most 2^64-1. Unsigned products fit u128;
    // compare signed products without a potentially overflowing subtraction.
    let lhs = dx.unsigned_abs() * py.unsigned_abs();
    let rhs = dy.unsigned_abs() * px.unsigned_abs();
    Ok(lhs == rhs && (lhs == 0 || dx.signum() * py.signum() == dy.signum() * px.signum()))
}

#[cfg(test)]
#[path = "schematic_edit_tests.rs"]
mod tests;

/// Exact local-to-placed transform for native schematic cardinal poses.
/// Irrational coordinates cannot be stored in the existing integer pin field.
pub fn place_local_point(
    point: Point,
    origin: Point,
    rotation: i32,
    mirrored: bool,
) -> Result<Point, EngineError> {
    let x = if mirrored {
        point.x.checked_neg()
    } else {
        Some(point.x)
    }
    .ok_or_else(|| {
        EngineError::Validation("Symbol reflection exceeds integer coordinates".into())
    })?;
    let y = point.y;
    let (x, y) = match rotation.rem_euclid(360) {
        0 => (Some(x), Some(y)),
        90 => (y.checked_neg(), Some(x)),
        180 => (x.checked_neg(), y.checked_neg()),
        270 => (Some(y), x.checked_neg()),
        _ => {
            return Err(EngineError::Validation(
                "This integer native symbol placement requires an exact cardinal pose".into(),
            ));
        }
    };
    Ok(Point {
        x: x.and_then(|x| origin.x.checked_add(x)).ok_or_else(|| {
            EngineError::Validation("Symbol placement exceeds integer coordinates".into())
        })?,
        y: y.and_then(|y| origin.y.checked_add(y)).ok_or_else(|| {
            EngineError::Validation("Symbol placement exceeds integer coordinates".into())
        })?,
    })
}
