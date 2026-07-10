pub(super) fn kicad_parse_layer_table(contents: &str) -> std::collections::HashMap<String, i32> {
    let mut map = std::collections::HashMap::new();
    let start = match contents.find("(layers") {
        Some(s) => s,
        None => return map,
    };
    let rest = &contents[start..];
    // Walk until balanced parens close the (layers ...) block.
    let mut depth: i32 = 0;
    let mut block_end = rest.len();
    for (i, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    block_end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let block = &rest[..block_end];
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('(') && !trimmed.starts_with("(layers") {
            let inner = trimmed.trim_start_matches('(').trim_end_matches(')');
            let mut parts = inner.split_whitespace();
            if let Some(id_str) = parts.next()
                && let Ok(id) = id_str.parse::<i32>()
                    && let Some(name) = parts.next() {
                        let name = canonicalize_kicad_layer_name(name.trim_matches('"'));
                        map.insert(name.to_string(), id);
                    }
        }
    }
    map
}

fn canonicalize_kicad_layer_name(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "f.cu" => "F.Cu".to_string(),
        "b.cu" => "B.Cu".to_string(),
        "b.adhes" => "B.Adhes".to_string(),
        "f.adhes" => "F.Adhes".to_string(),
        "b.paste" => "B.Paste".to_string(),
        "f.paste" => "F.Paste".to_string(),
        "b.silks" => "B.SilkS".to_string(),
        "f.silks" => "F.SilkS".to_string(),
        "b.mask" => "B.Mask".to_string(),
        "f.mask" => "F.Mask".to_string(),
        "dwgs.user" => "Dwgs.User".to_string(),
        "cmts.user" => "Cmts.User".to_string(),
        "eco1.user" => "Eco1.User".to_string(),
        "eco2.user" => "Eco2.User".to_string(),
        "edge.cuts" => "Edge.Cuts".to_string(),
        "margin" => "Margin".to_string(),
        "b.crtyd" => "B.CrtYd".to_string(),
        "f.crtyd" => "F.CrtYd".to_string(),
        "b.fab" => "B.Fab".to_string(),
        "f.fab" => "F.Fab".to_string(),
        _ => name.to_string(),
    }
}

pub(super) fn kicad_resolve_layer_id(
    name: &str,
    table: &std::collections::HashMap<String, i32>,
) -> i32 {
    let canonical_name = canonicalize_kicad_layer_name(name);
    if let Some(&id) = table.get(&canonical_name) {
        return id;
    }
    // Hardcoded fallbacks for common layers.
    match canonical_name.as_str() {
        "F.Cu" => 0,
        "B.Cu" => 31,
        "B.SilkS" => 36,
        "F.SilkS" => 37,
        "B.Fab" => 35,
        "F.Fab" => 38,
        "B.CrtYd" => 34,
        "F.CrtYd" => 39,
        "Edge.Cuts" => 44,
        _ => 0,
    }
}

pub(super) fn kicad_render_role(layer_name: &str) -> Option<&'static str> {
    match canonicalize_kicad_layer_name(layer_name).as_str() {
        "F.SilkS" | "B.SilkS" => Some("component_silkscreen"),
        "F.CrtYd" | "B.CrtYd" | "F.Fab" | "B.Fab" => Some("component_mechanical"),
        _ => None,
    }
}

#[derive(Default)]
struct KicadImportTextTrace {
    fp_text_total: usize,
    fp_text_template_skipped: usize,
    fp_text_hidden_skipped: usize,
    fp_text_imported: usize,
    property_total: usize,
    property_metadata_skipped: usize,
    property_empty_skipped: usize,
    property_hidden_skipped: usize,
    property_reference_imported: usize,
    property_value_imported: usize,
    gr_text_total: usize,
    gr_text_hidden_skipped: usize,
    gr_text_imported: usize,
    by_kind: BTreeMap<String, usize>,
    by_layer: BTreeMap<String, usize>,
    samples: Vec<String>,
}

impl KicadImportTextTrace {
    fn record_import(&mut self, kind: &str, layer_name: &str, layer: i32, text: &str) {
        *self.by_kind.entry(kind.to_string()).or_insert(0) += 1;
        *self
            .by_layer
            .entry(format!(
                "{}:{}",
                canonicalize_kicad_layer_name(layer_name),
                layer_id(layer)
            ))
            .or_insert(0) += 1;
        if self.samples.len() < 16 {
            self.samples.push(format!(
                "{}:{}:{}",
                kind,
                canonicalize_kicad_layer_name(layer_name),
                text
            ));
        }
    }

