//! Resolve member selectors and author only explicitly requested new scalars.
use super::*;
use crate::substrate::{ElectricalOccurrence, Operation};
use std::collections::BTreeMap;

pub(super) struct MemberSources {
    pub sources: BTreeMap<String, ElectricalOccurrence>,
    pub existing: BTreeMap<String, Uuid>,
    pub operations: Vec<Operation>,
}

pub(super) fn prepare(
    snapshot: &ElectricalSelectionSnapshot,
    sheet: &crate::schematic::Sheet,
    names: &[String],
    path: &[Uuid],
) -> Result<MemberSources, EngineError> {
    let mut result = MemberSources {
        sources: BTreeMap::new(),
        existing: BTreeMap::new(),
        operations: vec![],
    };
    let mut resolved = BTreeSet::new();
    for name in names {
        if name.is_empty()
            || name.trim() != name
            || name.chars().any(char::is_control)
            || result.sources.contains_key(name)
        {
            return Err(invalid("Declare distinct nonempty scalar member names"));
        }
        let declarations: Vec<_> = sheet
            .net_declarations
            .values()
            .filter(|d| &d.name == name)
            .collect();
        let candidates: Vec<_> = sheet
            .labels
            .values()
            .filter(|l| &l.name == name)
            .map(|l| members::origin("labels", l.uuid, path))
            .chain(
                sheet
                    .ports
                    .values()
                    .filter(|p| &p.name == name)
                    .map(|p| members::origin("ports", p.uuid, path)),
            )
            .collect();
        if declarations.len() > 1 || (!declarations.is_empty() && !candidates.is_empty()) {
            return Err(invalid(format!("Member {name} is ambiguous")));
        }
        let mut candidates = candidates;
        if let Some(declaration) = declarations.first() {
            candidates.push(members::origin("net_declarations", declaration.uuid, path));
        }
        if candidates.is_empty() {
            let schematic = snapshot
                .schematic_source()
                .ok_or_else(|| invalid("Native schematic unavailable"))?;
            if !path.is_empty()
                || schematic.sheet_instances.values().any(|i| {
                    schematic
                        .sheet_definitions
                        .get(&i.definition)
                        .is_some_and(|d| d.root_sheet == sheet.uuid)
                })
            {
                return Err(invalid(
                    "New scalar declarations require a unique root-sheet occurrence",
                ));
            }
            let declaration = crate::schematic::NetDeclaration {
                uuid: Uuid::new_v4(),
                name: name.clone(),
            };
            result.sources.insert(
                name.clone(),
                members::origin("net_declarations", declaration.uuid, &[]),
            );
            result
                .operations
                .push(Operation::CreateSchematicNetDeclaration {
                    sheet_id: sheet.uuid,
                    declaration_id: declaration.uuid,
                    declaration: serde_json::to_value(declaration)?,
                });
            continue;
        }
        let nets = candidates
            .iter()
            .map(|source| {
                snapshot.schematic_member_net(source).map_err(|e| {
                    invalid(format!("Member {name} has no canonical scalar Net: {e:?}"))
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if nets.len() != 1 {
            return Err(invalid(format!("Member {name} must name one scalar Net")));
        }
        let net = *nets.first().unwrap();
        if !resolved.insert(net) {
            return Err(invalid(
                "Different bus members resolve to the same scalar Net",
            ));
        }
        result.existing.insert(name.clone(), net);
        result.sources.insert(name.clone(), candidates.remove(0));
    }
    Ok(result)
}

impl MemberSources {
    pub(super) fn entry_member(
        &self,
        snapshot: &ElectricalSelectionSnapshot,
        name: Option<&str>,
        wire: Uuid,
        path: &[Uuid],
    ) -> Result<Option<ElectricalOccurrence>, EngineError> {
        let wire_net = snapshot
            .schematic_member_net(&members::origin("wires", wire, path))
            .map_err(|e| invalid(format!("Member wire has no canonical scalar Net: {e:?}")))?;
        let Some(name) = name else {
            if !self.existing.values().any(|id| *id == wire_net) {
                return Err(invalid("Choose the scalar member for this wire"));
            }
            return Ok(None);
        };
        let source = self
            .sources
            .get(name)
            .ok_or_else(|| invalid("Selected member is outside this Bus"))?;
        if self
            .existing
            .iter()
            .any(|(other, id)| other != name && *id == wire_net)
        {
            return Err(invalid("Wire is already on a different member of this Bus"));
        }
        if source.class != "net_declarations" {
            if self.existing.get(name) != Some(&wire_net) {
                return Err(invalid(
                    "Wire is not on the selected existing Label/Port member",
                ));
            }
            return Ok(None);
        }
        Ok(Some(source.clone()))
    }

    pub(super) fn final_nets(
        &self,
        membership: &BTreeMap<ElectricalOccurrence, Uuid>,
    ) -> Result<BTreeSet<Uuid>, EngineError> {
        let nets: BTreeSet<_> = self
            .sources
            .values()
            .map(|source| {
                membership
                    .get(source)
                    .copied()
                    .ok_or_else(|| invalid("Member has no final recorded Net"))
            })
            .collect::<Result<_, _>>()?;
        if nets.len() != self.sources.len() {
            return Err(invalid("Bus members must remain distinct scalar Nets"));
        }
        Ok(nets)
    }
}
