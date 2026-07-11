//! Exhaustive renderer-side assignment of authored primitives to UVT weight classes.
//!
//! This is deliberately a closed enum rather than a string map: adding a governed
//! primitive forces an explicit class decision. Class B/C strokes are ultimately
//! resolved against the live projection by the retained-stroke GPU lane; until
//! that lane exists, callers must retain nominal nm widths and must not bake a
//! width from the fit/reference projection.

use datum_gui_viewport::WeightClass;

pub(crate) const SILK_LINE_NM: i64 = 150_000;
pub(crate) const EDGE_CUT_NM: i64 = 100_000;
pub(crate) const SCHEMATIC_WIRE_NM: i64 = 152_400;
pub(crate) const SCHEMATIC_BUS_NM: i64 = 304_800;
pub(crate) const SYMBOL_OUTLINE_NM: i64 = 127_000;
pub(crate) const PIN_LINE_NM: i64 = 101_600;
pub(crate) const TERMINAL_DOT_NM: i64 = 300_000;
pub(crate) const JUNCTION_DOT_NM: i64 = 400_000;

#[allow(dead_code)] // Closed inventory: surface slices adopt entries incrementally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthoredStrokePrimitive {
    CopperTrace { width_nm: i64 },
    CopperZoneOutline,
    BoardSilkLine,
    EdgeCut,
    SchematicWire,
    SchematicBus,
    BusEntry,
    SymbolBodyOutline,
    PinLine,
    PinTerminalDot,
    JunctionDot,
    NetLabelBorder,
    NoConnectMarker,
    PowerSymbolGlyph,
}

impl AuthoredStrokePrimitive {
    pub(crate) fn weight(self) -> WeightClass {
        let authored = |nominal_nm, min_px| WeightClass::AuthoredConstantNm { nominal_nm, min_px };
        match self {
            Self::CopperTrace { width_nm } => WeightClass::WorldWidthWithMinClamp {
                nominal_nm: width_nm.max(1),
                min_px: 1.0,
            },
            Self::CopperZoneOutline => authored(EDGE_CUT_NM, 1.0),
            Self::BoardSilkLine => authored(SILK_LINE_NM, 1.0),
            Self::EdgeCut => authored(EDGE_CUT_NM, 1.0),
            Self::SchematicWire | Self::BusEntry | Self::NoConnectMarker => {
                authored(SCHEMATIC_WIRE_NM, 1.0)
            }
            Self::SchematicBus => authored(SCHEMATIC_BUS_NM, 1.5),
            Self::SymbolBodyOutline | Self::NetLabelBorder | Self::PowerSymbolGlyph => {
                authored(SYMBOL_OUTLINE_NM, 1.0)
            }
            Self::PinLine => authored(PIN_LINE_NM, 1.0),
            Self::PinTerminalDot => authored(TERMINAL_DOT_NM, 3.0),
            Self::JunctionDot => authored(JUNCTION_DOT_NM, 3.0),
        }
    }

    pub(crate) fn nominal_nm(self) -> i64 {
        match self.weight() {
            WeightClass::WorldWidthWithMinClamp { nominal_nm, .. }
            | WeightClass::AuthoredConstantNm { nominal_nm, .. } => nominal_nm,
            WeightClass::ScreenConstant(_) => unreachable!("authored primitives are class B/C"),
        }
    }
}

pub(crate) fn board_graphic_nominal_nm(layer_id: &str, imported_width_nm: Option<i64>) -> i64 {
    imported_width_nm
        .unwrap_or_else(|| {
            if layer_id.eq_ignore_ascii_case("Edge.Cuts")
                || layer_id.eq_ignore_ascii_case("edge_cuts")
            {
                EDGE_CUT_NM
            } else {
                SILK_LINE_NM
            }
        })
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_governed_authored_stroke_has_exactly_one_non_chrome_class() {
        let governed = [
            AuthoredStrokePrimitive::CopperTrace { width_nm: 203_200 },
            AuthoredStrokePrimitive::CopperZoneOutline,
            AuthoredStrokePrimitive::BoardSilkLine,
            AuthoredStrokePrimitive::EdgeCut,
            AuthoredStrokePrimitive::SchematicWire,
            AuthoredStrokePrimitive::SchematicBus,
            AuthoredStrokePrimitive::BusEntry,
            AuthoredStrokePrimitive::SymbolBodyOutline,
            AuthoredStrokePrimitive::PinLine,
            AuthoredStrokePrimitive::PinTerminalDot,
            AuthoredStrokePrimitive::JunctionDot,
            AuthoredStrokePrimitive::NetLabelBorder,
            AuthoredStrokePrimitive::NoConnectMarker,
            AuthoredStrokePrimitive::PowerSymbolGlyph,
        ];
        for primitive in governed {
            assert!(!matches!(
                primitive.weight(),
                WeightClass::ScreenConstant(_)
            ));
        }
    }

    #[test]
    fn authored_widths_scale_with_live_zoom_and_floor_in_device_pixels() {
        let wire = AuthoredStrokePrimitive::SchematicWire.weight();
        assert_eq!(wire.resolve_px(1.0e-9), 1.0);
        assert!((wire.resolve_px(1.0e-5) - 1.524).abs() < 0.001);
        assert!((wire.resolve_px(1.0e-4) - 15.24).abs() < 0.001);

        let bus = AuthoredStrokePrimitive::SchematicBus.weight();
        assert_eq!(bus.resolve_px(1.0e-9), 1.5);
        let terminal = AuthoredStrokePrimitive::PinTerminalDot.weight();
        assert_eq!(terminal.resolve_px(1.0e-9), 3.0);
    }

    #[test]
    fn per_object_copper_preserves_authored_width_and_provenance() {
        assert_eq!(
            AuthoredStrokePrimitive::CopperTrace { width_nm: 254_000 }.weight(),
            WeightClass::WorldWidthWithMinClamp {
                nominal_nm: 254_000,
                min_px: 1.0,
            }
        );
    }

    #[test]
    fn board_graphic_defaults_are_class_c_and_imported_width_wins() {
        assert_eq!(board_graphic_nominal_nm("F.SilkS", None), SILK_LINE_NM);
        assert_eq!(board_graphic_nominal_nm("Edge.Cuts", None), EDGE_CUT_NM);
        assert_eq!(board_graphic_nominal_nm("F.SilkS", Some(88_000)), 88_000);
    }
}
