//! PM057 supported documentation graphics and explicit loss/disposition reporting.
mod fields;
use super::board_objects::KiCadBoardImportIdentity;
use crate::{
    board::{
        Board, StackupLayerType,
        graphics::{BoardGraphic, GraphicGeometry as G},
    },
    error::EngineError,
    substrate::{ImportKey, ImportMapEntry, ImportMapEntryStatus, allocate_import_identity},
};
use fields::*;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct GraphicImportDisposition {
    pub form: String,
    pub source_uuid: Option<Uuid>,
    pub disposition: &'static str,
    pub explanation: String,
}
#[derive(Debug, Clone)]
pub struct NativeGraphicImport {
    pub graphics: Vec<BoardGraphic>,
    pub identities: Vec<KiCadBoardImportIdentity>,
    pub dispositions: Vec<GraphicImportDisposition>,
}
pub fn board_graphic_import_key(path: &Path, uuid: Uuid) -> ImportKey {
    format!("kicad:board-graphic:{}:{uuid}", path.display())
}

pub fn native_graphic_sources(
    source: &Path,
    import_map: &BTreeMap<ImportKey, ImportMapEntry>,
    board: &Board,
) -> Result<NativeGraphicImport, EngineError> {
    parse(source, &std::fs::read_to_string(source)?, import_map, board)
}
fn source_uuid(block: &str) -> Result<Uuid, EngineError> {
    let name = match (field(block, "uuid")?, field(block, "tstamp")?) {
        (Some(_), None) => "uuid",
        (None, Some(_)) => "tstamp",
        _ => return Err(refusal("missing or ambiguous graphic source UUID")),
    };
    scalar(block, name)?
        .parse::<Uuid>()
        .ok()
        .filter(|id| !id.is_nil())
        .ok_or_else(|| refusal("invalid graphic source UUID"))
}
fn geometry(block: &str, form: &str) -> Result<G, EngineError> {
    let from = || point(block, "start");
    let to = || point(block, "end");
    Ok(match form {
        "gr_line" => G::Line {
            from: from()?,
            to: to()?,
        },
        "gr_arc" => G::Arc {
            from: from()?,
            midpoint: point(block, "mid")?,
            to: to()?,
        },
        "gr_rect" => G::Rectangle {
            from: from()?,
            to: to()?,
            filled: filled(block)?,
        },
        "gr_poly" => G::Polygon {
            outer: polygon(block)?,
            holes: vec![],
            filled: filled(block)?,
        },
        "gr_circle" => {
            let center = point(block, "center")?;
            let end = to()?;
            let dx = i128::from(end.x) - i128::from(center.x);
            let dy = i128::from(end.y) - i128::from(center.y);
            let squared = dx
                .checked_mul(dx)
                .and_then(|x| dy.checked_mul(dy).and_then(|y| x.checked_add(y)))
                .and_then(|v| u128::try_from(v).ok())
                .ok_or_else(|| refusal("graphic radius range"))?;
            let radius = squared.isqrt();
            if radius.checked_mul(radius) != Some(squared) {
                return Err(refusal(
                    "non-integer graphic radius cannot be adopted exactly",
                ));
            }
            G::Circle {
                center,
                radius: i64::try_from(radius).map_err(|_| refusal("graphic radius range"))?,
                filled: filled(block)?,
            }
        }
        _ => return Err(refusal(format!("unsupported graphic source {form}"))),
    })
}
fn parse(
    source: &Path,
    contents: &str,
    map: &BTreeMap<ImportKey, ImportMapEntry>,
    board: &Board,
) -> Result<NativeGraphicImport, EngineError> {
    if head(contents.trim()) != "kicad_pcb" {
        return Err(refusal("graphic capture requires a KiCad board document"));
    }
    let mut plan = NativeGraphicImport {
        graphics: vec![],
        identities: vec![],
        dispositions: vec![],
    };
    let mut seen = BTreeSet::new();
    let mut refused = false;
    for block in children(contents.trim())? {
        let form = head(block);
        if !(form.starts_with("gr_") || form == "image") || form == "gr_text" {
            continue;
        }
        let uuid = source_uuid(block).ok();
        let result = (|| -> Result<Option<BoardGraphic>, EngineError> {
            let layer_name = scalar(block, "layer")?;
            if layer_name == "Edge.Cuts" && matches!(form, "gr_line" | "gr_arc") {
                // The existing BoardOutline slot owns these contributions,
                // including historical identifier-less input. This creates no
                // independent graphic and does not allocate a fallback identity.
                return Ok(None);
            }
            let uuid = source_uuid(block)?;
            if !seen.insert(uuid) {
                return Err(refusal("duplicate graphic source UUID"));
            }
            let layers: Vec<_> = board
                .stackup
                .layers
                .iter()
                .filter(|l| l.name == layer_name)
                .collect();
            if layers.len() != 1
                || !matches!(
                    layers[0].layer_type,
                    StackupLayerType::Silkscreen | StackupLayerType::Mechanical
                )
                || board
                    .stackup
                    .layers
                    .iter()
                    .filter(|l| l.id == layers[0].id)
                    .count()
                    != 1
            {
                return Err(refusal(
                    "graphic requires an unambiguous declared non-copper documentation layer",
                ));
            }
            for child in children(block)? {
                let name = head(child);
                let common = matches!(name, "uuid" | "tstamp" | "layer" | "stroke" | "width");
                let shape = match form {
                    "gr_line" => matches!(name, "start" | "end"),
                    "gr_arc" => matches!(name, "start" | "mid" | "end"),
                    "gr_rect" => matches!(name, "start" | "end" | "fill"),
                    "gr_circle" => matches!(name, "center" | "end" | "fill"),
                    "gr_poly" => matches!(name, "pts" | "fill"),
                    _ => false,
                };
                if !common && !shape {
                    return Err(refusal(format!(
                        "unsupported graphic field {}",
                        head(child)
                    )));
                }
            }
            let key = board_graphic_import_key(source, uuid);
            if map.get(&key).is_some_and(|e| {
                !matches!(
                    e.status,
                    ImportMapEntryStatus::Active | ImportMapEntryStatus::MissingInSource
                )
            }) {
                return Err(refusal(
                    "retired/replaced graphic source requires explicit lineage reconciliation",
                ));
            }
            let allocation = allocate_import_identity(map, key.clone());
            let graphic = BoardGraphic {
                uuid: allocation.object_id,
                layer: layers[0].id,
                width: width(block)?,
                geometry: geometry(block, form)?,
                import_key: Some(key.clone()),
            };
            graphic
                .validate()
                .map_err(|e| refusal(format!("invalid exact graphic geometry: {e:?}")))?;
            plan.identities.push(KiCadBoardImportIdentity::new(
                "board_graphic",
                key,
                graphic.uuid,
                uuid,
            ));
            Ok(Some(graphic))
        })();
        let (disposition, explanation) = match result {
            Ok(Some(graphic)) => { plan.graphics.push(graphic); ("supported", "exact graphic source; ImportMap adoption required".into()) },
            Ok(None) => (
                if board.outline.vertices.is_empty() { "pending_outline" } else { "outline_contribution" },
                "existing BoardOutline import route only; no standalone graphic allocation or exact arc CAM claim; reported outline limitations remain pending".into(),
            ),
            Err(error) => { refused = true; ("refused", error.to_string()) },
        };
        plan.dispositions.push(GraphicImportDisposition {
            form: form.into(),
            source_uuid: uuid,
            disposition,
            explanation,
        });
    }
    if refused {
        return Err(refusal(format!(
            "graphic adoption refused before mutation; source={} dispositions={}",
            source.display(),
            serde_json::to_string(&plan.dispositions)?
        )));
    }
    plan.graphics.sort_by_key(|g| g.uuid);
    Ok(plan)
}

#[cfg(test)]
mod tests;
