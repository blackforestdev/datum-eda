//! Native interface-port create/edit/delete through canonical schematic writers.
use super::{
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_connectivity::{
        build_create_schematic_port, build_delete_schematic_port, build_set_schematic_port,
    },
    schematic_edit::prepare_native_birth,
    schematic_selection_edit::selected_source,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{HierarchicalPort, PortDirection},
    substrate::ElectricalSelectionSnapshot,
};
use serde::Deserialize;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.place_port",
        build: place_port,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_port",
        build: delete_port,
    },
];

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Input,
    Output,
    Bidirectional,
    Passive,
}
impl From<Direction> for PortDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Input => Self::Input,
            Direction::Output => Self::Output,
            Direction::Bidirectional => Self::Bidirectional,
            Direction::Passive => Self::Passive,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceParams {
    sheet: Uuid,
    port: Option<Uuid>,
    instance_path: Option<Vec<Uuid>>,
    name: String,
    direction: Direction,
    x_nm: i64,
    y_nm: i64,
}

fn place_port(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: PlaceParams = parse_verb_params("datum.schematic.place_port", params)?;
    if params.name.trim().is_empty() || params.name.chars().any(char::is_control) {
        return Err(invalid("Port name must be nonempty and single-line"));
    }
    let port = HierarchicalPort {
        uuid: params.port.unwrap_or_else(Uuid::new_v4),
        name: params.name,
        direction: params.direction.into(),
        position: Point::new(params.x_nm, params.y_nm),
    };
    if params.port.is_some() {
        let path = params
            .instance_path
            .as_deref()
            .ok_or_else(|| invalid("Port edit requires its occurrence path"))?;
        let (source, sheet) = selected_source(context, "ports", port.uuid, path)?;
        if sheet != params.sheet {
            return Err(invalid("Port belongs to a different source sheet"));
        }
        if source.sheets[&sheet].ports[&port.uuid] == port {
            return Err(invalid("Port fields are unchanged"));
        }
        return build_set_schematic_port(context.model, provenance, sheet, &port);
    }
    if params
        .instance_path
        .as_ref()
        .is_some_and(|path| !path.is_empty())
    {
        return Err(invalid(
            "Port creation requires the supported source-sheet view",
        ));
    }
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Schematic source unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic source unavailable"))?;
    if !source
        .sheets
        .get(&params.sheet)
        .is_some_and(|sheet| sheet.uuid == params.sheet)
    {
        return Err(invalid("Native sheet not found"));
    }
    let write = build_create_schematic_port(context.model, provenance, params.sheet, &port)?;
    prepare_native_birth(context.model, write)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteParams {
    port: Uuid,
    instance_path: Vec<Uuid>,
}
fn delete_port(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteParams = parse_verb_params("datum.schematic.delete_port", params)?;
    let (source, sheet) = selected_source(context, "ports", params.port, &params.instance_path)?;
    if source
        .sheet_instances
        .values()
        .any(|instance| instance.ports.contains(&params.port))
    {
        return Err(invalid(
            "Port is bound to a sheet instance; remove its hierarchy binding first",
        ));
    }
    build_delete_schematic_port(
        context.model,
        provenance,
        sheet,
        &source.sheets[&sheet].ports[&params.port],
    )
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_port_tests.rs"]
mod tests;
