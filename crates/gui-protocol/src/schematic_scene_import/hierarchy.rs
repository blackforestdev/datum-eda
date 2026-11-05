//! Typed source geometry for sheet bodies and actual parent-port memberships.
use super::*;

pub(super) fn project(
    source: &Schematic,
    sheet: &Sheet,
    graphics: &mut Vec<BoardGraphicPrimitive>,
) {
    let ports: std::collections::BTreeMap<_, _> = graphics
        .iter()
        .filter(|graphic| graphic.schematic_hit_kind() == Some(SchematicHitKind::Port))
        .map(|graphic| (graphic.source_object_uuid.clone(), graphic.clone()))
        .collect();
    for instance in sorted_values(&source.sheet_instances) {
        if instance.parent_sheet != Some(sheet.uuid) {
            continue;
        }
        let key = format!("schematic-sheet-instance:{}", instance.uuid);
        for graphic in graphics.iter_mut().filter(|graphic| {
            graphic.object_id == key && graphic.source_object_uuid == instance.uuid.to_string()
        }) {
            graphic.object_kind = "schematic_sheet_instance".into();
        }
        let unique: std::collections::BTreeSet<_> = instance.ports.iter().collect();
        for port in unique {
            let Some(source_graphic) = ports.get(&port.to_string()) else {
                continue;
            };
            let mut membership = source_graphic.clone();
            membership.object_id =
                crate::schematic_documents::hierarchy::interface_key(instance.uuid, *port);
            membership.object_kind = "schematic_sheet_interface".into();
            graphics.push(membership);
        }
    }
}

/// Reuse actual name glyphs for each membership's hit and selection presentation.
/// Base ink is rendered once by the source Port; all aliases retain its geometry.
pub(super) fn project_text(
    source: &Schematic,
    sheet: &Sheet,
    texts: &mut Vec<BoardTextPrimitive>,
    geometries: &mut Vec<BoardTextGeometryPrimitive>,
) {
    let mut alias_texts = Vec::new();
    let mut alias_geometries = Vec::new();
    let source_texts: std::collections::BTreeMap<_, _> = texts
        .iter()
        .filter_map(|text| text.source_object_uuid.as_deref().map(|id| (id, text)))
        .collect();
    let source_geometry: std::collections::BTreeMap<_, _> = geometries
        .iter()
        .map(|geometry| (geometry.object_id.as_str(), geometry))
        .collect();
    for instance in sorted_values(&source.sheet_instances) {
        if instance.parent_sheet != Some(sheet.uuid) {
            continue;
        }
        let unique: std::collections::BTreeSet<_> = instance.ports.iter().collect();
        for port in unique {
            if !sheet.ports.contains_key(port) {
                continue;
            }
            let port_id = port.to_string();
            let Some(&text) = source_texts.get(port_id.as_str()) else {
                continue;
            };
            let Some(&geometry) = source_geometry.get(text.object_id.as_str()) else {
                continue;
            };
            let key = format!(
                "{}:text",
                crate::schematic_documents::hierarchy::interface_key(instance.uuid, *port)
            );
            let mut alias_text = text.clone();
            alias_text.object_id = key.clone();
            alias_text.object_kind = "schematic_sheet_interface_text".into();
            let mut alias_geometry = geometry.clone();
            alias_geometry.object_id = key;
            alias_geometry.object_kind = "schematic_sheet_interface_text".into();
            alias_texts.push(alias_text);
            alias_geometries.push(alias_geometry);
        }
    }
    texts.extend(alias_texts);
    geometries.extend(alias_geometries);
}
