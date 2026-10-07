//! Existing exact subset reader; source-preserving graphics extend this owner.
use super::*;
pub(crate) fn parse_rs274x_subset(gerber: &str) -> Result<ParsedGerber> {
    let mut aperture_map = BTreeMap::<usize, ParsedGerberAperture>::new();
    let mut current_aperture = None;
    let mut current_position = None;
    let mut pending_stroke = None::<PendingStroke>;
    let mut in_region = false;
    let mut region_points = Vec::<Point>::new();
    let mut geometries = Vec::new();
    let mut macros = BTreeMap::new();
    let mut pending_macro: Option<(String, Vec<Vec<Point>>)> = None;
    let mut arc_direction = None;

    for raw_line in gerber.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("G04") {
            continue;
        }
        if let Some((name, rings)) = &mut pending_macro {
            if line == "%" {
                let (name, mut rings) = pending_macro.take().unwrap();
                if rings.is_empty() {
                    bail!("empty Gerber polygon macro: {name}");
                }
                let outer = rings.remove(0);
                eda_engine::board::graphics::GraphicGeometry::Polygon {
                    outer: outer.clone(),
                    holes: rings.clone(),
                    filled: true,
                }
                .validate()
                .map_err(|e| anyhow::anyhow!("invalid exact polygon macro: {e:?}"))?;
                macros.insert(
                    name,
                    ParsedGerberAperture::Polygon {
                        outer,
                        holes: rings,
                    },
                );
            } else {
                rings.push(
                    super::graphic_semantics::macro_ring(line, rings.is_empty())
                        .with_context(|| format!("invalid exact Gerber macro {name}"))?,
                );
            }
            continue;
        }
        if let Some(name) = line.strip_prefix("%AM").and_then(|s| s.strip_suffix('*')) {
            if name.is_empty() || macros.contains_key(name) {
                bail!("ambiguous Gerber macro name");
            }
            pending_macro = Some((name.into(), vec![]));
            continue;
        }
        if matches!(line, "G01*" | "G02*" | "G03*" | "G75*") {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            if line != "G75*" {
                arc_direction = match line {
                    "G02*" => Some(false),
                    "G03*" => Some(true),
                    _ => None,
                };
            }
            continue;
        }
        if line == "%FSLAX46Y46*%" || line == "%MOMM*%" || line == "%LPD*%" {
            continue;
        }
        if let Some(rest) = line.strip_prefix("%ADD") {
            let split_idx = rest
                .find(|ch: char| ch.is_ascii_alphabetic())
                .context("unsupported Gerber aperture definition in comparison input")?;
            let (code_str, definition) = rest.split_at(split_idx);
            let code = code_str
                .parse::<usize>()
                .context("invalid Gerber aperture code")?;
            if let Some(aperture) = definition
                .strip_suffix("*%")
                .and_then(|name| macros.get(name))
            {
                if aperture_map.insert(code, aperture.clone()).is_some() {
                    bail!("duplicate Gerber aperture code");
                }
                continue;
            }
            let (kind, params) = definition.split_at(1);
            let params = params
                .strip_prefix(',')
                .context("unsupported Gerber aperture definition in comparison input")?
                .strip_suffix("*%")
                .context("unterminated Gerber aperture definition in comparison input")?;
            let code = code_str
                .parse::<usize>()
                .context("invalid Gerber aperture code in comparison input")?;
            let aperture = match kind {
                "C" => super::graphic_semantics::circle_aperture(params)?,
                "R" => {
                    let (width_str, height_str) = params.split_once('X').context(
                        "invalid Gerber rectangular aperture definition in comparison input",
                    )?;
                    ParsedGerberAperture::Rect {
                        width_nm: parse_mm_6_to_nm(width_str).context(
                            "invalid Gerber rectangular aperture width in comparison input",
                        )?,
                        height_nm: parse_mm_6_to_nm(height_str).context(
                            "invalid Gerber rectangular aperture height in comparison input",
                        )?,
                    }
                }
                _ => bail!("unsupported Gerber aperture definition in comparison input: {line}"),
            };
            if aperture_map.insert(code, aperture).is_some() {
                bail!("duplicate Gerber aperture code");
            }
            continue;
        }
        if line == "G36*" {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            if in_region {
                bail!("nested Gerber regions are not supported by semantic comparison");
            }
            in_region = true;
            region_points.clear();
            continue;
        }
        if line == "G37*" {
            if !in_region {
                bail!("Gerber region end encountered without region start");
            }
            in_region = false;
            if region_points.len() >= 2 {
                geometries.push(ParsedGerberGeometry::Region {
                    points: canonicalize_path_points(&region_points),
                });
            }
            region_points.clear();
            continue;
        }
        if let Some(code) = parse_aperture_select(line)? {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            current_aperture = Some(code);
            continue;
        }
        if line == "M02*" {
            break;
        }

        if let Some(counterclockwise) = arc_direction
            && line.contains('I')
        {
            if in_region {
                bail!("curved regions remain outside the exact comparison subset");
            }
            let from = current_position.context("arc has no current point")?;
            let code = current_aperture.context("arc has no aperture")?;
            let width = match aperture_map.get(&code) {
                Some(ParsedGerberAperture::Circle { diameter_nm }) => *diameter_nm,
                _ => bail!("arc requires a solid circular aperture"),
            };
            let (to, offset) = super::graphic_semantics::arc_command(line, from)?;
            geometries.push(ParsedGerberGeometry::Arc {
                width_nm: width,
                from,
                to,
                center_offset: offset,
                counterclockwise,
            });
            current_position = Some(to);
            continue;
        }

        let Some((point, operation)) = parse_gerber_coordinate_operation(line)? else {
            bail!("unsupported Gerber command in semantic comparison input: {line}");
        };
        let previous_position = current_position;
        current_position = Some(point);

        if in_region {
            match operation {
                2 => {
                    region_points.clear();
                    region_points.push(point);
                }
                1 => {
                    region_points.push(point);
                }
                3 => bail!("Gerber flashes inside regions are not supported"),
                _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
            }
            continue;
        }

        let aperture_code = current_aperture.context(
            "Gerber semantic comparison requires an active circular aperture before geometry",
        )?;
        let aperture = aperture_map.get(&aperture_code).with_context(|| {
            format!("unknown Gerber aperture D{aperture_code} in comparison input")
        })?;

        match operation {
            2 => {
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    _ => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                pending_stroke = Some(PendingStroke {
                    aperture_diameter_nm,
                    points: vec![point],
                });
            }
            1 => {
                if arc_direction.is_some() {
                    bail!("arc command is missing its exact I/J offsets");
                }
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    _ => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                let stroke = pending_stroke.get_or_insert_with(|| PendingStroke {
                    aperture_diameter_nm,
                    points: previous_position.into_iter().collect::<Vec<_>>(),
                });
                if stroke.aperture_diameter_nm != aperture_diameter_nm {
                    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                    pending_stroke = Some(PendingStroke {
                        aperture_diameter_nm,
                        points: previous_position
                            .into_iter()
                            .chain(std::iter::once(point))
                            .collect::<Vec<_>>(),
                    });
                } else {
                    stroke.points.push(point);
                }
            }
            3 => {
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                geometries.push(ParsedGerberGeometry::Flash {
                    aperture: aperture.clone(),
                    position: point,
                });
            }
            _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
        }
    }

    if pending_macro.is_some() {
        bail!("unterminated Gerber aperture macro");
    }
    if in_region {
        bail!("unterminated Gerber region in comparison input");
    }
    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
    Ok(ParsedGerber { geometries })
}
fn finalize_pending_stroke(
    pending_stroke: &mut Option<PendingStroke>,
    geometries: &mut Vec<ParsedGerberGeometry>,
) {
    if let Some(stroke) = pending_stroke.take()
        && stroke.points.len() >= 2
    {
        geometries.push(ParsedGerberGeometry::Stroke {
            aperture_diameter_nm: stroke.aperture_diameter_nm,
            points: canonicalize_path_points(&stroke.points),
        });
    }
}
fn parse_aperture_select(line: &str) -> Result<Option<usize>> {
    if !line.starts_with('D') || !line.ends_with('*') {
        return Ok(None);
    }
    let code = line[1..line.len() - 1]
        .parse::<usize>()
        .context("invalid Gerber aperture selection in comparison input")?;
    if code < 10 {
        return Ok(None);
    }
    Ok(Some(code))
}
fn parse_gerber_coordinate_operation(line: &str) -> Result<Option<(Point, u8)>> {
    let Some(rest) = line.strip_prefix('X') else {
        return Ok(None);
    };
    let (x_str, rest) = rest
        .split_once('Y')
        .context("invalid Gerber coordinate command in comparison input")?;
    let (y_str, d_str) = rest
        .split_once('D')
        .context("invalid Gerber coordinate command in comparison input")?;
    let operation = d_str
        .strip_suffix('*')
        .context("unterminated Gerber coordinate command in comparison input")?
        .parse::<u8>()
        .context("invalid Gerber coordinate operation in comparison input")?;
    Ok(Some((
        Point {
            x: x_str
                .parse::<i64>()
                .context("invalid Gerber X coordinate in comparison input")?,
            y: y_str
                .parse::<i64>()
                .context("invalid Gerber Y coordinate in comparison input")?,
        },
        operation,
    )))
}
