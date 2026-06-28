use std::path::Path;

use uuid::Uuid;

use crate::error::EngineError;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::substrate::{ImportKey, ImportMapEntry};

use super::board_objects::KiCadSchematicImportIdentity;
use super::parser_helpers::{count_top_level_form_lines, find_top_level_uuid};
use super::schematic_identity::{
    schematic_sheet_definition_import_key, schematic_sheet_instance_import_key,
};
use super::skeleton::parse_schematic_skeleton;

pub fn import_schematic_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_schematic, report) = import_schematic_document(path)?;
    Ok(report)
}

pub fn import_schematic_document(
    path: &Path,
) -> Result<(crate::schematic::Schematic, ImportReport), EngineError> {
    import_schematic_document_inner(path, None).map(|(schematic, report, _)| (schematic, report))
}

pub fn import_schematic_document_with_import_map_identities(
    path: &Path,
    import_map: &std::collections::BTreeMap<ImportKey, ImportMapEntry>,
) -> Result<
    (
        crate::schematic::Schematic,
        ImportReport,
        Vec<KiCadSchematicImportIdentity>,
    ),
    EngineError,
> {
    import_schematic_document_inner(path, Some(import_map))
}

fn import_schematic_document_inner(
    path: &Path,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
) -> Result<
    (
        crate::schematic::Schematic,
        ImportReport,
        Vec<KiCadSchematicImportIdentity>,
    ),
    EngineError,
> {
    let contents = std::fs::read_to_string(path)?;
    let schematic_uuid = find_top_level_uuid(&contents).unwrap_or_else(Uuid::new_v4);
    let mut schematic = crate::schematic::Schematic {
        uuid: schematic_uuid,
        sheets: std::collections::HashMap::new(),
        sheet_definitions: std::collections::HashMap::new(),
        sheet_instances: std::collections::HashMap::new(),
        variants: std::collections::HashMap::new(),
        waivers: Vec::new(),
    };
    let mut import_identities = Vec::new();
    import_schematic_sheet_recursive(
        path,
        "Root",
        import_map,
        Some(&mut import_identities),
        &mut schematic,
    )?;
    let mut report = ImportReport::new(
        ImportKind::KiCadSchematic,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad schematic header and skeleton forms only; full symbol/connectivity import is not implemented yet",
    );

    if let Some(version) = extract_kicad_schematic_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    report = report
        .with_metadata(
            "symbol_count",
            count_top_level_form_lines(&contents, "symbol").to_string(),
        )
        .with_metadata(
            "wire_count",
            count_top_level_form_lines(&contents, "wire").to_string(),
        )
        .with_metadata(
            "junction_count",
            count_top_level_form_lines(&contents, "junction").to_string(),
        )
        .with_metadata(
            "label_count",
            count_top_level_form_lines(&contents, "label").to_string(),
        )
        .with_metadata(
            "global_label_count",
            count_top_level_form_lines(&contents, "global_label").to_string(),
        )
        .with_metadata(
            "hierarchical_label_count",
            count_top_level_form_lines(&contents, "hierarchical_label").to_string(),
        )
        .with_metadata(
            "bus_count",
            count_top_level_form_lines(&contents, "bus").to_string(),
        )
        .with_metadata(
            "sheet_count",
            count_top_level_form_lines(&contents, "sheet").to_string(),
        )
        .with_metadata(
            "no_connect_count",
            count_top_level_form_lines(&contents, "no_connect").to_string(),
        );

    Ok((schematic, report, import_identities))
}

fn import_schematic_sheet_recursive(
    path: &Path,
    sheet_name: &str,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadSchematicImportIdentity>>,
    schematic: &mut crate::schematic::Schematic,
) -> Result<Uuid, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let mut import_identities = import_identities;
    let parsed = parse_schematic_skeleton(
        path,
        &contents,
        sheet_name,
        import_map,
        import_identities.as_deref_mut(),
    )?;
    let sheet_uuid = parsed.root_sheet.uuid;
    schematic.sheets.insert(sheet_uuid, parsed.root_sheet);

    for child in parsed.child_sheets {
        let child_sheet_uuid = child
            .sheetfile
            .as_ref()
            .map(|sheetfile| {
                path.parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(sheetfile)
            })
            .filter(|candidate| candidate.exists())
            .map(|child_path| {
                import_schematic_sheet_recursive(
                    &child_path,
                    &child.name,
                    import_map,
                    import_identities.as_deref_mut(),
                    schematic,
                )
            })
            .transpose()?
            .unwrap_or(Uuid::nil());
        let definition_uuid = crate::ir::ids::import_uuid(
            &crate::ir::ids::namespace_kicad(),
            &format!(
                "schematic-sheet-definition/{}/{}/{}",
                path.display(),
                child.instance_uuid,
                child.name
            ),
        );
        let definition_allocation = import_map.map(|import_map| {
            crate::substrate::allocate_import_identity(
                import_map,
                schematic_sheet_definition_import_key(path, definition_uuid),
            )
        });
        let definition_object_uuid = definition_allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(definition_uuid);
        if let (Some(allocation), Some(identities)) =
            (&definition_allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadSchematicImportIdentity::new(
                "schematic_sheet_definition",
                allocation.import_key.clone(),
                allocation.object_id,
                definition_uuid,
            ));
        }
        let instance_allocation = import_map.map(|import_map| {
            crate::substrate::allocate_import_identity(
                import_map,
                schematic_sheet_instance_import_key(path, child.instance_uuid),
            )
        });
        let instance_object_uuid = instance_allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(child.instance_uuid);
        if let (Some(allocation), Some(identities)) =
            (&instance_allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadSchematicImportIdentity::new(
                "schematic_sheet_instance",
                allocation.import_key.clone(),
                allocation.object_id,
                child.instance_uuid,
            ));
        }
        schematic.sheet_definitions.insert(
            definition_object_uuid,
            crate::schematic::SheetDefinition {
                uuid: definition_object_uuid,
                root_sheet: child_sheet_uuid,
                name: child.name.clone(),
            },
        );
        schematic.sheet_instances.insert(
            instance_object_uuid,
            crate::schematic::SheetInstance {
                uuid: instance_object_uuid,
                definition: definition_object_uuid,
                parent_sheet: Some(sheet_uuid),
                position: child.position,
                name: child.name,
                ports: child.ports,
            },
        );
    }

    Ok(sheet_uuid)
}

fn extract_kicad_schematic_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}
