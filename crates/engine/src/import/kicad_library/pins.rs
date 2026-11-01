//! Logical pin identity remains distinct from the schematic pin designator.
use super::geometry::{angle, length, point};
use super::{Node, Result, refusal};
use crate::ir::ids::{import_uuid, namespace_kicad};
use crate::pool::{
    Pin, PinDirection, SymbolPinAnchor, SymbolPinDecoration, SymbolPinOrientation, SymbolPinStyle,
};

pub(super) fn pin(
    node: &Node,
    identity: &str,
    warnings: &mut Vec<String>,
) -> Result<(Pin, SymbolPinAnchor)> {
    let electrical = node.required_atom(1)?;
    let direction = match electrical {
        "input" => PinDirection::Input,
        "output" => PinDirection::Output,
        "bidirectional" => PinDirection::Bidirectional,
        "tri_state" => PinDirection::TriState,
        "passive" => PinDirection::Passive,
        "power_in" => PinDirection::PowerIn,
        "power_out" => PinDirection::PowerOut,
        "open_collector" => PinDirection::OpenCollector,
        "open_emitter" => PinDirection::OpenEmitter,
        "no_connect" => PinDirection::NoConnect,
        _ => {
            return Err(refusal(format!(
                "unsupported electrical pin type {electrical}"
            )));
        }
    };
    if node.required_atom(2)? != "line" {
        return Err(refusal(
            "decorated pins are not supported by native symbol rendering",
        ));
    }
    if node.flag("hide") {
        if matches!(direction, PinDirection::PowerIn | PinDirection::PowerOut) {
            return Err(refusal(
                "hidden power pins require unsupported implicit net semantics",
            ));
        }
        warnings.push("Source hidden pins remain explicit native pins; source pin visibility is not represented.".into());
    }
    if node.child("alternate").is_some() {
        return Err(refusal("alternate pin functions are unsupported"));
    }
    let name = node.required_child("name")?.required_atom(1)?.to_owned();
    let number = node.required_child("number")?.required_atom(1)?.to_owned();
    if number.is_empty() {
        return Err(refusal("pin number must not be empty"));
    }
    let uuid = import_uuid(&namespace_kicad(), &format!("{identity}:pin:{number}"));
    let at = node.required_child("at")?;
    let orientation = match angle(at.required_atom(3)?)?.rem_euclid(360) {
        0 => SymbolPinOrientation::Right,
        90 => SymbolPinOrientation::Up,
        180 => SymbolPinOrientation::Left,
        270 => SymbolPinOrientation::Down,
        _ => return Err(refusal("non-cardinal pin direction is unsupported")),
    };
    let length_nm = length(node.required_child("length")?.required_atom(1)?)?;
    if length_nm < 0 {
        return Err(refusal("negative pin length"));
    }
    let position = point(at)?;
    let end_valid = match orientation {
        SymbolPinOrientation::Right => position.x.checked_add(length_nm),
        SymbolPinOrientation::Left => position.x.checked_sub(length_nm),
        SymbolPinOrientation::Up => position.y.checked_sub(length_nm),
        SymbolPinOrientation::Down => position.y.checked_add(length_nm),
    };
    if end_valid.is_none() {
        return Err(refusal("pin stub endpoint overflow"));
    }
    Ok((
        Pin {
            uuid,
            name,
            direction,
            swap_group: 0,
            alternates: Vec::new(),
        },
        SymbolPinAnchor {
            pin: uuid,
            number: Some(number),
            position,
            style: SymbolPinStyle {
                orientation,
                length_nm: Some(length_nm),
                decoration: SymbolPinDecoration::None,
            },
        },
    ))
}
