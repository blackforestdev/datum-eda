//! Selected-symbol conversion after inheritance resolution.
use super::{ConvertedSymbol, Library, Result, geometry, pins, refusal, resolve};
use crate::ir::ids::{import_uuid, namespace_kicad};
use crate::pool::{LibraryObjectProvenance, LibrarySymbolField, Symbol, Unit};
use std::collections::{HashMap, HashSet};

pub(super) fn convert(library: &Library, name: &str) -> Result<ConvertedSymbol> {
    let source = resolve::resolve(library, name)?;
    if source.flag("power") {
        return Err(refusal(
            "power symbols require unsupported implicit net naming semantics",
        ));
    }
    let identity = format!("symbol-library:{}:{name}", library.path.display());
    let unit_id = import_uuid(&namespace_kicad(), &format!("{identity}:unit"));
    let symbol_id = import_uuid(&namespace_kicad(), &format!("{identity}:symbol"));
    let mut warnings = vec![
        "Local source acquisition does not establish library approval or standards compliance.".into(),
        "Source fonts, property rotation/justification, pin label visibility and offsets use native display defaults.".into(),
        "Source footprint, simulation and BOM metadata remain descriptive fields; no physical assignment or simulation model is created.".into(),
    ];
    let mut fields = Vec::new();
    let mut prefix = None;
    for property in source.children("property") {
        let key = property.required_atom(1)?.to_owned();
        let value = property.required_atom(2)?.to_owned();
        if key == "Reference" {
            prefix = Some(value.clone());
        }
        let visible = !property.child("effects").is_some_and(|e| e.flag("hide"));
        fields.push(LibrarySymbolField {
            key,
            value,
            position: property.child("at").map(geometry::point).transpose()?,
            visible,
        });
    }
    let mut drawings = Vec::new();
    let mut logical_pins = HashMap::new();
    let mut anchors = Vec::new();
    for section in source.children("symbol") {
        let (unit, style) = resolve::section(section)?;
        if unit > 1 {
            return Err(refusal(
                "multiunit symbols require explicit native gate selection and are not supported",
            ));
        }
        if style > 1 {
            warnings.push("Only the normal body style is acquired; alternate body styles remain in the source library.".into());
            continue;
        }
        for item in section.items().iter().skip(2) {
            if item.tag() == Some("pin") {
                let (pin, anchor) = pins::pin(item, &identity, &mut warnings)?;
                if logical_pins.insert(pin.uuid, pin).is_some() {
                    return Err(refusal(
                        "duplicate pin number across selected unit/common sections",
                    ));
                }
                anchors.push(anchor);
            } else {
                drawings.extend(geometry::drawing(item, &mut warnings)?);
            }
        }
    }
    if drawings.is_empty() || anchors.is_empty() {
        return Err(refusal(
            "selected symbol must provide supported body geometry and electrical pins",
        ));
    }
    let source_path = library.path.to_string_lossy().into_owned();
    let symbol = Symbol {
        uuid: symbol_id,
        name: name.into(),
        unit: unit_id,
        fields,
        default_refdes_prefix: prefix,
        style_profile_assertions: Vec::new(),
        standards_basis: None,
        check_state: None,
        provenance: Some(LibraryObjectProvenance {
            source: Some(format!("{source_path}#{name}")),
            source_hash: Some(library.hash.clone()),
            reviewed_by: None,
            reviewed_at: None,
        }),
        drawings,
        pin_anchors: anchors,
    };
    warnings.sort();
    warnings.dedup();
    Ok(ConvertedSymbol {
        unit: Unit {
            uuid: unit_id,
            name: name.into(),
            manufacturer: String::new(),
            pins: logical_pins,
            tags: HashSet::new(),
        },
        symbol,
        warnings,
        source_hash: library.hash.clone(),
        source_path: library.path.clone(),
        source_symbol: name.into(),
    })
}
