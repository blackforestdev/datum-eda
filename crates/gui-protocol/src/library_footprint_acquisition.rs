//! Physical acquisition intent stays separate from schematic placement intent.
use super::{LibraryAcquisition, LibraryAcquisitionPreview};
use crate::single_line_editor::SingleLineEditor;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct FootprintAcquisitionPreview {
    pub package: Value,
    pub footprint: Value,
    pub padstacks: Vec<Value>,
}

impl FootprintAcquisitionPreview {
    /// Shared native outlines for authored and source-library footprint previews.
    pub fn outline_paths(raw: &Value) -> Vec<(Vec<crate::PointNm>, i64)> {
        use eda_engine::pool::Primitive;
        let point = |p: eda_engine::ir::geometry::Point| crate::PointNm { x: p.x, y: p.y };
        let mut paths = Vec::new();
        for section in ["mechanical", "fab", "silkscreen"] {
            for raw in raw
                .get(section)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Ok(p) = serde_json::from_value::<Primitive>(raw.clone()) else {
                    continue;
                };
                let (path, width) = match p {
                    Primitive::Line { from, to, width } => (vec![point(from), point(to)], width),
                    Primitive::Rect { min, max, width } => (
                        vec![
                            point(min),
                            crate::PointNm { x: max.x, y: min.y },
                            point(max),
                            crate::PointNm { x: min.x, y: max.y },
                            point(min),
                        ],
                        width,
                    ),
                    Primitive::Circle {
                        center,
                        radius,
                        width,
                    } => (
                        (0..=64)
                            .map(|i| {
                                let a = i as f64 * std::f64::consts::TAU / 64.;
                                crate::PointNm {
                                    x: (center.x as f64 + radius as f64 * a.cos()).round() as i64,
                                    y: (center.y as f64 + radius as f64 * a.sin()).round() as i64,
                                }
                            })
                            .collect(),
                        width,
                    ),
                    _ => continue,
                };
                paths.push((path, width));
            }
        }
        paths
    }
}

impl LibraryAcquisition {
    pub fn use_footprint_source(&mut self) {
        self.source_changed();
        self.source = crate::library_browser::acquisition::AcquisitionSource::KiCadFootprints;
        self.path = "/usr/share/kicad/footprints".into();
        self.path_editor = SingleLineEditor::at_end(&self.path);
    }

    pub fn object_label(&self) -> &'static str {
        if self.native_parts() {
            "Part"
        } else if self.footprints() {
            "footprint"
        } else {
            "symbol"
        }
    }

    pub(super) fn validate_preview_kind(
        &self,
        preview: &LibraryAcquisitionPreview,
    ) -> Result<(), String> {
        if self.native_parts() {
            return if preview.native_part.is_some() && preview.footprint.is_none() {
                Ok(())
            } else {
                Err("Native Part preview belongs to a different acquisition workflow".into())
            };
        }
        if preview.native_part.is_some() {
            return Err(
                "Native Part preview cannot be acquired as a bare symbol or footprint".into(),
            );
        }
        if self.footprints() != preview.footprint.is_some()
            || (self.footprints() && (self.bundled() || self.placement_pane.is_some()))
        {
            return Err("Library preview belongs to a different acquisition workflow".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_source_cannot_be_acquired_as_a_schematic_symbol() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.use_footprint_source();
        acquisition.selected = Some("R_0603".into());
        acquisition.selected_source = Some("/library/R_0603.kicad_mod".into());
        acquisition.preview = Some(LibraryAcquisitionPreview {
            native_part: None,
            name: "R_0603".into(),
            source_digest: "hash".into(),
            source_path: "/library/R_0603.kicad_mod".into(),
            unit: Value::Null,
            symbol: Value::Null,
            footprint: Some(FootprintAcquisitionPreview {
                package: Value::Null,
                footprint: Value::Null,
                padstacks: Vec::new(),
            }),
            notices: Vec::new(),
        });
        let request = acquisition.request().unwrap();
        assert_eq!(request.source_format, "kicad_footprint");
        assert_eq!(request.symbol_name, "R_0603");
        acquisition.placement_pane = Some(crate::PaneId(1));
        assert!(acquisition.request().is_err());
        acquisition.placement_pane = None;
        acquisition.source = crate::library_browser::acquisition::AcquisitionSource::KiCadSymbols;
        assert!(acquisition.request().is_err());
    }
}
