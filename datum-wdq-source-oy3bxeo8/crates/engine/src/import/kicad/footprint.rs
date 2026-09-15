use std::collections::BTreeMap;
use std::path::Path;

use crate::error::EngineError;
use crate::import::ids_sidecar::compute_source_hash_bytes;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::ir::geometry::Polygon;
use crate::pool::{Footprint, Package, Padstack, Primitive};
use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedKiCadFootprint {
    pub package: Package,
    pub footprint: Footprint,
    pub padstacks: Vec<Padstack>,
    pub mechanical: Vec<Primitive>,
}

pub fn import_footprint_document(
    path: &Path,
) -> Result<(ImportedKiCadFootprint, ImportReport), EngineError> {
    import_footprint_document_with_import_map(path, &BTreeMap::new())
}

pub fn import_footprint_document_with_import_map(
    path: &Path,
    import_map: &BTreeMap<ImportKey, ImportMapEntry>,
) -> Result<(ImportedKiCadFootprint, ImportReport), EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let source_hash = compute_source_hash_bytes(contents.as_bytes());
    let footprint_name = super::footprint_name(&contents).unwrap_or_else(|| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("kicad-footprint")
            .to_string()
    });
    let import_key = footprint_package_import_key(path);
    let identity = allocate_import_identity(import_map, import_key.clone());
    let footprint_uuid = footprint_identity_for_package(identity.object_id);

    let (silkscreen, mechanical) = super::import_footprint_graphics(path, &contents)?;
    let (pads, padstacks) = super::import_footprint_pads(path, &contents)?;
    let courtyard = super::import_footprint_courtyard(&mechanical, &silkscreen);

    let package = Package {
        uuid: identity.object_id,
        name: footprint_name.clone(),
        package_family: None,
        package_code: None,
        mounting_type: None,
        body_dimensions: None,
        terminals: std::collections::HashMap::new(),
        pads: std::collections::HashMap::new(),
        courtyard: Polygon::new(Vec::new()),
        silkscreen: Vec::new(),
        models_3d: Vec::new(),
        body_height_nm: None,
        body_height_mounted_nm: None,
        tags: std::collections::HashSet::from([
            "source:kicad".to_string(),
            "imported:footprint".to_string(),
        ]),
    };
    let footprint = Footprint {
        uuid: footprint_uuid,
        name: footprint_name,
        package: identity.object_id,
        pads,
        courtyard,
        silkscreen,
        fab: Vec::new(),
        assembly: Vec::new(),
        mechanical: mechanical.clone(),
        models_3d: Vec::new(),
        standards_basis: None,
        ipc_basis: None,
        process_aperture_policy: Some("import_preserved".to_string()),
        tags: std::collections::HashSet::from([
            "source:kicad".to_string(),
            "imported:footprint".to_string(),
        ]),
    };
    let report = ImportReport::new(
        ImportKind::KiCadFootprint,
        path,
        ImportObjectCounts {
            padstacks: padstacks.len(),
            packages: 1,
            ..ImportObjectCounts::default()
        },
    )
    .with_metadata("pad_count", footprint.pads.len().to_string())
    .with_metadata(
        "silkscreen_primitives",
        footprint.silkscreen.len().to_string(),
    )
    .with_metadata("mechanical_primitives", mechanical.len().to_string())
    .with_metadata("import_key", import_key)
    .with_metadata("footprint_uuid", footprint_uuid.to_string())
    .with_metadata("source_hash", source_hash)
    .with_metadata(
        "reused_existing_identity",
        identity.reused_existing.to_string(),
    );

    Ok((
        ImportedKiCadFootprint {
            package,
            footprint,
            padstacks,
            mechanical,
        },
        report,
    ))
}

pub fn footprint_package_import_key(path: &Path) -> ImportKey {
    format!("kicad:footprint-package:{}", path.display())
}

fn footprint_identity_for_package(package_id: uuid::Uuid) -> uuid::Uuid {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-footprint-landpattern:{package_id}").as_bytes(),
    )
}
