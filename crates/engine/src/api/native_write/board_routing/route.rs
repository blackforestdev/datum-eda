//! Source-qualified manual routing. Building or inspecting a chain never writes.
//! Structural/source admission is separate from clearance and DRC qualification.
use super::{BatchComposer, PreparedWrite, WriteProvenance};
use crate::api::native_write::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};
use crate::board::query_pad::QueryPadGeometry;
use crate::board::{Net, NetClass, Stackup, StackupLayerType, Track, occupied_copper};
use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::rules::eval::{ScopeEvalContext, eval_scope_m2};
use crate::substrate::{
    DesignModel, ModelRevision, Operation, SourceShardKind,
    component_pad_source::{self as source, ComponentPadMaterializationStatus},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.pcb.draw_track",
    build: draw_track,
}];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteBasis {
    pub project: Uuid,
    pub board: Uuid,
    pub revision: ModelRevision,
    pub tip: Option<Uuid>,
    pub start_pad: Uuid,
    pub net: Uuid,
    pub start: Point,
    pub layer: i32,
    /// A suggestion from an actual rule/class, never an invented routing default.
    pub preferred_width_nm: Option<i64>,
    pub minimum_width_nm: Option<i64>,
    pub maximum_width_nm: Option<i64>,
    pub clearance: Option<super::stop::ClearancePolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteRequest {
    pub basis: RouteBasis,
    pub end_pad: Option<Uuid>,
    pub points: Vec<Point>,
    pub width_nm: i64,
    pub edit_seed: Uuid,
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("Manual PCB route: {}", reason.into()))
}

/// Capture the actual starting terminal and current width policy without writes.
pub fn capture_route_basis(
    model: &DesignModel,
    start_pad: Uuid,
    layer: i32,
) -> Result<RouteBasis, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    if stackup.layers.iter().filter(|l| l.id == layer).count() != 1
        || !stackup
            .layers
            .iter()
            .any(|l| l.id == layer && l.layer_type == StackupLayerType::Copper)
    {
        return Err(invalid("Choose one existing copper layer"));
    }
    let (start, net) = terminal(model, &board, &stackup, start_pad, layer)?;
    let net_source: Net = serde_json::from_value(board["nets"][net.to_string()].clone())?;
    if net_source.uuid != net {
        return Err(invalid(
            "Starting Pad Net identity does not match its source",
        ));
    }
    let (minimum_width_nm, preferred_width_nm, maximum_width_nm) =
        width_policy(model, &board, &net_source, layer)?;
    Ok(RouteBasis {
        project: model.project.project_id,
        board: serde_json::from_value(board["uuid"].clone())?,
        revision: model.model_revision.clone(),
        tip: crate::revision::transaction_tip(&model.journal),
        start_pad,
        net,
        start,
        layer,
        preferred_width_nm,
        minimum_width_nm,
        maximum_width_nm,
        clearance: super::stop::clearance_policy(model, &net_source, layer)?,
    })
}

fn terminal(
    model: &DesignModel,
    board: &serde_json::Value,
    stackup: &Stackup,
    id: Uuid,
    layer: i32,
) -> Result<(Point, Uuid), EngineError> {
    let projection = source::component_pad_projection(model, board)?;
    if let Some(reason) = projection.unavailable.get(&id) {
        return Err(invalid(format!("Pad source is unavailable: {reason}")));
    }
    let geometry = if let Some(geometry) = projection.query_pads.get(&id) {
        if projection.statuses.get(&id) != Some(&ComponentPadMaterializationStatus::Current) {
            return Err(invalid("Pad materialization is not current"));
        }
        geometry.clone()
    } else {
        let value = board["pads"]
            .get(id.to_string())
            .ok_or_else(|| invalid("Starting or ending Pad source is unavailable"))?;
        QueryPadGeometry::Integer(serde_json::from_value(value.clone())?)
    };
    // Source-relative centres are exact irrational positions, not their local
    // integer terminal coordinates. Do not silently round or route from those.
    let QueryPadGeometry::Integer(pad) = geometry else {
        return Err(invalid(
            "This Pad requires source-relative routing coordinates",
        ));
    };
    if pad.uuid != id {
        return Err(invalid("Pad identity differs from its source key"));
    }
    occupied_copper::validate_pad(&pad)
        .map_err(|e| invalid(format!("Pad geometry unavailable: {e:?}")))?;
    if !occupied_copper::pad_layers(stackup, &pad)
        .map_err(|e| invalid(format!("Pad copper layers unavailable: {e:?}")))?
        .contains(&layer)
    {
        return Err(invalid("Pad has no copper on the chosen layer"));
    }
    let net = pad
        .net
        .filter(|id| !id.is_nil())
        .ok_or_else(|| invalid("Pad has no qualified Net"))?;
    if board["nets"].get(net.to_string()).is_none() {
        return Err(invalid("Pad Net has no current board source"));
    }
    Ok((pad.position, net))
}

