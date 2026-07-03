use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::imports::{
    KiCadSchematicImportSpec, KiCadSchematicSheetCreateSpec, build_kicad_schematic_import,
    build_kicad_schematic_sheet_imports, kicad_schematic_import_map_relative_path,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    KiCadSchematicImportIdentity, import_schematic_document_with_import_map_identities,
};
use eda_engine::schematic::{Sheet, SheetDefinition, SheetInstance};
use eda_engine::substrate::{CommitSource, ImportMapEntry, ProjectResolver, SourceShardKind};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_imports_schematic_identities::{
    is_same_kicad_schematic_source_entry, schematic_generated_port_import_identities,
    schematic_import_object_source_shards,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectKiCadSchematicImportView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) source_path: String,
    pub(crate) sheet_uuid: Uuid,
    pub(crate) sheet_created: bool,
    pub(crate) import_map_path: String,
    pub(crate) imported_symbol_count: usize,
    pub(crate) imported_wire_count: usize,
    pub(crate) imported_junction_count: usize,
    pub(crate) imported_label_count: usize,
    pub(crate) imported_bus_count: usize,
    pub(crate) imported_bus_entry_count: usize,
    pub(crate) imported_noconnect_count: usize,
    pub(crate) imported_sheet_count: usize,
    pub(crate) imported_definition_count: usize,
    pub(crate) imported_instance_count: usize,
    pub(crate) imported_port_count: usize,
    pub(crate) imported_text_count: usize,
    pub(crate) imported_drawing_count: usize,
    pub(crate) import_map_entry_count: usize,
    pub(crate) created_object_count: usize,
    pub(crate) reused_existing_identity_count: usize,
}

pub(crate) fn import_native_project_kicad_schematic(
    root: &Path,
    source: &Path,
) -> Result<NativeProjectKiCadSchematicImportView> {
    let mut before = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let mut schematic_root: super::NativeSchematicRoot = serde_json::from_value(
        before
            .materialized_source_shard_value(SourceShardKind::SchematicRoot)
            .context("failed to materialize schematic root")?,
    )
    .context("failed to parse resolver-materialized schematic root")?;
    let (imported, _report, mut identities) =
        import_schematic_document_with_import_map_identities(source, &before.import_map)
            .with_context(|| format!("failed to import KiCad schematic {}", source.display()))?;
    identities.extend(schematic_generated_port_import_identities(
        &imported, source,
    ));
    let root_sheet = imported
        .sheets
        .get(&imported.uuid)
        .or_else(|| imported.sheets.values().next())
        .context("imported KiCad schematic did not contain a root sheet")?;
    let (target_sheet_id, _target_sheet_relative_path, target_sheet_created, create_sheet_spec) =
        schematic_import_target_sheet(&schematic_root, root_sheet)?;
    let sheet_id_map = schematic_sheet_id_map(imported.uuid, target_sheet_id, &imported);
    let mut sheet_create_specs = Vec::new();
    if let Some(spec) = create_sheet_spec {
        sheet_create_specs.push(spec);
    }
    sheet_create_specs.extend(schematic_missing_sheet_create_specs(
        &schematic_root,
        &imported,
        &sheet_id_map,
        target_sheet_id,
    )?);
    if !sheet_create_specs.is_empty() {
        let first_created_sheet_id = sheet_create_specs.first().map(|spec| spec.sheet_id);
        let prepared = build_kicad_schematic_sheet_imports(
            &before,
            WriteProvenance::new(
                "datum-eda-cli",
                CommitSource::Cli,
                format!(
                    "create sheet for KiCad schematic import {}",
                    source.display()
                ),
            ),
            schematic_root.uuid,
            sheet_create_specs,
        )?;
        let mut model = before;
        commit_prepared(&mut model, root, prepared)?;
        before = ProjectResolver::new(root).resolve().with_context(|| {
            format!(
                "failed to resolve imported schematic sheet {:?} in {}",
                first_created_sheet_id,
                root.display()
            )
        })?;
        schematic_root = serde_json::from_value(
            before
                .materialized_source_shard_value(SourceShardKind::SchematicRoot)
                .context("failed to materialize schematic root after sheet import")?,
        )
        .context("failed to parse resolver-materialized schematic root after sheet import")?;
    }
    let (target_sheet_id, _target_sheet_relative_path, _created, _create_sheet_spec) =
        schematic_import_target_sheet(&schematic_root, root_sheet)?;
    let object_source_shards =
        schematic_import_object_source_shards(&schematic_root, &imported, &sheet_id_map);
    let reused_existing_identity_count = identities
        .iter()
        .filter(|identity| {
            object_source_shards.contains_key(&identity.object_id)
                && before.import_map.contains_key(&identity.import_key)
        })
        .count();
    let definition_payloads =
        schematic_definition_payloads(&before.objects, &imported, &sheet_id_map)?;
    let instance_payloads = schematic_instance_payloads(&before.objects, &imported, &sheet_id_map)?;
    let source_hash = compute_source_hash_file(source)?;
    let import_map_entries = schematic_import_map_entries(
        &before.import_map,
        &identities,
        &object_source_shards,
        source,
        &source_hash,
    );
    let import_map_relative_path = kicad_schematic_import_map_relative_path(source);
    let import_map_entry_count = import_map_entries.len();
    let write = build_kicad_schematic_import(
        &before,
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            format!("import KiCad schematic root sheet {}", source.display()),
        ),
        KiCadSchematicImportSpec {
            schematic_id: schematic_root.uuid,
            imported: &imported,
            sheet_id_map: &sheet_id_map,
            definition_payloads,
            instance_payloads,
            import_map_entries,
            source,
        },
    )?;
    let created_object_count = write.created_object_count;
    if let Some(prepared) = write.prepared {
        let mut model = before;
        commit_prepared(&mut model, root, prepared)?;
    }
    let after_write = ProjectResolver::new(root).resolve().with_context(|| {
        format!(
            "failed to resolve imported schematic objects {}",
            root.display()
        )
    })?;
    for identity in identities
        .iter()
        .filter(|identity| object_source_shards.contains_key(&identity.object_id))
    {
        after_write
            .objects
            .get(&identity.object_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "imported schematic object {} ({}) was not resolver-visible",
                    identity.object_id,
                    identity.object_family
                )
            })?;
    }
    Ok(NativeProjectKiCadSchematicImportView {
        contract: "native_project_kicad_schematic_import_v1",
        project_id: after_write.project.project_id,
        source_path: source.display().to_string(),
        sheet_uuid: target_sheet_id,
        sheet_created: target_sheet_created,
        import_map_path: root.join(&import_map_relative_path).display().to_string(),
        imported_symbol_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.symbols.len())
            .sum(),
        imported_wire_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.wires.len())
            .sum(),
        imported_junction_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.junctions.len())
            .sum(),
        imported_label_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.labels.len())
            .sum(),
        imported_bus_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.buses.len())
            .sum(),
        imported_bus_entry_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.bus_entries.len())
            .sum(),
        imported_noconnect_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.noconnects.len())
            .sum(),
        imported_sheet_count: imported.sheets.len(),
        imported_definition_count: imported.sheet_definitions.len(),
        imported_instance_count: imported.sheet_instances.len(),
        imported_port_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.ports.len())
            .sum(),
        imported_text_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.texts.len())
            .sum(),
        imported_drawing_count: imported
            .sheets
            .values()
            .map(|sheet| sheet.drawings.len())
            .sum(),
        import_map_entry_count,
        created_object_count: created_object_count + usize::from(target_sheet_created),
        reused_existing_identity_count,
    })
}