    fn emit(&self, scope: &str, board_texts: usize, geometries: usize, glyph_assets: usize) {
        if !kicad_import_text_trace_enabled() {
            return;
        }
        eprintln!(
            "[datum-import-text] {scope} fp_text total={} imported={} skipped_template={} skipped_hidden={} property total={} ref={} value={} skipped_metadata={} skipped_empty={} skipped_hidden={} gr_text total={} imported={} skipped_hidden={} board_texts={} geometries={} glyph_assets={} by_kind={:?} by_layer={:?} samples={:?}",
            self.fp_text_total,
            self.fp_text_imported,
            self.fp_text_template_skipped,
            self.fp_text_hidden_skipped,
            self.property_total,
            self.property_reference_imported,
            self.property_value_imported,
            self.property_metadata_skipped,
            self.property_empty_skipped,
            self.property_hidden_skipped,
            self.gr_text_total,
            self.gr_text_imported,
            self.gr_text_hidden_skipped,
            board_texts,
            geometries,
            glyph_assets,
            self.by_kind,
            self.by_layer,
            self.samples,
        );
    }
}

pub(super) fn kicad_import_text_trace_enabled() -> bool {
    std::env::var_os("DATUM_TRACE_IMPORT_TEXT").is_some()
        || std::env::var_os("DATUM_TRACE_TIMING").is_some()
}

pub(super) fn trace_protocol_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-protocol] {message}");
    }
}

pub(super) fn kicad_block_hidden(block: &str) -> bool {
    block.contains("(hide yes)")
}

/// Convert mm to nm.
pub(super) fn kicad_mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}

/// Parse a `(form x y ...)` anywhere in a line and return the (x, y) in nm.
pub(super) fn kicad_parse_xy_anywhere(line: &str, form: &str) -> Option<PointNm> {
    let marker = format!("({form} ");
    let start = line.find(&marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(PointNm {
        x: kicad_mm_to_nm(x),
        y: kicad_mm_to_nm(y),
    })
}

/// Parse the stroke/line width from a KiCad block.
/// Handles both old-style `(width 0.12)` and new-style `(stroke (width 0.12) ...)`.
pub(super) fn kicad_parse_width_nm(block: &str) -> i64 {
    // Try `(stroke (width N) ...)` first (KiCad 7+).
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("(stroke ") {
            let rest = &trimmed[pos..];
            if let Some(w_pos) = rest.find("(width ") {
                let after = &rest[w_pos + "(width ".len()..];
                let end = after.find(')').unwrap_or(after.len());
                if let Ok(mm) = after[..end].trim().parse::<f64>() {
                    return kicad_mm_to_nm(mm);
                }
            }
        }
    }
    // Fall back to top-level `(width N)`.
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("(width ") {
            let rest = trimmed.trim_start_matches("(width ").trim_end_matches(')');
            if let Ok(mm) = rest.split_whitespace().next().unwrap_or("").parse::<f64>() {
                return kicad_mm_to_nm(mm);
            }
        }
    }
    120_000 // default 0.12mm
}

/// Parse a `(layer "Name")` from anywhere in a block line.
pub(super) fn kicad_parse_layer_anywhere(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim();
        let start = trimmed.find("(layer ")? + "(layer ".len();
        let rest = &trimmed[start..];
        // Quoted name
        if let Some(inner) = rest.strip_prefix('"') {
            let end = inner.find('"')?;
            Some(canonicalize_kicad_layer_name(&inner[..end]))
        } else {
            let end = rest.find(')')?;
            Some(canonicalize_kicad_layer_name(rest[..end].trim()))
        }
    })
}

/// Parse a `(uuid "...")` from a block.
pub(super) fn kicad_parse_uuid(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim();
        let start = trimmed.find("(uuid ")? + "(uuid ".len();
        let rest = &trimmed[start..];
        if let Some(inner) = rest.strip_prefix('"') {
            let end = inner.find('"')?;
            Some(inner[..end].to_string())
        } else {
            let end = rest.find(')')?;
            Some(rest[..end].trim().to_string())
        }
    })
}