fn width_policy(
    model: &DesignModel,
    board: &serde_json::Value,
    net: &Net,
    layer: i32,
) -> Result<(Option<i64>, Option<i64>, Option<i64>), EngineError> {
    let class = if net.class.is_nil() {
        None
    } else {
        let value = board["net_classes"]
            .get(net.class.to_string())
            .ok_or_else(|| invalid("Net class source is unavailable"))?;
        let class: NetClass = serde_json::from_value(value.clone())?;
        if class.uuid != net.class {
            return Err(invalid("Net class identity differs from its source"));
        }
        Some(class)
    };
    if let Some(rule) = selected_rule(model, "track_width", net, layer)? {
        let parameter = |name: &str| {
            rule["params"][name]
                .as_i64()
                .ok_or_else(|| invalid("Track width parameter is not an integer"))
        };
        return Ok((
            Some(parameter("min_nm")?),
            Some(parameter("preferred_nm")?),
            Some(parameter("max_nm")?),
        ));
    }
    if let Some(class) = class {
        if class.track_width <= 0 {
            return Err(invalid("Net class width source is invalid"));
        }
        // Retain the existing native DRC interpretation of class track_width.
        return Ok((Some(class.track_width), Some(class.track_width), None));
    }
    Ok((None, None, None))
}

/// Existing native rule schema and supported scope evaluation. No fallback or
/// implicit resolution of an ambiguous/unsupported rule is supplied here.
pub(super) fn selected_rule(
    model: &DesignModel,
    kind: &str,
    net: &Net,
    layer: i32,
) -> Result<Option<serde_json::Value>, EngineError> {
    let rules = model.materialized_source_shard_value(SourceShardKind::RulesRoot)?;
    let rules = rules["rules"]
        .as_array()
        .ok_or_else(|| invalid("Rules source has no rule array"))?;
    let context = ScopeEvalContext {
        net: Some(net.uuid),
        net_class: Some(net.class),
        layer: Some(layer),
    };
    let mut applicable = Vec::new();
    for rule in rules {
        crate::substrate::validate_native_project_rule_payload(rule)?;
        if rule["enabled"] == true
            && rule["rule_type"] == kind
            && eval_scope_m2(&serde_json::from_value(rule["scope"].clone())?, context)?
        {
            applicable.push(rule);
        }
    }
    applicable.sort_by_key(|r| std::cmp::Reverse(r["priority"].as_u64()));
    if let Some(rule) = applicable.first() {
        if applicable
            .get(1)
            .is_some_and(|other| other["priority"] == rule["priority"])
        {
            return Err(invalid(format!(
                "{kind} rules have ambiguous equal priority"
            )));
        }
        return Ok(Some((*rule).clone()));
    }
    Ok(None)
}

/// Prepare exactly one complete straight-segment chain. No autorouting, derived
/// copper refresh, electrical assignment or unrelated board mutation is implied.
pub fn prepare_route(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &RouteRequest,
) -> Result<PreparedWrite, EngineError> {
    let basis = capture_route_basis(model, request.basis.start_pad, request.basis.layer)?;
    if basis != request.basis {
        return Err(invalid(
            "Route basis changed; restart routing from the current Pad",
        ));
    }
    if request.width_nm <= 0
        || basis
            .minimum_width_nm
            .is_some_and(|min| request.width_nm < min)
        || basis
            .maximum_width_nm
            .is_some_and(|max| request.width_nm > max)
    {
        return Err(invalid(
            "Width must be positive and satisfy the current width constraints",
        ));
    }
    if request.edit_seed.is_nil()
        || request.points.len() < 2
        || request.points.first() != Some(&basis.start)
    {
        return Err(invalid(
            "Route needs a retained edit identity and at least two points starting at the Pad centre",
        ));
    }
    if let Some(end) = request.end_pad {
        let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
        let stackup = serde_json::from_value(board["stackup"].clone())?;
        let (position, net) = terminal(model, &board, &stackup, end, basis.layer)?;
        if net != basis.net || request.points.last() != Some(&position) {
            return Err(invalid(
                "Ending Pad must be on the same Net and its centre must terminate the chain",
            ));
        }
    }
    let mut composer = BatchComposer::compose(model, provenance);
    let mut primary = None;
    let mut tracks = Vec::new();
    for (index, points) in request.points.windows(2).enumerate() {
        if points[0] == points[1] {
            return Err(invalid("A route segment has zero length"));
        }
        let id = Uuid::new_v5(
            &request.edit_seed,
            format!("{}:manual-route:{index}", basis.project).as_bytes(),
        );
        if model.objects.contains_key(&id)
            || model
                .journal
                .iter()
                .any(|t| t.diff.created.contains(&id) || t.diff.deleted.contains(&id))
        {
            return Err(invalid(
                "Route identity has already been used; restart the route",
            ));
        }
        let track = Track::straight(
            id,
            basis.net,
            points[0],
            points[1],
            request.width_nm,
            basis.layer,
        );
        occupied_copper::validate_track(&track)
            .map_err(|e| invalid(format!("Unsupported track geometry: {e:?}")))?;
        let radius = request.width_nm / 2 + request.width_nm % 2;
        for point in points {
            if [point.x, point.y]
                .into_iter()
                .any(|v| v.checked_add(radius).is_none() || v.checked_sub(radius).is_none())
            {
                return Err(invalid("Track copper extent exceeds integer coordinates"));
            }
        }
        primary.get_or_insert(id);
        tracks.push(track);
    }
    super::stop::StopContext::capture(model, &basis)?.validate_tracks(&tracks)?;
    for track in tracks {
        composer = composer.push_op(Operation::CreateBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track)?,
        });
    }
    composer
        .primary_object(primary.ok_or_else(|| invalid("Empty route"))?)
        .finish()
}

fn draw_track(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    prepare_route(
        context.model,
        provenance,
        &parse_verb_params("datum.pcb.draw_track", params)?,
    )
}

#[cfg(test)]
#[path = "route_tests.rs"]
pub(crate) mod tests;
