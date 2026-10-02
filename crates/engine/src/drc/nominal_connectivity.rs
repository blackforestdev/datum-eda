//! Native connectivity checks reuse complete Net-constrained physical authority.
use super::{DrcSeverity, DrcViolation};
use crate::connectivity::BoardCopperSource;
use crate::rules::ast::RuleType;
use crate::substrate::{ElectricalQueryFailure, ElectricalSelectionSnapshot};
use std::collections::BTreeSet;
use uuid::Uuid;
fn finding(code: &str, mut objects: Vec<Uuid>, message: String) -> DrcViolation {
    objects.sort();
    objects.dedup();
    DrcViolation {
        id: super::checks::stable_violation_id(code, RuleType::Connectivity, None, &objects),
        code: code.into(),
        rule_type: RuleType::Connectivity,
        severity: DrcSeverity::Error,
        message,
        location: None,
        objects,
        fingerprint: None,
        standards_basis: None,
        rule_revision: None,
        import_key: None,
        waived: false,
    }
}
pub(super) fn unavailable(objects: Vec<Uuid>, reason: String) -> DrcViolation {
    finding(
        "nominal_connectivity_unavailable",
        objects,
        format!("complete nominal connectivity unavailable: {reason}"),
    )
}
fn failure_objects(net: Uuid, failure: &ElectricalQueryFailure) -> Vec<Uuid> {
    use crate::connectivity::PhysicalQueryFailure as P;
    let mut ids = vec![net];
    match failure {
        ElectricalQueryFailure::Physical(P::UnavailableGeometry { sources, .. }) => {
            ids.extend(sources)
        }
        ElectricalQueryFailure::Physical(
            P::AbsentSource { source_id }
            | P::UnavailableAssignment { source_id }
            | P::UnavailableFill { source_id, .. }
            | P::AmbiguousOrigin { source_id }
            | P::OriginOutsideCopper { source_id },
        ) => ids.push(*source_id),
        _ => {}
    }
    ids
}
pub(super) fn run(
    snapshot: &ElectricalSelectionSnapshot,
    source: &BoardCopperSource,
) -> Vec<DrcViolation> {
    let mut result = vec![];
    // Assignment validity is a checking obligation even for isolated copper.
    let mut invalid = BTreeSet::new();
    for (id, net) in source
        .tracks
        .iter()
        .map(|(id, v)| (*id, v.net))
        .chain(source.vias.iter().map(|(id, v)| (*id, v.net)))
        .chain(source.zones.iter().map(|(id, v)| (*id, v.net)))
        .chain(
            source
                .pads
                .iter()
                .filter_map(|(id, v)| v.net.map(|net| (*id, net))),
        )
    {
        if net.is_nil() || !source.nets.get(&net).is_some_and(|n| n.uuid == net) {
            invalid.insert(id);
        }
    }
    if !invalid.is_empty() {
        result.push(unavailable(
            invalid.into_iter().collect(),
            "invalid authored Net assignment".into(),
        ));
    }
    let mut shorts = BTreeSet::new();
    for (net_id, net) in &source.nets {
        let components = match snapshot.board_net_components(source, *net_id) {
            Ok(value) => value,
            Err(failure) => {
                result.push(unavailable(
                    failure_objects(*net_id, &failure),
                    format!("{failure:?}"),
                ));
                continue;
            }
        };
        let terminals: Vec<_> = components
            .components
            .iter()
            .filter(|c| c.iter().any(|m| m.class == "pads"))
            .collect();
        if terminals.len() > 1 {
            result.push(finding(
                "connectivity_unrouted_net",
                vec![*net_id],
                format!(
                    "net {} has {} disconnected terminal copper components",
                    net.name,
                    terminals.len()
                ),
            ));
            let routed = components
                .components
                .iter()
                .flatten()
                .any(|m| m.class != "pads");
            if !routed {
                result.push(finding(
                    "connectivity_no_copper",
                    vec![*net_id],
                    format!(
                        "net {} has disconnected pins but no routed copper",
                        net.name
                    ),
                ));
            }
        } else if let Some(terminal) = terminals.first()
            && terminal.len() == 1
            && source
                .pads
                .values()
                .filter(|p| p.net == Some(*net_id))
                .count()
                == 1
        {
            result.push(finding(
                "connectivity_unconnected_pin",
                vec![*net_id],
                format!(
                    "single pin on net {} is not connected to routed copper",
                    net.name
                ),
            ));
        }
        match components.contacts {
            Ok(contacts) => {
                for contact in contacts {
                    let mut ids = vec![contact.left.source_id, contact.right.source_id];
                    ids.sort();
                    ids.dedup();
                    shorts.insert(ids);
                }
            }
            Err(failure) => {
                let failure = ElectricalQueryFailure::Physical(failure);
                result.push(unavailable(
                    failure_objects(*net_id, &failure),
                    format!("foreign contact evidence: {failure:?}"),
                ));
            }
        }
    }
    for objects in shorts {
        result.push(finding("connectivity_cross_net_contact",objects,"occupied copper contacts across distinct authored Net identities; no authored tie exception is certified".into()));
    }
    // Native consumers use these findings through the existing fingerprint,
    // sort, waiver and summary owner; no private finalization or source write.
    result
}
