use std::path::Path;

use eda_engine::import::kicad::{KiCadSchematicImportIdentity, schematic_sheet_port_import_key};
use eda_engine::schematic::Sheet;
use eda_engine::substrate::ImportMapEntry;
use uuid::Uuid;

pub(crate) fn schematic_import_object_source_shards(
    schematic_root: &super::NativeSchematicRoot,
    imported: &eda_engine::schematic::Schematic,
    sheet_id_map: &std::collections::BTreeMap<Uuid, Uuid>,
) -> std::collections::BTreeMap<Uuid, Uuid> {
    let mut object_source_shards = std::collections::BTreeMap::new();
    let root_shard_id = source_shard_id_for_relative_path("schematic/schematic.json");
    for definition in imported.sheet_definitions.values() {
        let relative_path = format!("schematic/definitions/{}.json", definition.uuid);
        object_source_shards.insert(
            definition.uuid,
            source_shard_id_for_relative_path(&relative_path),
        );
    }
    for instance in imported.sheet_instances.values() {
        object_source_shards.insert(instance.uuid, root_shard_id);
    }
    for sheet in imported.sheets.values() {
        let Some(target_sheet_id) = sheet_id_map.get(&sheet.uuid).copied() else {
            continue;
        };
        let relative_path = schematic_root
            .sheets
            .get(&target_sheet_id.to_string())
            .cloned()
            .unwrap_or_else(|| format!("sheets/{target_sheet_id}.json"));
        let source_shard_id =
            source_shard_id_for_relative_path(&format!("schematic/{relative_path}"));
        for object_id in sheet
            .symbols
            .keys()
            .chain(sheet.wires.keys())
            .chain(sheet.junctions.keys())
            .chain(sheet.labels.keys())
            .chain(sheet.ports.keys())
            .chain(sheet.buses.keys())
            .chain(sheet.bus_entries.keys())
            .chain(sheet.noconnects.keys())
            .chain(sheet.texts.keys())
            .chain(sheet.drawings.keys())
        {
            object_source_shards.insert(*object_id, source_shard_id);
        }
    }
    object_source_shards
}

pub(crate) fn schematic_generated_port_import_identities(
    imported: &eda_engine::schematic::Schematic,
    source: &Path,
) -> Vec<KiCadSchematicImportIdentity> {
    let mut identities = Vec::new();
    let mut sheets: Vec<&Sheet> = imported.sheets.values().collect();
    sheets.sort_by_key(|sheet| sheet.uuid);
    for sheet in sheets {
        let mut ports: Vec<Uuid> = sheet.ports.keys().copied().collect();
        ports.sort();
        for port_id in ports {
            identities.push(KiCadSchematicImportIdentity {
                object_family: "schematic_sheet_port",
                import_key: schematic_sheet_port_import_key(source, port_id),
                object_id: port_id,
                source_uuid: port_id,
            });
        }
    }
    identities
}

pub(crate) fn is_same_kicad_schematic_source_entry(
    entry: &ImportMapEntry,
    source_path: &str,
) -> bool {
    if entry.source_tool != "kicad" || entry.source_path != source_path {
        return false;
    }
    let Some(rest) = entry.import_key.strip_prefix("kicad:schematic-") else {
        return false;
    };
    let Some((family, _)) = rest.split_once(':') else {
        return false;
    };
    matches!(
        family,
        "symbol"
            | "wire"
            | "junction"
            | "label"
            | "bus"
            | "bus-entry"
            | "no-connect"
            | "text"
            | "drawing"
            | "sheet-definition"
            | "sheet-instance"
            | "sheet-port"
    )
}

fn source_shard_id_for_relative_path(relative_path: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    )
}