fn schematic_sheet_id_map(
    imported_root_sheet_id: Uuid,
    target_root_sheet_id: Uuid,
    imported: &eda_engine::schematic::Schematic,
) -> BTreeMap<Uuid, Uuid> {
    imported
        .sheets
        .keys()
        .map(|sheet_id| {
            let target_id = if *sheet_id == imported_root_sheet_id {
                target_root_sheet_id
            } else {
                *sheet_id
            };
            (*sheet_id, target_id)
        })
        .collect()
}

fn schematic_import_target_sheet(
    schematic_root: &super::NativeSchematicRoot,
    imported_root_sheet: &Sheet,
) -> Result<(Uuid, String, bool, Option<KiCadSchematicSheetCreateSpec>)> {
    let imported_root_sheet_id = imported_root_sheet.uuid.to_string();
    if let Some(relative_path) = schematic_root.sheets.get(&imported_root_sheet_id) {
        return Ok((imported_root_sheet.uuid, relative_path.clone(), false, None));
    }
    if let Some((sheet_id, relative_path)) = schematic_root.sheets.iter().next() {
        let sheet_id = Uuid::parse_str(sheet_id)
            .with_context(|| format!("invalid native sheet UUID {sheet_id}"))?;
        return Ok((sheet_id, relative_path.clone(), false, None));
    }
    let sheet_id = imported_root_sheet.uuid;
    let relative_path = format!("sheets/{sheet_id}.json");
    let sheet = empty_native_sheet(sheet_id, &imported_root_sheet.name);
    Ok((
        sheet_id,
        relative_path.clone(),
        true,
        Some(KiCadSchematicSheetCreateSpec {
            sheet_id,
            relative_path,
            sheet: serde_json::to_value(sheet)?,
        }),
    ))
}

fn schematic_missing_sheet_create_specs(
    schematic_root: &super::NativeSchematicRoot,
    imported: &eda_engine::schematic::Schematic,
    sheet_id_map: &BTreeMap<Uuid, Uuid>,
    explicit_root_sheet_id: Uuid,
) -> Result<Vec<KiCadSchematicSheetCreateSpec>> {
    let mut sheets: Vec<&Sheet> = imported.sheets.values().collect();
    sheets.sort_by_key(|sheet| sheet.uuid);
    let mut specs = Vec::new();
    for sheet in sheets {
        let sheet_id = *sheet_id_map
            .get(&sheet.uuid)
            .context("missing schematic sheet id mapping")?;
        if sheet_id == explicit_root_sheet_id {
            continue;
        }
        if schematic_root.sheets.contains_key(&sheet_id.to_string()) {
            continue;
        }
        let relative_path = format!("sheets/{sheet_id}.json");
        let native_sheet = empty_native_sheet(sheet_id, &sheet.name);
        specs.push(KiCadSchematicSheetCreateSpec {
            sheet_id,
            relative_path,
            sheet: serde_json::to_value(native_sheet)?,
        });
    }
    Ok(specs)
}

