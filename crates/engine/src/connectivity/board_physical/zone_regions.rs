//! Engine-owned component tokens and basis-bound succession. No semantic UUIDs.
use super::*;
use crate::board::occupied_region::equivalent;
use crate::substrate::Operation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneRegionQualifier {
    project_id: Uuid,
    revision: ModelRevision,
    source_basis: Vec<SelectionSourceBasis>,
    source: ElectricalOccurrence,
    // Private graph component index, valid only in its exact captured basis.
    // Never a renderer polygon index or a persistent semantic identity.
    component: usize,
}
impl ZoneRegionQualifier {
    pub fn source(&self) -> &ElectricalOccurrence {
        &self.source
    }
    pub fn revision(&self) -> &ModelRevision {
        &self.revision
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneClearReason {
    Deleted,
    CurrentEmpty,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneRegionSuccessor {
    Unique {
        qualifier: ZoneRegionQualifier,
        run: Box<BoardRunMembership>,
    },
    Split {
        successors: Vec<ZoneRegionQualifier>,
    },
    Cleared {
        reason: ZoneClearReason,
    },
    Suspended {
        failure: PhysicalQueryFailure,
    },
    Unknown,
}
pub(super) fn qualifier(
    model: &DesignModel,
    basis: &[SelectionSourceBasis],
    id: Uuid,
    component: usize,
) -> ZoneRegionQualifier {
    ZoneRegionQualifier {
        project_id: model.project.project_id,
        revision: model.model_revision.clone(),
        source_basis: basis.to_vec(),
        source: reference("zones", id),
        component,
    }
}
fn polygons(graph: &Graph, component: usize, id: Uuid) -> Vec<Polygon> {
    graph.groups[component]
        .iter()
        .filter_map(|i| {
            let n = &graph.nodes[*i];
            match &n.shape {
                Shape::Fill(p) if n.reference.source_id == id => Some(p.clone()),
                _ => None,
            }
        })
        .collect()
}
fn all_polygons(graph: &Graph, id: Uuid) -> Vec<Polygon> {
    graph
        .nodes
        .iter()
        .filter_map(|n| match &n.shape {
            Shape::Fill(p) if n.reference.source_id == id => Some(p.clone()),
            _ => None,
        })
        .collect()
}
fn current_basis(old: &DesignModel, new: &DesignModel) -> bool {
    old.project.project_id == new.project.project_id
        && new.journal.starts_with(&old.journal)
        && new
            .journal
            .get(old.journal.len())
            .map_or(old.model_revision == new.model_revision, |tx| {
                tx.before_model_revision == old.model_revision
            })
}
fn translation(old: &DesignModel, new: &DesignModel, a: &Zone, b: &Zone) -> Option<Point> {
    // An exact single canonical source edit supplies the operation witness.
    // Verify both source endpoints and translated occupied fill separately.
    let mut writes = vec![];
    for tx in &new.journal[old.journal.len()..] {
        for op in &tx.operations {
            match op {
                Operation::SetBoardZone { zone_id, zone } if *zone_id == a.uuid => {
                    writes.push(zone)
                }
                Operation::CreateBoardZone { zone_id, .. }
                | Operation::DeleteBoardZone { zone_id, .. }
                    if *zone_id == a.uuid =>
                {
                    return None;
                }
                _ => {}
            }
        }
    }
    if writes.len() != 1
        || *writes[0] != serde_json::to_value(b).ok()?
        || a.layer != b.layer
        || a.net != b.net
    {
        return None;
    }
    let first_a = a.polygon.vertices.first()?;
    let first_b = b.polygon.vertices.first()?;
    let shift = Point::new(
        first_b.x.checked_sub(first_a.x)?,
        first_b.y.checked_sub(first_a.y)?,
    );
    let mut expected = a.clone();
    expected.polygon = translated(std::slice::from_ref(&a.polygon), shift, a.uuid)
        .ok()?
        .pop()?;
    (expected == *b).then_some(shift)
}
fn translated(
    polygons: &[Polygon],
    shift: Point,
    id: Uuid,
) -> Result<Vec<Polygon>, PhysicalQueryFailure> {
    polygons
        .iter()
        .map(|p| {
            let mut p = p.clone();
            for v in &mut p.vertices {
                v.x =
                    v.x.checked_add(shift.x)
                        .ok_or_else(|| geometry([id], GeometryError::ArithmeticRange))?;
                v.y =
                    v.y.checked_add(shift.y)
                        .ok_or_else(|| geometry([id], GeometryError::ArithmeticRange))?;
            }
            Ok(p)
        })
        .collect()
}
pub(crate) fn successor(
    old_source: &BoardCopperSource,
    old: &DesignModel,
    old_basis: &[SelectionSourceBasis],
    new_source: &BoardCopperSource,
    new: &DesignModel,
    new_basis: &[SelectionSourceBasis],
    token: &ZoneRegionQualifier,
) -> Result<ZoneRegionSuccessor, PhysicalQueryFailure> {
    if !current_basis(old, new)
        || *token != qualifier(old, old_basis, token.source.source_id, token.component)
    {
        return Err(PhysicalQueryFailure::InvalidQualifier);
    }
    let id = token.source.source_id;
    let old_net = assignment(old_source, &token.source)?;
    let old_graph = Graph::build(old_source, old, old_net)?;
    if token.component >= old_graph.groups.len() {
        return Err(PhysicalQueryFailure::InvalidQualifier);
    }
    let region = polygons(&old_graph, token.component, id);
    if region.is_empty() {
        return Err(PhysicalQueryFailure::InvalidQualifier);
    }
    let Some(new_zone) = new_source.zones.get(&id) else {
        if new.objects.contains_key(&id) {
            return Err(PhysicalQueryFailure::InvalidQualifier);
        }
        if !new.journal[old.journal.len()..]
            .iter()
            .flat_map(|tx| &tx.operations)
            .any(|op| matches!(op, Operation::DeleteBoardZone { zone_id, .. } if *zone_id == id))
        {
            return Ok(ZoneRegionSuccessor::Unknown);
        }
        return Ok(ZoneRegionSuccessor::Cleared {
            reason: ZoneClearReason::Deleted,
        });
    };
    if current_fill(new, id).is_some_and(|fill| fill.islands.is_empty()) {
        return Ok(ZoneRegionSuccessor::Cleared {
            reason: ZoneClearReason::CurrentEmpty,
        });
    }
    let new_net = match assignment(new_source, &token.source) {
        Ok(net) => net,
        Err(failure) => return Ok(ZoneRegionSuccessor::Suspended { failure }),
    };
    let new_graph = match Graph::build(new_source, new, new_net) {
        Ok(graph) => graph,
        Err(failure) => return Ok(ZoneRegionSuccessor::Suspended { failure }),
    };
    let all_new = all_polygons(&new_graph, id);
    if all_new.is_empty() {
        return Ok(ZoneRegionSuccessor::Cleared {
            reason: ZoneClearReason::CurrentEmpty,
        });
    }
    let old_zone = &old_source.zones[&id];
    if old_zone.layer != new_zone.layer || old_zone.net != new_zone.net {
        return Ok(ZoneRegionSuccessor::Unknown);
    }
    let all_old = all_polygons(&old_graph, id);
    let eq = |a: &[Polygon], b: &[Polygon]| equivalent(a, b).map_err(|e| geometry([id], e));
    let (region, partition_witness) = if eq(&all_old, &all_new)? {
        (region, true)
    } else if let Some(shift) = translation(old, new, old_zone, new_zone) {
        let moved_all = translated(&all_old, shift, id)?;
        if eq(&moved_all, &all_new)? {
            (translated(&region, shift, id)?, true)
        } else {
            (region, false)
        }
    } else {
        (region, false)
    };
    let mut successors = vec![];
    for index in 0..new_graph.groups.len() {
        let current = polygons(&new_graph, index, id);
        if current.is_empty() {
            continue;
        }
        let mut matches = eq(&region, &current)?;
        if partition_witness {
            // Complete Zone copper equivalence certifies this partition mapping.
            // Contact here is not used as lineage for changed occupied regions.
            for a in &region {
                for b in &current {
                    matches |= copper::polygons_within(a, b, 0, DistanceBoundary::Inclusive)
                        .map_err(|e| geometry([id], e))?;
                }
            }
        }
        if matches {
            successors.push((index, qualifier(new, new_basis, id, index)));
        }
    }
    match successors.len() {
        1 => {
            let (index, qualifier) = successors.pop().unwrap();
            let run = membership(new_source, new, &token.source, &new_graph, index, new_basis)?;
            Ok(ZoneRegionSuccessor::Unique {
                qualifier,
                run: Box::new(run),
            })
        }
        n if n > 1 && partition_witness => Ok(ZoneRegionSuccessor::Split {
            successors: successors.into_iter().map(|(_, q)| q).collect(),
        }),
        _ => Ok(ZoneRegionSuccessor::Unknown),
    }
}
