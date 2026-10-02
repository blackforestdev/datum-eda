//! Internal complete scalar partitions for authored identity transitions.
//! Uses existing segment and label semantics; no public membership query.
use super::{canonical_label_name, is_bus_container_label, point_on_wire_segment};
use crate::{
    error::EngineError,
    schematic::{LabelKind, Schematic, Sheet},
    substrate::ElectricalOccurrence,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Context {
    sheet: Uuid,
    path: Vec<Uuid>,
}

#[derive(Default)]
struct Groups {
    parent: BTreeMap<ElectricalOccurrence, ElectricalOccurrence>,
}
impl Groups {
    fn root(&mut self, member: &ElectricalOccurrence) -> ElectricalOccurrence {
        let mut current = member.clone();
        let mut path = Vec::new();
        loop {
            let parent = self
                .parent
                .entry(current.clone())
                .or_insert_with(|| current.clone())
                .clone();
            if parent == current {
                break;
            }
            path.push(current);
            current = parent;
        }
        for child in path {
            self.parent.insert(child, current.clone());
        }
        current
    }
    fn join(&mut self, a: &ElectricalOccurrence, b: &ElectricalOccurrence) {
        let a = self.root(a);
        let b = self.root(b);
        if a != b {
            let (a, b) = if a < b { (a, b) } else { (b, a) };
            self.parent.insert(b, a);
        }
    }
}
fn reference(context: &Context, class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: context.path.clone(),
    }
}

pub(crate) fn partitions(
    schematic: &Schematic,
) -> Result<Vec<BTreeSet<ElectricalOccurrence>>, EngineError> {
    let contexts = contexts(schematic)?;
    let mut groups = Groups::default();
    let mut global = BTreeMap::<String, ElectricalOccurrence>::new();
    for context in &contexts {
        let sheet = &schematic.sheets[&context.sheet];
        local(context, sheet, &mut groups, true);
        for label in sheet
            .labels
            .values()
            .filter(|l| matches!(l.kind, LabelKind::Global) && !is_bus_container_label(l))
        {
            let member = reference(context, "labels", label.uuid);
            if let Some(first) = global.get(&canonical_label_name(&label.name)) {
                groups.join(first, &member);
            } else {
                global.insert(canonical_label_name(&label.name), member);
            }
        }
    }
    // Each child context is linked only to its own parent occurrence, using
    // the existing explicit instance.ports -> unique child label contract.
    for context in &contexts {
        let Some(instance_id) = context.path.last() else {
            continue;
        };
        let instance = &schematic.sheet_instances[instance_id];
        let Some(parent_id) = instance.parent_sheet else {
            continue;
        };
        let parent = &schematic.sheets[&parent_id];
        let child = &schematic.sheets[&context.sheet];
        let parent_context = Context {
            sheet: parent_id,
            path: context.path[..context.path.len() - 1].to_vec(),
        };
        let mut names = BTreeSet::new();
        for port_id in &instance.ports {
            let port = parent
                .ports
                .get(port_id)
                .ok_or_else(|| invalid("missing parent port"))?;
            if !names.insert(&port.name) {
                return Err(invalid("multiply mapped parent port"));
            }
            let labels: Vec<_> = child
                .labels
                .values()
                .filter(|l| {
                    matches!(l.kind, LabelKind::Hierarchical)
                        && !is_bus_container_label(l)
                        && l.name == port.name
                })
                .collect();
            if labels.len() != 1 {
                return Err(invalid("missing/multiply mapped child port"));
            }
            groups.join(
                &reference(&parent_context, "ports", port.uuid),
                &reference(context, "labels", labels[0].uuid),
            );
        }
        if child.labels.values().any(|l| {
            matches!(l.kind, LabelKind::Hierarchical)
                && !is_bus_container_label(l)
                && !names.contains(&l.name)
        }) {
            return Err(invalid("unbound child hierarchical label"));
        }
    }
    Ok(finish(groups))
}

pub(crate) fn physical_partitions(
    schematic: &Schematic,
) -> Result<Vec<BTreeSet<ElectricalOccurrence>>, EngineError> {
    let mut groups = Groups::default();
    for context in contexts(schematic)? {
        local(
            &context,
            &schematic.sheets[&context.sheet],
            &mut groups,
            false,
        );
    }
    Ok(finish(groups))
}

pub(crate) fn representation_occurrences(
    schematic: &Schematic,
) -> Result<BTreeSet<ElectricalOccurrence>, EngineError> {
    let mut result = BTreeSet::new();
    for context in contexts(schematic)? {
        let sheet = &schematic.sheets[&context.sheet];
        for (class, ids) in [
            ("buses", sheet.buses.keys().copied().collect::<Vec<_>>()),
            ("bus_entries", sheet.bus_entries.keys().copied().collect()),
            ("labels", sheet.labels.keys().copied().collect()),
            ("ports", sheet.ports.keys().copied().collect()),
        ] {
            result.extend(ids.into_iter().map(|id| reference(&context, class, id)));
        }
    }
    Ok(result)
}

pub(crate) fn occurrence_sheet<'a>(
    schematic: &'a Schematic,
    origin: &ElectricalOccurrence,
) -> Result<&'a Sheet, EngineError> {
    let found: Vec<_> = contexts(schematic)?
        .into_iter()
        .filter(|c| {
            c.path == origin.instance_path
                && schematic.sheets[&c.sheet]
                    .buses
                    .contains_key(&origin.source_id)
        })
        .collect();
    if found.len() != 1 {
        return Err(invalid("Bus origin has no unique sheet occurrence"));
    }
    Ok(&schematic.sheets[&found[0].sheet])
}

