//! Shared, source-declared pin stub geometry for library and schematic displays.
use crate::PointNm;
use eda_engine::pool::{SymbolPinAnchor, SymbolPinOrientation};

/// A library preview never invents an absent direction or length.
pub fn symbol_pin_stub(anchor: &serde_json::Value) -> Option<(PointNm, PointNm)> {
    anchor.pointer("/style/orientation")?.as_str()?;
    anchor.pointer("/style/length_nm")?.as_i64()?;
    pin_stub_points(&serde_json::from_value(anchor.clone()).ok()?)
}

pub(crate) fn pin_stub_points(anchor: &SymbolPinAnchor) -> Option<(PointNm, PointNm)> {
    let from = PointNm {
        x: anchor.position.x,
        y: anchor.position.y,
    };
    let length = anchor.style.length_nm?;
    if length < 0 {
        return None;
    }
    let to = match anchor.style.orientation {
        SymbolPinOrientation::Left => PointNm {
            x: from.x.checked_sub(length)?,
            y: from.y,
        },
        SymbolPinOrientation::Right => PointNm {
            x: from.x.checked_add(length)?,
            y: from.y,
        },
        SymbolPinOrientation::Up => PointNm {
            x: from.x,
            y: from.y.checked_sub(length)?,
        },
        SymbolPinOrientation::Down => PointNm {
            x: from.x,
            y: from.y.checked_add(length)?,
        },
    };
    Some((from, to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn declared_symbol_pin_stub_has_exact_endpoints_without_defaults_or_overflow() {
        for (orientation, end) in [
            ("Left", (7, 20)),
            ("Right", (13, 20)),
            ("Up", (10, 17)),
            ("Down", (10, 23)),
        ] {
            let raw = json!({"pin":"70000000-0000-4000-8000-000000000003",
                "position":{"x":10,"y":20},
                "style":{"orientation":orientation,"length_nm":3,"decoration":"none"}});
            assert_eq!(
                symbol_pin_stub(&raw),
                Some((PointNm { x: 10, y: 20 }, PointNm { x: end.0, y: end.1 }))
            );
        }
        let mut raw = json!({"pin":"70000000-0000-4000-8000-000000000003",
            "position":{"x":i64::MAX,"y":0},"style":{"orientation":"Right","length_nm":1}});
        assert!(symbol_pin_stub(&raw).is_none());
        raw["style"]["length_nm"] = json!(0);
        let (from, to) = symbol_pin_stub(&raw).unwrap();
        assert_eq!(from, to);
        raw["style"].as_object_mut().unwrap().remove("length_nm");
        assert!(symbol_pin_stub(&raw).is_none());
    }
}
