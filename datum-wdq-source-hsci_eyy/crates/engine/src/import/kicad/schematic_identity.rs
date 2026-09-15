use std::collections::BTreeMap;
use std::path::Path;

use uuid::Uuid;

use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};

use super::board_objects::KiCadSchematicImportIdentity;

pub(super) fn schematic_import_id(
    path: &Path,
    source_uuid: Uuid,
    source_family: &'static str,
    object_family: &'static str,
    import_map: Option<&BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadSchematicImportIdentity>>,
) -> Uuid {
    let Some(import_map) = import_map else {
        return source_uuid;
    };
    let allocation = allocate_import_identity(
        import_map,
        schematic_import_key(path, source_family, source_uuid),
    );
    let object_id = allocation.object_id;
    if let Some(identities) = import_identities {
        identities.push(KiCadSchematicImportIdentity::new(
            object_family,
            allocation.import_key,
            object_id,
            source_uuid,
        ));
    }
    object_id
}

pub fn schematic_symbol_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "symbol", source_uuid)
}

pub fn schematic_wire_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "wire", source_uuid)
}

pub fn schematic_junction_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "junction", source_uuid)
}

pub fn schematic_label_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "label", source_uuid)
}

pub fn schematic_bus_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "bus", source_uuid)
}

pub fn schematic_bus_entry_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "bus-entry", source_uuid)
}

pub fn schematic_noconnect_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "no-connect", source_uuid)
}

pub fn schematic_text_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "text", source_uuid)
}

pub fn schematic_drawing_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "drawing", source_uuid)
}

pub fn schematic_sheet_definition_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "sheet-definition", source_uuid)
}

pub fn schematic_sheet_instance_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "sheet-instance", source_uuid)
}

pub fn schematic_sheet_port_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    schematic_import_key(path, "sheet-port", source_uuid)
}

pub(super) fn schematic_import_key(
    path: &Path,
    source_family: &str,
    source_uuid: Uuid,
) -> ImportKey {
    format!(
        "kicad:schematic-{source_family}:{}:{source_uuid}",
        path.display()
    )
}
