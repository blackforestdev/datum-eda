use super::{ConvertedFootprint, Node, Result, drawings, identity, pads, refusal};
use crate::{
    ir::geometry::Polygon,
    pool::{Footprint, Package, PackageTerminal},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub(super) fn convert(
    path: PathBuf,
    text: String,
    node: Node,
    hash: String,
) -> Result<ConvertedFootprint> {
    let name = node.required_atom(1)?.to_owned();
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| refusal("non UTF-8 source identity"))?;
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let source = if parent.ends_with(".pretty") {
        format!("kicad/{parent}/{filename}")
    } else {
        path.to_string_lossy().into_owned()
    };
    // Only standalone front-oriented definitions. Board instances have another authority.
    if node.child("layer").and_then(|n| n.atom(1)) != Some("F.Cu") || node.child("at").is_some() {
        return Err(refusal(
            "only standalone front-oriented library definitions are supported",
        ));
    }
    let mut warnings = vec![
        "Source library data is unchecked; no manufacturer or standards approval is inferred.".into(),
        "Missing source mask/paste margins remain inherited and unresolved until an explicit project process basis is selected.".into(),
    ];
    let (pads, padstacks) = pads::convert(&node, &source)?;
    if pads.is_empty() {
        return Err(refusal("no supported physical pads"));
    }
    let graphics = drawings::convert(&node, &mut warnings)?;
    let package_id = identity(&source, "package");
    let footprint_id = identity(&source, "footprint");
    let terminals = pads
        .values()
        .filter(|p| !p.name.is_empty())
        .map(|p| {
            let uuid = identity(&source, &format!("terminal:{}", p.name));
            (
                uuid,
                PackageTerminal {
                    uuid,
                    name: p.name.clone(),
                    role: None,
                },
            )
        })
        .collect();
    let tags = HashSet::from(["source-format:kicad".into()]);
    let package = Package {
        uuid: package_id,
        name: name.clone(),
        package_family: None,
        package_code: None,
        mounting_type: node
            .child("attr")
            .and_then(|n| n.atom(1))
            .map(str::to_owned),
        body_dimensions: None,
        terminals,
        pads: HashMap::new(),
        courtyard: Polygon::new(vec![]),
        silkscreen: vec![],
        models_3d: vec![],
        body_height_nm: None,
        body_height_mounted_nm: None,
        tags: tags.clone(),
    };
    if node.child("model").is_some() {
        warnings.push("3D model paths/transforms are retained in source provenance; external models are not loaded or attached.".into());
    }
    let footprint = Footprint {
        uuid: footprint_id,
        name,
        package: package_id,
        pads,
        courtyard: graphics.courtyard,
        silkscreen: graphics.silkscreen,
        fab: graphics.fab,
        assembly: vec![],
        mechanical: graphics.mechanical,
        models_3d: vec![],
        standards_basis: None,
        ipc_basis: None,
        process_aperture_policy: None,
        tags,
    };
    Ok(ConvertedFootprint {
        package,
        footprint,
        padstacks,
        source_path: path,
        source_hash: hash,
        warnings,
        source_document: text,
    })
}