/// Parse `(at x y [rotation])` from a block's first `(at ...)` line.
pub(super) fn kicad_parse_at(block: &str) -> Option<(PointNm, i32)> {
    let line = block.lines().find(|l| l.trim().contains("(at "))?;
    let trimmed = line.trim();
    let start = trimmed.find("(at ")? + "(at ".len();
    let rest = &trimmed[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    let rotation = parts
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|r| r.round() as i32)
        .unwrap_or(0);
    Some((
        PointNm {
            x: kicad_mm_to_nm(x),
            y: kicad_mm_to_nm(y),
        },
        rotation,
    ))
}

/// Parse `(xy x y)` points from a block (used for polygons).
pub(super) fn kicad_parse_xy_points(block: &str) -> Vec<PointNm> {
    let mut points = Vec::new();
    let mut rest = block;
    let marker = "(xy ";
    while let Some(start) = rest.find(marker) {
        let after = &rest[start + marker.len()..];
        let Some(end) = after.find(')') else { break };
        let mut parts = after[..end].split_whitespace();
        if let (Some(x), Some(y)) = (
            parts.next().and_then(|v| v.parse::<f64>().ok()),
            parts.next().and_then(|v| v.parse::<f64>().ok()),
        ) {
            points.push(PointNm {
                x: kicad_mm_to_nm(x),
                y: kicad_mm_to_nm(y),
            });
        }
        rest = &after[end + 1..];
    }
    points
}

