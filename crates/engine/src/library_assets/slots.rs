//! Schema-owned forward asset references; subobject and owner links stay contextual.
use serde_json::Value;
use uuid::Uuid;

use super::{AssetKind, invalid};
use crate::error::EngineError;

pub(crate) struct ReferenceSlot {
    pub(crate) path: Vec<String>,
    pub(crate) target_id: Uuid,
    pub(crate) kind: AssetKind,
}

pub(crate) fn slots(kind: AssetKind, value: &Value) -> Result<Vec<ReferenceSlot>, EngineError> {
    let mut slots = Vec::new();
    let mut field = |path: Vec<String>, target, optional| -> Result<(), EngineError> {
        let mut item = value;
        for token in &path {
            item = item.get(token).unwrap_or(&Value::Null);
        }
        if item.is_null() && optional {
            return Ok(());
        }
        let id = item
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or_else(|| invalid(format!("invalid asset reference at {path:?}")))?;
        slots.push(ReferenceSlot {
            path,
            target_id: id,
            kind: target,
        });
        Ok(())
    };
    match kind {
        AssetKind::Symbol => field(vec!["unit".into()], AssetKind::Unit, false)?,
        AssetKind::Entity => {
            for id in object_keys(value, "gates")? {
                for (name, target) in [("unit", AssetKind::Unit), ("symbol", AssetKind::Symbol)] {
                    field(vec!["gates".into(), id.clone(), name.into()], target, false)?;
                }
            }
        }
        AssetKind::Part => {
            for (name, target, optional) in [
                ("entity", AssetKind::Entity, false),
                ("package", AssetKind::Package, false),
                ("default_footprint", AssetKind::Footprint, true),
                ("default_pin_pad_map", AssetKind::PinPadMap, true),
                ("base", AssetKind::Part, true),
            ] {
                field(vec![name.into()], target, optional)?;
            }
        }
        AssetKind::Footprint | AssetKind::Package => {
            if kind == AssetKind::Footprint {
                field(vec!["package".into()], AssetKind::Package, false)?;
            }
            if value.get("pads").is_some() {
                for id in object_keys(value, "pads")? {
                    field(
                        vec!["pads".into(), id, "padstack".into()],
                        AssetKind::Padstack,
                        false,
                    )?;
                }
            }
        }
        AssetKind::PinPadMap => field(vec!["footprint".into()], AssetKind::Footprint, true)?,
        AssetKind::Unit | AssetKind::Padstack | AssetKind::ModelAttachment => {}
    }
    slots.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(slots)
}

fn object_keys(value: &Value, key: &str) -> Result<Vec<String>, EngineError> {
    let map = value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| invalid(format!("{key} must be an object")))?;
    Ok(map.keys().cloned().collect())
}
