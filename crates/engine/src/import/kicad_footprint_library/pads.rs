use super::{Node, Result, identity, length, point, refusal};
use crate::pool::{
    Pad, Padstack, PadstackAperture as A, PadstackLayerSpan as S, PadstackMaskPolicy as M,
    PadstackPastePolicy as P,
};
use std::collections::HashMap;

pub(super) fn convert(
    root: &Node,
    source: &str,
) -> Result<(HashMap<uuid::Uuid, Pad>, Vec<Padstack>)> {
    let mut pads = HashMap::new();
    let mut stacks = Vec::new();
    for (index, node) in root.children("pad").enumerate() {
        for child in node.items().iter().skip(4) {
            if child.tag().is_none() {
                return Err(refusal("unsupported pad flag alters copper membership"));
            }
            if !matches!(
                child.tag(),
                Some(
                    "at" | "size"
                        | "layers"
                        | "drill"
                        | "roundrect_rratio"
                        | "solder_mask_margin"
                        | "solder_paste_margin"
                        | "solder_paste_margin_ratio"
                        | "uuid"
                        | "tstamp"
                )
            ) {
                return Err(refusal(format!(
                    "pad {index}: unsupported {}",
                    child.tag().unwrap_or("feature")
                )));
            }
        }
        let name = node.required_atom(1)?.to_owned();
        let through = match node.required_atom(2)? {
            "smd" => false,
            "thru_hole" => true,
            other => return Err(refusal(format!("pad {name}: unsupported mounting {other}"))),
        };
        let at = node.required_child("at")?;
        let position = point(at)?;
        let size = node.required_child("size")?;
        let (mut w, mut h) = (length(size, 1)?, length(size, 2)?);
        if w <= 0 || h <= 0 {
            return Err(refusal(format!("pad {name}: nonpositive size")));
        }
        let shape = node.required_atom(3)?;
        let angle = if let Some(token) = at.atom(3) {
            crate::ir::units::parse_decimal_degrees(
                token,
                crate::ir::units::CanonicalAngleScale::new(1),
                true,
            )
            .map_err(|_| refusal("invalid pad rotation"))?
            .canonical_value
            .rem_euclid(360)
        } else {
            0
        };
        if shape != "circle" {
            match angle {
                0 | 180 => {}
                90 | 270 => std::mem::swap(&mut w, &mut h),
                _ => {
                    return Err(refusal(format!(
                        "pad {name}: noncardinal rotation is not supported"
                    )));
                }
            }
        }
        let aperture = match shape {
            "circle" if w == h => A::Circle { diameter_nm: w },
            "rect" => A::Rect {
                width_nm: w,
                height_nm: h,
            },
            "roundrect" => {
                let ratio = length(node.required_child("roundrect_rratio")?, 1)?;
                if !(0..=500_000).contains(&ratio) {
                    return Err(refusal("rounded pad radius ratio outside 0..0.5"));
                }
                A::RoundRect {
                    width_nm: w,
                    height_nm: h,
                    rratio_ppm: ratio as u32,
                }
            }
            // A capsule is exactly a rounded rectangle with half-minimum radius.
            "oval" => A::RoundRect {
                width_nm: w,
                height_nm: h,
                rratio_ppm: 500_000,
            },
            other => return Err(refusal(format!("pad {name}: unsupported shape {other}"))),
        };
        let layers: Vec<_> = node
            .required_child("layers")?
            .items()
            .iter()
            .skip(1)
            .map(|n| match n {
                Node::Atom(s) => Ok(s.as_str()),
                _ => Err(refusal("invalid pad layers")),
            })
            .collect::<Result<_>>()?;
        let expected: &[&str] = if through {
            &["*.Cu", "*.Mask"]
        } else {
            &["F.Cu", "F.Mask", "F.Paste"]
        };
        if layers.iter().any(|l| !expected.contains(l)) || !layers.contains(&expected[0]) {
            return Err(refusal(format!(
                "pad {name}: unsupported copper/process layer membership"
            )));
        }
        let drill_nm = if through {
            let drill = node.required_child("drill")?;
            if drill.items().len() != 2 {
                return Err(refusal("slotted or offset drills are unsupported"));
            }
            let diameter = length(drill, 1)?;
            if diameter <= 0 || diameter > w.min(h) {
                return Err(refusal("invalid plated drill diameter"));
            }
            Some(diameter)
        } else {
            if node.child("drill").is_some() {
                return Err(refusal("SMD pad cannot contain a drill"));
            }
            None
        };
        let margin = |tag| -> Result<Option<i64>> {
            node.child(tag)
                .or_else(|| root.child(tag))
                .map(|n| length(n, 1))
                .transpose()
        };
        let mask_margin = margin("solder_mask_margin")?;
        let paste_margin = margin("solder_paste_margin")?;
        let paste_ratio = node
            .child("solder_paste_margin_ratio")
            .or_else(|| root.child("solder_paste_ratio"))
            .map(|n| length(n, 1))
            .transpose()?;
        if (mask_margin.is_some() || paste_margin.is_some() || paste_ratio.is_some())
            && root.child("version").and_then(|v| v.atom(1)) != Some("20241229")
        {
            return Err(refusal(
                "explicit process overrides require the supported 20241229 source version; legacy override inheritance is ambiguous",
            ));
        }
        let mask_policy = if !layers.iter().any(|l| l.ends_with(".Mask")) {
            M::Tented
        } else {
            match mask_margin {
                Some(n) => M::ExpansionNm(n),
                None => M::Inherit,
            }
        };
        let paste_policy = if !layers.contains(&"F.Paste") {
            P::None
        } else {
            match (paste_margin, paste_ratio) {
                (None, None) => P::Inherit,
                (Some(n), Some(0)) => P::ExpansionNm(n),
                (Some(_), Some(_)) => {
                    return Err(refusal("nonzero source paste ratio is not yet represented"));
                }
                _ => {
                    return Err(refusal(
                        "partial source paste overrides leave an inherited margin or ratio unresolved",
                    ));
                }
            }
        };
        let uuid = identity(source, &format!("pad:{index}"));
        let sid = identity(source, &format!("padstack:{index}"));
        let mut stack = Padstack::new(
            sid,
            format!("{} pad {} ({index})", root.required_atom(1)?, name),
        );
        stack.aperture = Some(aperture);
        stack.drill_nm = drill_nm;
        stack.plated = Some(through);
        stack.layer_span = if through { S::Through } else { S::PadLayer };
        stack.mask_policy = mask_policy;
        stack.paste_policy = paste_policy;
        pads.insert(
            uuid,
            Pad {
                uuid,
                name,
                position,
                padstack: sid,
                layer: 1,
            },
        );
        stacks.push(stack);
    }
    Ok((pads, stacks))
}
