//! Same-library inheritance, with explicit cycle and unit-selection boundaries.
use super::{Library, Node, Result, refusal};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn resolve(library: &Library, name: &str) -> Result<Node> {
    let mut visiting = BTreeSet::new();
    inherit(library, name, &mut visiting)
}

fn inherit(library: &Library, name: &str, visiting: &mut BTreeSet<String>) -> Result<Node> {
    if visiting.len() >= 128 {
        return Err(refusal("inheritance depth exceeds 128"));
    }
    if !visiting.insert(name.to_owned()) {
        return Err(refusal(format!("inheritance cycle at {name}")));
    }
    let source = library
        .symbols
        .get(name)
        .ok_or_else(|| refusal(format!("missing symbol or inheritance parent {name}")))?;
    let mut merged: BTreeMap<String, Node> = BTreeMap::new();
    if let Some(parent) = source.child("extends") {
        let parent = inherit(library, parent.required_atom(1)?, visiting)?;
        for child in parent.items().iter().skip(2) {
            merged.insert(key(child)?, child.clone());
        }
    }
    let mut local_keys = BTreeSet::new();
    for child in source.items().iter().skip(2) {
        if child.tag() != Some("extends") {
            let key = key(child)?;
            if !local_keys.insert(key.clone()) {
                return Err(refusal(format!("duplicate symbol attribute {key}")));
            }
            merged.insert(key, child.clone());
        }
    }
    visiting.remove(name);
    let mut items = vec![Node::Atom("symbol".into()), Node::Atom(name.into())];
    items.extend(merged.into_values());
    Ok(Node::List(items))
}

fn key(node: &Node) -> Result<String> {
    let tag = node
        .tag()
        .ok_or_else(|| refusal("unexpected symbol attribute"))?;
    Ok(match tag {
        "property" => format!("property:{}", node.required_atom(1)?),
        "symbol" => {
            let (unit, style) = section(node)?;
            format!("symbol:{unit}:{style}")
        }
        _ => tag.into(),
    })
}

pub(super) fn section(node: &Node) -> Result<(u32, u32)> {
    let name = node.required_atom(1)?;
    let mut pieces = name.rsplit('_');
    let parse = |s: Option<&str>| {
        s.and_then(|s| s.parse().ok())
            .ok_or_else(|| refusal(format!("invalid unit/body name {name}")))
    };
    let style = parse(pieces.next())?;
    let unit = parse(pieces.next())?;
    Ok((unit, style))
}
