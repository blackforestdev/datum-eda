//! Clearance for one native pin's labels against its displayed symbol ink.
use super::*;
use eda_engine::schematic::SymbolPin;

#[derive(Clone, Copy)]
struct Bounds {
    min: [i128; 2],
    max: [i128; 2],
}

impl Bounds {
    fn point(point: PointNm) -> [i128; 2] {
        [point.x.into(), point.y.into()]
    }

    fn stroke(a: PointNm, b: PointNm, radius: i128) -> Self {
        let (a, b) = (Self::point(a), Self::point(b));
        Self {
            min: std::array::from_fn(|i| a[i].min(b[i]) - radius),
            max: std::array::from_fn(|i| a[i].max(b[i]) + radius),
        }
    }

    fn include(&mut self, other: Self) {
        for i in 0..2 {
            self.min[i] = self.min[i].min(other.min[i]);
            self.max[i] = self.max[i].max(other.max[i]);
        }
    }

    fn overlaps(self, other: Self) -> bool {
        (0..2).all(|i| self.min[i] < other.max[i] && other.min[i] < self.max[i])
    }

    fn shift(&mut self, axis: usize, by: i128) {
        self.min[axis] += by;
        self.max[axis] += by;
    }
}

fn radius(width: i64) -> i128 {
    (i128::from(width).abs() + 1) / 2
}

fn ink_bounds(geometry: &BoardTextGeometryPrimitive) -> Option<Bounds> {
    let mut bounds: Option<Bounds> = None;
    let mut include = |next| match &mut bounds {
        Some(bounds) => bounds.include(next),
        None => bounds = Some(next),
    };
    if let Some(points) = crate::selection_geometry_catalog::layout_polygon(geometry) {
        for point in points {
            let floor = point.map(|v| v.numerator.div_euclid(v.denominator));
            let ceil = std::array::from_fn(|i| {
                floor[i] + i128::from(point[i].numerator.rem_euclid(point[i].denominator) != 0)
            });
            include(Bounds {
                min: floor,
                max: ceil,
            });
        }
    }
    for fill in &geometry.fills {
        for &point in fill.outer.iter().chain(fill.holes.iter().flatten()) {
            include(Bounds::stroke(point, point, 0));
        }
    }
    for stroke in &geometry.strokes {
        include(Bounds::stroke(
            stroke.from,
            stroke.to,
            radius(stroke.width_nm),
        ));
    }
    bounds
}

fn clearance(bounds: &mut Bounds, obstacles: &[Bounds], axis: usize, sign: i128) -> i128 {
    let mut total = 0;
    loop {
        let shift = obstacles
            .iter()
            .filter(|&&other| bounds.overlaps(other))
            .map(|other| {
                if sign > 0 {
                    other.max[axis] - bounds.min[axis]
                } else {
                    bounds.max[axis] - other.min[axis]
                }
            })
            .max()
            .unwrap_or(0);
        if shift == 0 {
            return total;
        }
        let by = sign * shift;
        bounds.shift(axis, by);
        total += by;
    }
}

struct Label {
    key: &'static str,
    layer: i32,
    height: i64,
    align: (TextHAlign, TextVAlign),
    position: PointNm,
    axis: usize,
    sign: i128,
    body_obstacles: Option<usize>,
}

