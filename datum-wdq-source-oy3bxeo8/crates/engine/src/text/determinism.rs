use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ir::serialization::to_json_deterministic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlattenedOutlinePoint {
    pub x_nm: i64,
    pub y_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlattenedOutlineContour {
    pub points: Vec<FlattenedOutlinePoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlattenedGlyphFixture {
    pub codepoint: u32,
    pub contours: Vec<FlattenedOutlineContour>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineDeterminismFixture {
    pub family: String,
    pub style: String,
    pub tolerance_nm: i64,
    pub glyphs: Vec<FlattenedGlyphFixture>,
}

pub fn canonical_outline_fixture_json(
    fixture: &OutlineDeterminismFixture,
) -> Result<String, serde_json::Error> {
    to_json_deterministic(fixture)
}

pub fn vendored_font_asset_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join(file_name)
}

pub fn golden_text_fixture_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join("golden")
        .join("text")
        .join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_fixture_serializes_canonically() {
        let fixture = OutlineDeterminismFixture {
            family: "inter".to_string(),
            style: "regular".to_string(),
            tolerance_nm: 50_000,
            glyphs: vec![FlattenedGlyphFixture {
                codepoint: 'A' as u32,
                contours: vec![FlattenedOutlineContour {
                    points: vec![
                        FlattenedOutlinePoint {
                            x_nm: 1_000,
                            y_nm: 2_000,
                        },
                        FlattenedOutlinePoint {
                            x_nm: 3_000,
                            y_nm: 4_000,
                        },
                    ],
                }],
            }],
        };
        let json = canonical_outline_fixture_json(&fixture).expect("fixture should serialize");
        assert_eq!(
            json,
            r#"{"family":"inter","glyphs":[{"codepoint":65,"contours":[{"points":[{"x_nm":1000,"y_nm":2000},{"x_nm":3000,"y_nm":4000}]}]}],"style":"regular","tolerance_nm":50000}"#
        );
    }

    #[test]
    fn phase_two_font_and_golden_locations_exist() {
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/README.md")
                .exists()
        );
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/FONT_PROVENANCE.md")
                .exists()
        );
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata/golden/text/README.md")
                .exists()
        );
    }

    #[test]
    fn outline_vendored_font_flatten_matches_golden_fixture() {
        let font_path = vendored_font_asset_path("dev/DejaVuSans.ttf");
        let golden_path = golden_text_fixture_path("dejavu_sans_regular_outline_fixture.json");
        assert!(
            font_path.exists(),
            "vendored outline font missing: {}",
            font_path.display()
        );
        let bytes = std::fs::read(&font_path).expect("vendored outline font should read");
        let fixture = OutlineDeterminismFixture {
            family: "dev_dejavu_sans".to_string(),
            style: "regular".to_string(),
            tolerance_nm: 50_000,
            glyphs: vec![
                crate::text::flatten_glyph_from_font_bytes(&bytes, 'A', 1_000_000, 50_000)
                    .expect("A should flatten"),
                crate::text::flatten_glyph_from_font_bytes(&bytes, 'R', 1_000_000, 50_000)
                    .expect("R should flatten"),
                crate::text::flatten_glyph_from_font_bytes(&bytes, '8', 1_000_000, 50_000)
                    .expect("8 should flatten"),
            ],
        };
        let actual =
            canonical_outline_fixture_json(&fixture).expect("fixture should serialize canonically");
        if std::env::var_os("UPDATE_GOLDENS").is_some() {
            if let Some(parent) = golden_path.parent() {
                std::fs::create_dir_all(parent).expect("golden dir should create");
            }
            std::fs::write(&golden_path, format!("{actual}\n")).expect("golden should write");
            return;
        }
        assert!(
            golden_path.exists(),
            "outline golden fixture missing: {}",
            golden_path.display()
        );
        let expected =
            std::fs::read_to_string(&golden_path).expect("outline determinism golden should read");
        assert_eq!(actual.trim(), expected.trim());
    }
}
