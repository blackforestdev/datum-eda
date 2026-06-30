use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, PartialEq)]
pub struct FixtureManifest {
    pub name: String,
    pub lane: String,
    pub suite: String,
    pub fixture_format_version: i64,
    pub project_path: PathBuf,
    pub project_kind: String,
    pub viewport: VisualViewport,
    pub ui_scale_factors: Vec<f32>,
    pub golden: VisualGolden,
    pub blank_check: VisualBlankCheck,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualViewport {
    pub width_px: u32,
    pub height_px: u32,
    pub center_mm: [f64; 2],
    pub zoom_mm_per_px: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualGolden {
    pub filename: String,
    pub diff_policy: String,
    pub diff_tolerance_per_pixel: u8,
    pub diff_tolerance_total_px_pct: f64,
    pub ssim_threshold: f64,
    pub mask_filename: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualBlankCheck {
    pub expect_non_blank_pct: f64,
}

impl FixtureManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read visual fixture manifest {}", path.display()))?;
        Self::parse(&text)
            .with_context(|| format!("parse visual fixture manifest {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let doc = SimpleToml::parse(text)?;
        let manifest = Self {
            name: required_string(&doc, &["fixture", "name"])?,
            lane: required_string(&doc, &["fixture", "lane"])?,
            suite: required_string(&doc, &["fixture", "suite"])?,
            fixture_format_version: required_i64(&doc, &["fixture", "fixture_format_version"])?,
            project_path: PathBuf::from(required_string(&doc, &["input", "project_path"])?),
            project_kind: required_string(&doc, &["input", "project_kind"])?,
            viewport: VisualViewport {
                width_px: required_i64(&doc, &["viewport", "width_px"])?
                    .try_into()
                    .context("viewport.width_px must fit u32")?,
                height_px: required_i64(&doc, &["viewport", "height_px"])?
                    .try_into()
                    .context("viewport.height_px must fit u32")?,
                center_mm: required_f64_array_2(&doc, &["viewport", "center_mm"])?,
                zoom_mm_per_px: required_f64(&doc, &["viewport", "zoom_mm_per_px"])?,
            },
            ui_scale_factors: optional_f64_array(&doc, &["viewport", "ui_scale_factors"])?
                .unwrap_or_else(|| vec![1.0])
                .into_iter()
                .map(|value| value as f32)
                .collect(),
            golden: VisualGolden {
                filename: required_string(&doc, &["golden", "filename"])?,
                diff_policy: required_string(&doc, &["golden", "diff_policy"])?,
                diff_tolerance_per_pixel: required_i64(
                    &doc,
                    &["golden", "diff_tolerance_per_pixel"],
                )?
                .try_into()
                .context("golden.diff_tolerance_per_pixel must fit u8")?,
                diff_tolerance_total_px_pct: required_f64(
                    &doc,
                    &["golden", "diff_tolerance_total_px_pct"],
                )?,
                ssim_threshold: required_f64(&doc, &["golden", "ssim_threshold"])?,
                mask_filename: required_string(&doc, &["golden", "mask_filename"])?,
            },
            blank_check: VisualBlankCheck {
                expect_non_blank_pct: required_f64(&doc, &["blank_check", "expect_non_blank_pct"])?,
            },
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if self.fixture_format_version != 1 {
            bail!(
                "unsupported visual fixture format version {}",
                self.fixture_format_version
            );
        }
        if self.lane != "A" {
            bail!("Layer A harness only accepts lane = \"A\"");
        }
        if self.project_kind != "datum-native" {
            bail!("Layer A Phase 1 only accepts project_kind = \"datum-native\"");
        }
        if self.viewport.width_px == 0 || self.viewport.height_px == 0 {
            bail!("viewport dimensions must be non-zero");
        }
        if self.viewport.zoom_mm_per_px <= 0.0 {
            bail!("viewport.zoom_mm_per_px must be positive");
        }
        if self.ui_scale_factors.is_empty() {
            bail!("viewport.ui_scale_factors must not be empty");
        }
        for scale_factor in &self.ui_scale_factors {
            if !scale_factor.is_finite() || *scale_factor <= 0.0 {
                bail!("viewport.ui_scale_factors must contain only positive finite numbers");
            }
        }
        match self.golden.diff_policy.as_str() {
            "exact" => {
                if self.golden.diff_tolerance_per_pixel != 0
                    || self.golden.diff_tolerance_total_px_pct != 0.0
                    || self.golden.ssim_threshold != 0.0
                    || !self.golden.mask_filename.is_empty()
                {
                    bail!("exact diff policy must not carry tolerance, ssim, or mask parameters");
                }
            }
            "tolerance" => {
                if self.golden.ssim_threshold != 0.0 || !self.golden.mask_filename.is_empty() {
                    bail!("tolerance diff policy must not carry ssim or mask parameters");
                }
            }
            "ssim" => {
                if !(0.0..=1.0).contains(&self.golden.ssim_threshold) {
                    bail!("ssim threshold must be within 0.0..=1.0");
                }
            }
            "masked" => {
                if self.golden.mask_filename.is_empty() {
                    bail!("masked diff policy requires mask_filename");
                }
            }
            other => bail!("unsupported visual diff policy {other:?}"),
        }
        if self.blank_check.expect_non_blank_pct < 0.0 {
            bail!("blank_check.expect_non_blank_pct must be non-negative");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SimpleTomlValue {
    String(String),
    Integer(i64),
    Float(f64),
    FloatArray(Vec<f64>),
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SimpleToml {
    values: std::collections::BTreeMap<String, SimpleTomlValue>,
}

impl SimpleToml {
    fn parse(text: &str) -> Result<Self> {
        let mut section = String::new();
        let mut values = std::collections::BTreeMap::new();
        for (line_index, raw_line) in text.lines().enumerate() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_string();
                if section.is_empty() {
                    bail!("empty TOML section at line {}", line_index + 1);
                }
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                bail!("expected key/value at line {}", line_index + 1);
            };
            let key = key.trim();
            if key.is_empty() {
                bail!("empty TOML key at line {}", line_index + 1);
            }
            if section.is_empty() {
                bail!(
                    "manifest keys must be inside sections; line {}",
                    line_index + 1
                );
            }
            let path = format!("{section}.{key}");
            values.insert(path, parse_value(value.trim(), line_index + 1)?);
        }
        Ok(Self { values })
    }

    fn get(&self, path: &[&str]) -> Option<&SimpleTomlValue> {
        self.values.get(&path.join("."))
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_value(value: &str, line_number: usize) -> Result<SimpleTomlValue> {
    if let Some(stripped) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Ok(SimpleTomlValue::String(stripped.to_string()));
    }
    if value.starts_with('[') && value.ends_with(']') {
        let inner = value[1..value.len() - 1].trim();
        if inner.is_empty() {
            return Ok(SimpleTomlValue::FloatArray(Vec::new()));
        }
        let mut values = Vec::new();
        for part in inner.split(',') {
            values.push(
                part.trim().parse::<f64>().with_context(|| {
                    format!("array item at line {line_number} must be a number")
                })?,
            );
        }
        return Ok(SimpleTomlValue::FloatArray(values));
    }
    if value.contains('.') {
        return value
            .parse::<f64>()
            .map(SimpleTomlValue::Float)
            .with_context(|| format!("value at line {line_number} must be a float"));
    }
    value
        .parse::<i64>()
        .map(SimpleTomlValue::Integer)
        .with_context(|| {
            format!("value at line {line_number} must be an integer, float, array, or string")
        })
}

fn required_value<'a>(doc: &'a SimpleToml, path: &[&str]) -> Result<&'a SimpleTomlValue> {
    doc.get(path)
        .with_context(|| format!("missing manifest key {}", path.join(".")))
}

fn required_string(doc: &SimpleToml, path: &[&str]) -> Result<String> {
    match required_value(doc, path)? {
        SimpleTomlValue::String(value) => Ok(value.clone()),
        _ => bail!("manifest key {} must be a string", path.join(".")),
    }
}

fn required_i64(doc: &SimpleToml, path: &[&str]) -> Result<i64> {
    match required_value(doc, path)? {
        SimpleTomlValue::Integer(value) => Ok(*value),
        _ => bail!("manifest key {} must be an integer", path.join(".")),
    }
}

fn required_f64(doc: &SimpleToml, path: &[&str]) -> Result<f64> {
    match required_value(doc, path)? {
        SimpleTomlValue::Integer(value) => Ok(*value as f64),
        SimpleTomlValue::Float(value) => Ok(*value),
        _ => bail!("manifest key {} must be a number", path.join(".")),
    }
}

fn required_f64_array_2(doc: &SimpleToml, path: &[&str]) -> Result<[f64; 2]> {
    match required_value(doc, path)? {
        SimpleTomlValue::FloatArray(values) if values.len() == 2 => Ok([values[0], values[1]]),
        SimpleTomlValue::FloatArray(_) => {
            bail!(
                "manifest key {} must have exactly 2 elements",
                path.join(".")
            )
        }
        _ => bail!("manifest key {} must be an array", path.join(".")),
    }
}

fn optional_f64_array(doc: &SimpleToml, path: &[&str]) -> Result<Option<Vec<f64>>> {
    let Some(value) = doc.get(path) else {
        return Ok(None);
    };
    match value {
        SimpleTomlValue::FloatArray(values) => Ok(Some(values.clone())),
        _ => bail!("manifest key {} must be an array", path.join(".")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_layer_a_manifest() {
        let manifest = FixtureManifest::parse(
            r#"
            [fixture]
            name = "text-density-repro"
            lane = "A"
            suite = "board-text"
            fixture_format_version = 1

            [input]
            project_path = "crates/engine/testdata/golden/text/native/text-density-repro"
            project_kind = "datum-native"

            [viewport]
            width_px = 1024
            height_px = 768
            center_mm = [100.0, 75.0]
            zoom_mm_per_px = 0.12

            [golden]
            filename = "text-density-repro.golden.png"
            diff_policy = "exact"
            diff_tolerance_per_pixel = 0
            diff_tolerance_total_px_pct = 0.0
            ssim_threshold = 0.0
            mask_filename = ""

            [blank_check]
            expect_non_blank_pct = 1.0
            "#,
        )
        .expect("manifest should parse");
        assert_eq!(manifest.name, "text-density-repro");
        assert_eq!(manifest.viewport.width_px, 1024);
        assert_eq!(manifest.viewport.center_mm, [100.0, 75.0]);
        assert_eq!(manifest.ui_scale_factors, vec![1.0]);
    }

    #[test]
    fn parses_explicit_ui_scale_factors() {
        let manifest = FixtureManifest::parse(
            r#"
            [fixture]
            name = "scale-repro"
            lane = "A"
            suite = "layout"
            fixture_format_version = 1

            [input]
            project_path = "crates/engine/testdata/golden/text/native/text-density-repro"
            project_kind = "datum-native"

            [viewport]
            width_px = 1024
            height_px = 768
            center_mm = [100.0, 75.0]
            zoom_mm_per_px = 0.12
            ui_scale_factors = [1.0, 1.25, 1.5, 2.0]

            [golden]
            filename = "scale-repro.golden.png"
            diff_policy = "exact"
            diff_tolerance_per_pixel = 0
            diff_tolerance_total_px_pct = 0.0
            ssim_threshold = 0.0
            mask_filename = ""

            [blank_check]
            expect_non_blank_pct = 1.0
            "#,
        )
        .expect("manifest should parse explicit UI scales");

        assert_eq!(manifest.ui_scale_factors, vec![1.0, 1.25, 1.5, 2.0]);
    }

    #[test]
    fn rejects_exact_policy_with_tolerance() {
        let err = FixtureManifest::parse(
            r#"
            [fixture]
            name = "bad"
            lane = "A"
            suite = "board-text"
            fixture_format_version = 1

            [input]
            project_path = "project"
            project_kind = "datum-native"

            [viewport]
            width_px = 1
            height_px = 1
            center_mm = [0.0, 0.0]
            zoom_mm_per_px = 1.0

            [golden]
            filename = "bad.golden.png"
            diff_policy = "exact"
            diff_tolerance_per_pixel = 1
            diff_tolerance_total_px_pct = 0.0
            ssim_threshold = 0.0
            mask_filename = ""

            [blank_check]
            expect_non_blank_pct = 1.0
            "#,
        )
        .expect_err("exact policy with tolerance should fail");
        assert!(format!("{err:#}").contains("exact diff policy"));
    }

    #[test]
    fn parses_checked_in_text_fixture_manifests() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("golden")
            .join("board");
        for fixture in [
            "text-intent-repro",
            "text-fidelity-repro",
            "text-transform-repro",
            "text-density-repro",
        ] {
            let path = root.join(format!("{fixture}.fixture.toml"));
            let manifest = FixtureManifest::load(&path).expect("fixture manifest should parse");
            assert_eq!(manifest.name, fixture);
            assert!(manifest.project_path.ends_with(fixture));
        }
    }
}