fn add_label(
    scene: &mut BoardReviewSceneV1,
    obstacles: &mut Vec<Bounds>,
    symbol: Uuid,
    pin: Uuid,
    index: usize,
    content: &str,
    label: Label,
) -> Result<()> {
    let key = format!("{}:{index}", label.key);
    let id = text_uuid(symbol, &key).to_string();
    let mut sink = SchematicTextSink::default();
    sink.push(
        &mut Vec::new(),
        symbol,
        &key,
        content,
        label.position,
        label.height,
        label.align.0,
        label.align.1,
        label.layer,
    );
    sink.bind_last_source(pin);
    let (mut texts, mut geometries, assets) = sink.into_parts();
    let Some(geometry) = geometries.first_mut() else {
        return Ok(());
    };
    let Some(mut bounds) = ink_bounds(geometry) else {
        return Ok(());
    };
    let original = bounds;
    let mut by = clearance(&mut bounds, obstacles, label.axis, label.sign);
    if let Some(count) = label.body_obstacles {
        let axis = label.axis;
        let across = 1 - axis;
        let anchor = Bounds::point(label.position)[axis];
        // An undersized authored body cannot give every name full clearance.
        // Do not escape a far outline by carrying a name to the opposite pin.
        // Preserve its owning-side anchor and full text instead.
        let crosses_far_edge = obstacles[..count].iter().any(|edge| {
            let crosses_band =
                original.min[across] < edge.max[across] && edge.min[across] < original.max[across];
            crosses_band
                && if label.sign > 0 {
                    edge.min[axis] > anchor && bounds.max[axis] > edge.min[axis]
                } else {
                    edge.max[axis] < anchor && bounds.min[axis] < edge.max[axis]
                }
        });
        if crosses_far_edge {
            bounds = original;
            // Keep the existing own-pin ink clearance without using a far
            // outline as a reason to carry the name across the symbol.
            by = clearance(&mut bounds, &obstacles[count..], axis, label.sign);
            let near = obstacles[..count]
                .iter()
                .map(|edge| edge.min[axis])
                .min()
                .unwrap();
            let far = obstacles[..count]
                .iter()
                .map(|edge| edge.max[axis])
                .max()
                .unwrap();
            let center = (near + far) / 2;
            let shifted = anchor + by;
            if (anchor < center && shifted >= center) || (anchor > center && shifted <= center) {
                // Oversized own-pin text can also be impossible to separate.
                // Retain association rather than trade it for full clearance.
                bounds = original;
                by = 0;
            }
        }
    }
    let translate = |point: &mut PointNm| -> Result<()> {
        let component = if label.axis == 0 {
            &mut point.x
        } else {
            &mut point.y
        };
        *component =
            i64::try_from(i128::from(*component) + by).context("Pin label position overflow")?;
        Ok(())
    };
    translate(&mut texts[0].position)?;
    if let Some(t) = &mut geometry.world_transform_nm {
        let mut point = PointNm {
            x: t.tx_nm,
            y: t.ty_nm,
        };
        translate(&mut point)?;
        t.tx_nm = point.x;
        t.ty_nm = point.y;
    }
    for fill in &mut geometry.fills {
        for point in fill.outer.iter_mut().chain(fill.holes.iter_mut().flatten()) {
            translate(point)?;
        }
    }
    for stroke in &mut geometry.strokes {
        translate(&mut stroke.from)?;
        translate(&mut stroke.to)?;
    }
    scene.board_texts.retain(|text| text.text_uuid != id);
    scene
        .board_text_geometries
        .retain(|geometry| geometry.text_uuid != id);
    scene.board_texts.extend(texts);
    scene.board_text_geometries.extend(geometries);
    for asset in assets {
        if !scene
            .glyph_mesh_assets
            .iter()
            .any(|old| old.handle == asset.handle)
        {
            scene.glyph_mesh_assets.push(asset);
        }
    }
    obstacles.push(bounds);
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct LabelStyle {
    pub name_height: i64,
    pub number_height: i64,
    pub terminal_radius: i64,
    pub stub_width: i64,
    pub body_width: Option<i64>,
    pub minimum_stroke_radius: i64,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            name_height: PIN_NAME_HEIGHT_NM,
            number_height: PIN_NUMBER_HEIGHT_NM,
            terminal_radius: PIN_TERMINAL_RADIUS_NM + schematic_stroke::TERMINAL_OUTLINE_NM / 2,
            stub_width: schematic_stroke::PIN_LINE_NM,
            body_width: Some(schematic_stroke::SYMBOL_OUTLINE_NM),
            minimum_stroke_radius: 0,
        }
    }
}

pub(super) fn project(
    scene: &mut BoardReviewSceneV1,
    body_start: usize,
    symbol: Uuid,
    index: usize,
    pin: &SymbolPin,
    from: PointNm,
    to: PointNm,
    direction: [i128; 2],
    style: LabelStyle,
) -> Result<()> {
    let mut obstacles: Vec<_> = scene.board_graphics[body_start..]
        .iter()
        .filter(|graphic| graphic.object_kind == "schematic_symbol")
        .flat_map(|graphic| {
            let radius = radius(style.body_width.or(graphic.width_nm).unwrap_or(0))
                .max(style.minimum_stroke_radius.into());
            graphic
                .path
                .windows(2)
                .map(move |p| Bounds::stroke(p[0], p[1], radius))
        })
        .collect();
    let body_obstacles = obstacles.len();
    obstacles.extend([
        Bounds::stroke(
            from,
            to,
            radius(style.stub_width).max(style.minimum_stroke_radius.into()),
        ),
        Bounds::stroke(from, from, i128::from(style.terminal_radius)),
    ]);

    let [dx, dy] = direction;
    let horizontal = dx.abs() >= dy.abs();
    let inward = if horizontal { dx } else { dy };
    let inward = if inward < 0 { -1 } else { 1 };
    let midpoint = |a: i64, b: i64| ((i128::from(a) + i128::from(b)) / 2) as i64;
    let number = Label {
        key: "pin-number",
        layer: SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT,
        height: style.number_height,
        align: if horizontal {
            (TextHAlign::Center, TextVAlign::Bottom)
        } else {
            (TextHAlign::Left, TextVAlign::Center)
        },
        position: PointNm {
            x: midpoint(from.x, to.x),
            y: midpoint(from.y, to.y),
        },
        axis: usize::from(horizontal),
        sign: if horizontal { -1 } else { 1 },
        body_obstacles: None,
    };
    let name = Label {
        key: "pin-name",
        layer: SCHEMATIC_PIN_NAME_TEXT_LAYER_INT,
        height: style.name_height,
        align: match (horizontal, inward) {
            (true, 1) => (TextHAlign::Left, TextVAlign::Center),
            (true, _) => (TextHAlign::Right, TextVAlign::Center),
            (false, 1) => (TextHAlign::Center, TextVAlign::Top),
            (false, _) => (TextHAlign::Center, TextVAlign::Bottom),
        },
        position: to,
        axis: usize::from(!horizontal),
        sign: inward,
        body_obstacles: Some(body_obstacles),
    };
    add_label(
        scene,
        &mut obstacles,
        symbol,
        pin.uuid,
        index,
        &pin.number,
        number,
    )?;
    add_label(
        scene,
        &mut obstacles,
        symbol,
        pin.uuid,
        index,
        &pin.name,
        name,
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "native_pin_label_layout_tests.rs"]
mod tests;
