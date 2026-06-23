use std::collections::BTreeMap;
use std::path::Path;

use crate::error::EngineError;
use crate::import::ids_sidecar::compute_source_hash_bytes;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::pool::{Package, Padstack, Primitive};
use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedKiCadFootprint {
    pub package: Package,
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

    let (silkscreen, mechanical) = super::import_footprint_graphics(path, &contents)?;
    let (pads, padstacks) = super::import_footprint_pads(path, &contents)?;
    let courtyard = super::import_footprint_courtyard(&mechanical, &silkscreen);

    let package = Package {
        uuid: identity.object_id,
        name: footprint_name,
        pads,
        courtyard,
        silkscreen,
        models_3d: Vec::new(),
        body_height_nm: None,
        body_height_mounted_nm: None,
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
    .with_metadata("pad_count", package.pads.len().to_string())
    .with_metadata(
        "silkscreen_primitives",
        package.silkscreen.len().to_string(),
    )
    .with_metadata("mechanical_primitives", mechanical.len().to_string())
    .with_metadata("import_key", import_key)
    .with_metadata("source_hash", source_hash)
    .with_metadata(
        "reused_existing_identity",
        identity.reused_existing.to_string(),
    );

    Ok((
        ImportedKiCadFootprint {
            package,
            padstacks,
            mechanical,
        },
        report,
    ))
}

pub fn footprint_package_import_key(path: &Path) -> ImportKey {
    format!("kicad:footprint-package:{}", path.display())
}