/// Extract nested s-expression blocks for a given form within a parent block.
pub(super) fn kicad_nested_blocks(contents: &str, form: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut depth: i32 = 0;
    let prefix = format!("({form}");

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if !capturing
            && trimmed.starts_with(&prefix)
            && matches!(
                trimmed.as_bytes().get(prefix.len()),
                Some(b' ') | Some(b'\t') | Some(b')') | None
            )
        {
            capturing = true;
            current.clear();
            depth = 0;
        }

        if capturing {
            current.push(line.to_string());
            let opens = line.chars().filter(|c| *c == '(').count() as i32;
            let closes = line.chars().filter(|c| *c == ')').count() as i32;
            depth += opens - closes;
            if depth <= 0 {
                blocks.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }
    blocks
}

/// Extract nested s-expression blocks for several forms with one parent scan.
pub(super) fn kicad_nested_blocks_by_form(
    contents: &str,
    forms: &[&str],
) -> BTreeMap<String, Vec<String>> {
    let mut blocks = forms
        .iter()
        .map(|form| ((*form).to_string(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    let prefixes = forms
        .iter()
        .map(|form| ((*form).to_string(), format!("({form}")))
        .collect::<Vec<_>>();
    let mut current = Vec::new();
    let mut capturing_form: Option<String> = None;
    let mut depth: i32 = 0;

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if capturing_form.is_none()
            && let Some((form, _)) = prefixes.iter().find(|(_, prefix)| {
                trimmed.starts_with(prefix)
                    && matches!(
                        trimmed.as_bytes().get(prefix.len()),
                        Some(b' ') | Some(b'\t') | Some(b')') | None
                    )
            }) {
                capturing_form = Some(form.clone());
                current.clear();
                depth = 0;
            }

        if let Some(form) = capturing_form.as_ref() {
            current.push(line.to_string());
            let opens = line.chars().filter(|c| *c == '(').count() as i32;
            let closes = line.chars().filter(|c| *c == ')').count() as i32;
            depth += opens - closes;
            if depth <= 0 {
                blocks
                    .entry(form.clone())
                    .or_default()
                    .push(current.join("\n"));
                current.clear();
                capturing_form = None;
            }
        }
    }

    blocks
}

/// Compute arc center, radius, start_angle_tenths, end_angle_tenths from three
/// points (start, mid, end), all in nm. Returns None for collinear points.
pub(super) fn kicad_arc_from_three_points(
    start: &PointNm,
    mid: &PointNm,
    end: &PointNm,
) -> Option<(PointNm, i64, i32, i32)> {
    let (x1, y1) = (start.x as f64, start.y as f64);
    let (x2, y2) = (mid.x as f64, mid.y as f64);
    let (x3, y3) = (end.x as f64, end.y as f64);
    let d = 2.0 * (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2));
    if d.abs() < f64::EPSILON {
        return None;
    }
    let ux = ((x1 * x1 + y1 * y1) * (y2 - y3)
        + (x2 * x2 + y2 * y2) * (y3 - y1)
        + (x3 * x3 + y3 * y3) * (y1 - y2))
        / d;
    let uy = ((x1 * x1 + y1 * y1) * (x3 - x2)
        + (x2 * x2 + y2 * y2) * (x1 - x3)
        + (x3 * x3 + y3 * y3) * (x2 - x1))
        / d;
    let center = PointNm {
        x: ux.round() as i64,
        y: uy.round() as i64,
    };
    let radius = ((x1 - ux).powi(2) + (y1 - uy).powi(2)).sqrt().round() as i64;
    let start_angle =
        (((y1 - uy).atan2(x1 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    let end_angle =
        (((y3 - uy).atan2(x3 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    Some((center, radius, start_angle, end_angle))
}

/// Parse font size from `(effects (font (size H W) ...))`.
pub(super) fn kicad_parse_font_height_nm(block: &str) -> i64 {
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("(size ") {
            let rest = &trimmed[pos + "(size ".len()..];
            let end = rest.find(')').unwrap_or(rest.len());
            let mut parts = rest[..end].split_whitespace();
            if let Some(h) = parts.next().and_then(|v| v.parse::<f64>().ok()) {
                return kicad_mm_to_nm(h);
            }
        }
    }
    1_000_000 // default 1mm
}

pub(super) fn kicad_parse_font_thickness_nm(block: &str) -> Option<i64> {
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("(thickness ") {
            continue;
        }
        let inner = trimmed
            .trim_start_matches("(thickness ")
            .trim_end_matches(')');
        if let Ok(mm) = inner.trim().parse::<f64>() {
            return Some(kicad_mm_to_nm(mm));
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KicadTextHJustify {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KicadTextVJustify {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KicadTextJustify {
    pub(super) h: KicadTextHJustify,
    pub(super) v: KicadTextVJustify,
    pub(super) mirrored: bool,
    pub(super) keep_upright: bool,
}

impl Default for KicadTextJustify {
    fn default() -> Self {
        Self {
            h: KicadTextHJustify::Center,
            v: KicadTextVJustify::Center,
            mirrored: false,
            keep_upright: false,
        }
    }
}

pub(super) fn kicad_parse_text_justify(block: &str) -> KicadTextJustify {
    let mut justify = KicadTextJustify::default();
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("(justify ") {
            continue;
        }
        if trimmed.contains(" left") {
            justify.h = KicadTextHJustify::Left;
        } else if trimmed.contains(" right") {
            justify.h = KicadTextHJustify::Right;
        }
        if trimmed.contains(" top") {
            justify.v = KicadTextVJustify::Top;
        } else if trimmed.contains(" bottom") {
            justify.v = KicadTextVJustify::Bottom;
        }
        if trimmed.contains(" mirror") {
            justify.mirrored = true;
        }
    }
    justify
}

pub(super) fn kicad_text_attributes(
    anchor_position: PointNm,
    rotation_degrees: i32,
    height_nm: i64,
    stroke_width_nm: Option<i64>,
    justify: KicadTextJustify,
) -> TextAttributes {
    TextAttributes {
        position: eda_engine::ir::geometry::Point {
            x: anchor_position.x,
            y: anchor_position.y,
        },
        rotation_degrees,
        height_nm,
        stroke_width_nm: stroke_width_nm.unwrap_or(default_stroke_width_nm(height_nm)),
        h_align: match justify.h {
            KicadTextHJustify::Left => TextHAlign::Left,
            KicadTextHJustify::Center => TextHAlign::Center,
            KicadTextHJustify::Right => TextHAlign::Right,
        },
        v_align: match justify.v {
            KicadTextVJustify::Top => TextVAlign::Top,
            KicadTextVJustify::Center => TextVAlign::Center,
            KicadTextVJustify::Bottom => TextVAlign::Bottom,
        },
        mirrored: justify.mirrored,
        keep_upright: justify.keep_upright,
        line_spacing_ratio_ppm: 1_350_000,
        render_intent: TextRenderIntent::Manufacturing,
        family: TextFamilyId::default(),
        family_source: eda_engine::text::TextFamilySource::ImplicitDefault,
        style: TextStyleId::default(),
        italic: false,
        bold: false,
        style_class: None,
    }
}

pub(super) fn kicad_text_rotation_degrees(rotation_degrees: i32) -> i32 {
    -rotation_degrees
}