fn finish(mut groups: Groups) -> Vec<BTreeSet<ElectricalOccurrence>> {
    let mut result = BTreeMap::<ElectricalOccurrence, BTreeSet<ElectricalOccurrence>>::new();
    for member in groups.parent.keys().cloned().collect::<Vec<_>>() {
        result
            .entry(groups.root(&member))
            .or_default()
            .insert(member);
    }
    let mut result: Vec<_> = result.into_values().collect();
    result.sort();
    result
}

fn local(context: &Context, sheet: &Sheet, groups: &mut Groups, resolve_names: bool) {
    let mut endpoints = BTreeMap::<(i64, i64), ElectricalOccurrence>::new();
    let mut names = BTreeMap::<(u8, String), ElectricalOccurrence>::new();
    let mut attachments = Vec::new();
    for wire in sheet.wires.values() {
        let member = reference(context, "wires", wire.uuid);
        groups.root(&member);
        for point in [wire.from, wire.to] {
            if let Some(first) = endpoints.get(&(point.x, point.y)) {
                groups.join(first, &member);
            } else {
                endpoints.insert((point.x, point.y), member.clone());
            }
        }
    }
    for junction in sheet.junctions.values() {
        attachments.push((
            reference(context, "junctions", junction.uuid),
            junction.position,
        ));
    }
    for label in sheet.labels.values().filter(|l| !is_bus_container_label(l)) {
        let member = reference(context, "labels", label.uuid);
        // Local labels join only inside this occurrence. Hierarchical labels
        // retain the existing same-sheet interface scope before explicit links.
        if resolve_names && !matches!(label.kind, LabelKind::Global) {
            let name = (
                if matches!(label.kind, LabelKind::Local) {
                    0
                } else {
                    1
                },
                canonical_label_name(&label.name),
            );
            if let Some(first) = names.get(&name) {
                groups.join(first, &member);
            } else {
                names.insert(name, member.clone());
            }
        }
        attachments.push((member, label.position));
    }
    for port in sheet.ports.values() {
        let member = reference(context, "ports", port.uuid);
        if resolve_names {
            if let Some(first) = names.get(&(1, port.name.clone())) {
                groups.join(first, &member);
            } else {
                names.insert((1, port.name.clone()), member.clone());
            }
        }
        attachments.push((member, port.position));
    }
    for symbol in sheet.symbols.values() {
        for pin in &symbol.pins {
            attachments.push((reference(context, "pins", pin.uuid), pin.position));
        }
    }
    for (member, point) in attachments {
        groups.root(&member);
        for wire in sheet.wires.values() {
            if point_on_wire_segment(point, wire.from, wire.to) {
                groups.join(&member, &reference(context, "wires", wire.uuid));
            }
        }
        // Co-located authored attachment objects are on the same electrical node.
        if let Some(first) = endpoints.get(&(point.x, point.y)) {
            groups.join(first, &member);
        } else {
            endpoints.insert((point.x, point.y), member);
        }
    }
}

fn contexts(schematic: &Schematic) -> Result<BTreeSet<Context>, EngineError> {
    let mut result = BTreeSet::new();
    let instantiated: BTreeSet<_> = schematic
        .sheet_instances
        .values()
        .map(|i| {
            schematic
                .sheet_definitions
                .get(&i.definition)
                .map(|d| d.root_sheet)
                .ok_or_else(|| invalid("missing sheet definition"))
        })
        .collect::<Result<_, _>>()?;
    for sheet in schematic
        .sheets
        .keys()
        .filter(|s| !instantiated.contains(s))
    {
        visit(schematic, *sheet, vec![], &mut BTreeSet::new(), &mut result)?;
    }
    for instance in schematic
        .sheet_instances
        .values()
        .filter(|i| i.parent_sheet.is_none())
    {
        visit(
            schematic,
            schematic.sheet_definitions[&instance.definition].root_sheet,
            vec![instance.uuid],
            &mut BTreeSet::new(),
            &mut result,
        )?;
    }
    if schematic
        .sheet_instances
        .values()
        .any(|i| !result.iter().any(|c| c.path.last() == Some(&i.uuid)))
    {
        return Err(invalid("cyclic/unreachable sheet instance"));
    }
    Ok(result)
}
fn visit(
    schematic: &Schematic,
    sheet: Uuid,
    path: Vec<Uuid>,
    ancestors: &mut BTreeSet<Uuid>,
    result: &mut BTreeSet<Context>,
) -> Result<(), EngineError> {
    if !schematic.sheets.contains_key(&sheet) || !ancestors.insert(sheet) {
        return Err(invalid("missing/cyclic occurrence sheet"));
    }
    if !result.insert(Context {
        sheet,
        path: path.clone(),
    }) {
        return Err(invalid("duplicate occurrence context"));
    }
    for instance in schematic
        .sheet_instances
        .values()
        .filter(|i| i.parent_sheet == Some(sheet))
    {
        let mut child = path.clone();
        child.push(instance.uuid);
        visit(
            schematic,
            schematic.sheet_definitions[&instance.definition].root_sheet,
            child,
            ancestors,
            result,
        )?;
    }
    ancestors.remove(&sheet);
    Ok(())
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid occurrence topology: {reason}"))
}