fn empty_native_sheet(sheet_id: Uuid, name: &str) -> super::NativeSheetRoot {
    super::NativeSheetRoot {
        schema_version: 1,
        uuid: sheet_id,
        name: name.to_string(),
        frame: None,
        symbols: BTreeMap::new(),
        wires: BTreeMap::new(),
        junctions: BTreeMap::new(),
        labels: BTreeMap::new(),
        buses: BTreeMap::new(),
        bus_entries: BTreeMap::new(),
        ports: BTreeMap::new(),
        noconnects: BTreeMap::new(),
        texts: BTreeMap::new(),
        drawings: BTreeMap::new(),
    }
}

fn schematic_definition_payloads(
    existing_objects: &BTreeMap<Uuid, eda_engine::substrate::DomainObject>,
    imported: &eda_engine::schematic::Schematic,
    sheet_id_map: &BTreeMap<Uuid, Uuid>,
) -> Result<BTreeMap<Uuid, serde_json::Value>> {
    let mut payloads = BTreeMap::new();
    let definitions: Vec<&SheetDefinition> = imported.sheet_definitions.values().collect();
    for definition in definitions {
        if existing_objects.contains_key(&definition.uuid) {
            continue;
        }
        let root_sheet = *sheet_id_map
            .get(&definition.root_sheet)
            .context("missing definition root-sheet mapping")?;
        let payload = super::NativeSheetDefinitionRoot {
            schema_version: 1,
            uuid: definition.uuid,
            root_sheet,
            name: definition.name.clone(),
        };
        payloads.insert(definition.uuid, serde_json::to_value(payload)?);
    }
    Ok(payloads)
}

fn schematic_instance_payloads(
    existing_objects: &BTreeMap<Uuid, eda_engine::substrate::DomainObject>,
    imported: &eda_engine::schematic::Schematic,
    sheet_id_map: &BTreeMap<Uuid, Uuid>,
) -> Result<BTreeMap<Uuid, serde_json::Value>> {
    let mut payloads = BTreeMap::new();
    let instances: Vec<&SheetInstance> = imported.sheet_instances.values().collect();
    for instance in instances {
        if existing_objects.contains_key(&instance.uuid) {
            continue;
        }
        let payload = super::NativeSchematicInstance {
            uuid: instance.uuid,
            definition: instance.definition,
            parent_sheet: instance
                .parent_sheet
                .and_then(|sheet_id| sheet_id_map.get(&sheet_id).copied()),
            position: super::NativePoint {
                x: instance.position.x,
                y: instance.position.y,
            },
            name: instance.name.clone(),
            ports: instance.ports.clone(),
        };
        payloads.insert(instance.uuid, serde_json::to_value(payload)?);
    }
    Ok(payloads)
}

fn schematic_import_map_entries(
    existing_import_map: &BTreeMap<String, ImportMapEntry>,
    identities: &[KiCadSchematicImportIdentity],
    object_source_shards: &BTreeMap<Uuid, Uuid>,
    source: &Path,
    source_hash: &str,
) -> Vec<ImportMapEntry> {
    let source_path = source.display().to_string();
    let mut desired = BTreeMap::new();
    for identity in identities {
        let Some(source_shard_id) = object_source_shards.get(&identity.object_id).copied() else {
            continue;
        };
        desired.insert(
            identity.import_key.clone(),
            ImportMapEntry {
                import_key: identity.import_key.clone(),
                object_id: identity.object_id,
                source_shard_id,
                status: eda_engine::substrate::ImportMapEntryStatus::Active,
                source_tool: "kicad".to_string(),
                source_path: source_path.clone(),
                source_object_ref: schematic_source_object_ref(identity),
                source_hash: source_hash.to_string(),
            },
        );
    }
    for (import_key, existing) in existing_import_map {
        if desired.contains_key(import_key)
            || !is_same_kicad_schematic_source_entry(existing, &source_path)
        {
            continue;
        }
        let mut entry = existing.clone();
        entry.status = eda_engine::substrate::ImportMapEntryStatus::MissingInSource;
        entry.source_hash = source_hash.to_string();
        desired.insert(import_key.clone(), entry);
    }
    if desired.iter().all(|(key, entry)| {
        existing_import_map
            .get(key)
            .is_some_and(|existing| existing == entry)
    }) {
        Vec::new()
    } else {
        desired.into_values().collect()
    }
}

fn schematic_source_object_ref(identity: &KiCadSchematicImportIdentity) -> String {
    format!(
        "{}:{}",
        identity.object_family.replace('_', "-"),
        identity.source_uuid
    )
}
