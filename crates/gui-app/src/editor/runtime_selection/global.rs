//! Complete global authored acquisition with source-owned child collapse.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass as C, AuthoredSelectionIdentity};

pub(super) fn candidates(
    authority: &NativeSelectionResolution,
    schematic: bool,
    root_sheet: Option<datum_gui_protocol::SelectionProjectId>,
) -> anyhow::Result<BTreeSet<AuthoredSelectionIdentity>> {
    let source = authority.snapshot().schematic_source();
    Ok(authority
        .authored_identities()
        .iter()
        .filter(|id| {
            if id.class == C::Pin {
                return false;
            }
            if id.class == C::Pad {
                return !schematic && authority.is_standalone_pad(id);
            }
            let is_schematic = matches!(
                id.class,
                C::Symbol
                    | C::Wire
                    | C::BusSection
                    | C::Label
                    | C::Port
                    | C::Junction
                    | C::NoConnect
                    | C::SchematicText
                    | C::SchematicGraphic
            );
            if schematic != is_schematic {
                return false;
            }
            if !schematic {
                return true;
            }
            if !id.instance_path.is_empty() {
                return false;
            }
            source
                .and_then(|s| root_sheet.and_then(|sheet| s.sheets.get(&sheet)))
                .is_some_and(|sheet| match id.class {
                    C::Symbol => sheet.symbols.contains_key(&id.id),
                    C::Wire => sheet.wires.contains_key(&id.id),
                    C::BusSection => sheet.buses.contains_key(&id.id),
                    C::Label => sheet.labels.contains_key(&id.id),
                    C::Port => sheet.ports.contains_key(&id.id),
                    C::Junction => sheet.junctions.contains_key(&id.id),
                    C::NoConnect => sheet.noconnects.contains_key(&id.id),
                    C::SchematicText => sheet.texts.contains_key(&id.id),
                    C::SchematicGraphic => sheet.drawings.contains_key(&id.id),
                    _ => false,
                })
        })
        .cloned()
        .collect())
}

#[cfg(test)]
#[path = "global/tests.rs"]
mod tests;
