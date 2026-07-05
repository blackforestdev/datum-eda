use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::{Point, Polygon};

use super::{
    Footprint, Pad, Padstack, PadstackAperture, PadstackLayerSpan, PadstackMaskPolicy,
    PadstackPastePolicy,
};

const IPC_7351_FAMILY: &str = "IPC-7351";
const IPC_7351_REVISION: &str = "B";
const TWO_TERMINAL_DERIVATION_VERSION: &str = "datum-ipc7351b-two-terminal-chip-v1";
const SOIC_DERIVATION_VERSION: &str = "datum-ipc7351b-soic-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcDensityLevel {
    Most,
    Nominal,
    Least,
}

impl IpcDensityLevel {
    fn code(self) -> &'static str {
        match self {
            Self::Most => "A",
            Self::Nominal => "B",
            Self::Least => "C",
        }
    }

    fn chip_j_values(self) -> IpcJValues {
        match self {
            Self::Most => IpcJValues {
                toe_nm: 550_000,
                heel_nm: 450_000,
                side_nm: 50_000,
            },
            Self::Nominal => IpcJValues {
                toe_nm: 350_000,
                heel_nm: 350_000,
                side_nm: 0,
            },
            Self::Least => IpcJValues {
                toe_nm: 150_000,
                heel_nm: 250_000,
                side_nm: -50_000,
            },
        }
    }

    fn gull_wing_j_values(self) -> IpcJValues {
        match self {
            Self::Most => IpcJValues {
                toe_nm: 550_000,
                heel_nm: 450_000,
                side_nm: 50_000,
            },
            Self::Nominal => IpcJValues {
                toe_nm: 350_000,
                heel_nm: 350_000,
                side_nm: 0,
            },
            Self::Least => IpcJValues {
                toe_nm: 150_000,
                heel_nm: 250_000,
                side_nm: -50_000,
            },
        }
    }

    fn courtyard_excess_nm(self) -> i64 {
        match self {
            Self::Most => 500_000,
            Self::Nominal => 250_000,
            Self::Least => 100_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcSourceDimensions {
    pub body_length_nm: i64,
    pub body_width_nm: i64,
    pub terminal_length_nm: i64,
    pub terminal_width_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcJValues {
    pub toe_nm: i64,
    pub heel_nm: i64,
    pub side_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcFootprintBasis {
    pub family: String,
    pub revision: String,
    pub density_level: IpcDensityLevel,
    pub package_family: String,
    pub package_code: String,
    pub source_dimensions: IpcSourceDimensions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_nm: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead_span_nm: Option<i64>,
    pub source_j_values: IpcJValues,
    pub courtyard_excess_nm: i64,
    pub mask_expansion_nm: i64,
    pub paste_reduction_nm: i64,
    pub derivation_version: String,
    pub naming_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcTwoTerminalChipSpec {
    pub footprint_uuid: Uuid,
    pub package_uuid: Uuid,
    pub padstack_uuid: Uuid,
    pub pad_a_uuid: Uuid,
    pub pad_b_uuid: Uuid,
    pub name: Option<String>,
    pub metric_code: String,
    pub dimensions: IpcSourceDimensions,
    pub density_level: IpcDensityLevel,
    pub mask_expansion_nm: i64,
    pub paste_reduction_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcSoicSpec {
    pub footprint_uuid: Uuid,
    pub package_uuid: Uuid,
    pub padstack_uuid: Uuid,
    pub pad_uuids: Vec<Uuid>,
    pub name: Option<String>,
    pub package_code: String,
    pub pin_count: u32,
    pub pitch_nm: i64,
    pub body_length_nm: i64,
    pub body_width_nm: i64,
    pub lead_span_nm: i64,
    pub terminal_length_nm: i64,
    pub terminal_width_nm: i64,
    pub density_level: IpcDensityLevel,
    pub mask_expansion_nm: i64,
    pub paste_reduction_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedIpcFootprint {
    pub footprint: Footprint,
    pub padstack: Padstack,
}

pub fn generate_ipc7351b_two_terminal_chip(
    spec: IpcTwoTerminalChipSpec,
) -> Result<GeneratedIpcFootprint, String> {
    validate_two_terminal_spec(&spec)?;

    let j = spec.density_level.chip_j_values();
    let pad_length_nm = spec.dimensions.terminal_length_nm + j.toe_nm + j.heel_nm;
    let pad_width_nm = spec.dimensions.terminal_width_nm + (2 * j.side_nm);
    if pad_length_nm <= 0 {
        return Err("generated IPC chip pad length must be positive".to_string());
    }
    if pad_width_nm <= 0 {
        return Err("generated IPC chip pad width must be positive".to_string());
    }

    let pad_center_x = (spec.dimensions.body_length_nm + j.toe_nm - j.heel_nm) / 2;
    let pad_outer_x = pad_center_x + (pad_length_nm / 2);
    let pad_outer_y = pad_width_nm / 2;
    let courtyard_x = pad_outer_x + spec.density_level.courtyard_excess_nm();
    let courtyard_y = pad_outer_y.max(spec.dimensions.body_width_nm / 2)
        + spec.density_level.courtyard_excess_nm();

    let padstack = Padstack {
        uuid: spec.padstack_uuid,
        name: format!(
            "{}_{}_chip_pad_{}x{}nm",
            IPC_7351_FAMILY,
            spec.density_level.code(),
            pad_length_nm,
            pad_width_nm
        ),
        aperture: Some(PadstackAperture::Rect {
            width_nm: pad_length_nm,
            height_nm: pad_width_nm,
        }),
        drill_nm: None,
        plated: Some(false),
        layer_span: PadstackLayerSpan::PadLayer,
        mask_policy: PadstackMaskPolicy::ExpansionNm(spec.mask_expansion_nm),
        paste_policy: PadstackPastePolicy::ExpansionNm(-spec.paste_reduction_nm.abs()),
        annular_ring_nm: None,
        thermal: None,
        antipad: None,
    };

    let pads = HashMap::from([
        (
            spec.pad_a_uuid,
            Pad {
                uuid: spec.pad_a_uuid,
                name: "1".to_string(),
                position: Point::new(-pad_center_x, 0),
                padstack: spec.padstack_uuid,
                layer: 1,
            },
        ),
        (
            spec.pad_b_uuid,
            Pad {
                uuid: spec.pad_b_uuid,
                name: "2".to_string(),
                position: Point::new(pad_center_x, 0),
                padstack: spec.padstack_uuid,
                layer: 1,
            },
        ),
    ]);

    let basis = IpcFootprintBasis {
        family: IPC_7351_FAMILY.to_string(),
        revision: IPC_7351_REVISION.to_string(),
        density_level: spec.density_level,
        package_family: "two_terminal_chip".to_string(),
        package_code: spec.metric_code.clone(),
        source_dimensions: spec.dimensions,
        pin_count: None,
        pitch_nm: None,
        lead_span_nm: None,
        source_j_values: j,
        courtyard_excess_nm: spec.density_level.courtyard_excess_nm(),
        mask_expansion_nm: spec.mask_expansion_nm,
        paste_reduction_nm: spec.paste_reduction_nm.abs(),
        derivation_version: TWO_TERMINAL_DERIVATION_VERSION.to_string(),
        naming_basis: format!(
            "IPC-7351B two-terminal chip density {}",
            spec.density_level.code()
        ),
    };
    let name = spec.name.unwrap_or_else(|| {
        format!(
            "CHIP-{}_IPC7351B_{}",
            basis.package_code,
            spec.density_level.code()
        )
    });

    Ok(GeneratedIpcFootprint {
        footprint: Footprint {
            uuid: spec.footprint_uuid,
            name,
            package: spec.package_uuid,
            pads,
            courtyard: rectangular_polygon(courtyard_x, courtyard_y),
            silkscreen: Vec::new(),
            fab: Vec::new(),
            assembly: Vec::new(),
            mechanical: Vec::new(),
            models_3d: Vec::new(),
            standards_basis: Some(basis.naming_basis.clone()),
            ipc_basis: Some(basis),
            process_aperture_policy: Some("ipc_derived".to_string()),
            tags: HashSet::from(["ipc_7351b".to_string(), "two_terminal_chip".to_string()]),
        },
        padstack,
    })
}

pub fn generate_ipc7351b_soic(spec: IpcSoicSpec) -> Result<GeneratedIpcFootprint, String> {
    validate_soic_spec(&spec)?;

    let j = spec.density_level.gull_wing_j_values();
    let pad_length_nm = spec.terminal_length_nm + j.toe_nm + j.heel_nm;
    let pad_width_nm = spec.terminal_width_nm + (2 * j.side_nm);
    if pad_length_nm <= 0 {
        return Err("generated IPC SOIC pad length must be positive".to_string());
    }
    if pad_width_nm <= 0 {
        return Err("generated IPC SOIC pad width must be positive".to_string());
    }

    let row_count = spec.pin_count / 2;
    let row_x = (spec.lead_span_nm / 2) - (spec.terminal_length_nm / 2)
        + ((j.toe_nm - j.heel_nm) / 2);
    if row_x <= 0 {
        return Err("generated IPC SOIC row center must be positive".to_string());
    }
    let first_y = ((row_count as i64 - 1) * spec.pitch_nm) / 2;
    let mut pads = HashMap::new();
    for index in 0..row_count {
        let pad_id = spec.pad_uuids[index as usize];
        pads.insert(
            pad_id,
            Pad {
                uuid: pad_id,
                name: (index + 1).to_string(),
                position: Point::new(-row_x, first_y - (index as i64 * spec.pitch_nm)),
                padstack: spec.padstack_uuid,
                layer: 1,
            },
        );
    }
    for index in 0..row_count {
        let pad_number = row_count + index + 1;
        let pad_id = spec.pad_uuids[(row_count + index) as usize];
        pads.insert(
            pad_id,
            Pad {
                uuid: pad_id,
                name: pad_number.to_string(),
                position: Point::new(row_x, -first_y + (index as i64 * spec.pitch_nm)),
                padstack: spec.padstack_uuid,
                layer: 1,
            },
        );
    }

    let padstack = Padstack {
        uuid: spec.padstack_uuid,
        name: format!(
            "{}_{}_soic_pad_{}x{}nm",
            IPC_7351_FAMILY,
            spec.density_level.code(),
            pad_length_nm,
            pad_width_nm
        ),
        aperture: Some(PadstackAperture::Rect {
            width_nm: pad_length_nm,
            height_nm: pad_width_nm,
        }),
        drill_nm: None,
        plated: Some(false),
        layer_span: PadstackLayerSpan::PadLayer,
        mask_policy: PadstackMaskPolicy::ExpansionNm(spec.mask_expansion_nm),
        paste_policy: PadstackPastePolicy::ExpansionNm(-spec.paste_reduction_nm.abs()),
        annular_ring_nm: None,
        thermal: None,
        antipad: None,
    };

    let row_outer_x = row_x + (pad_length_nm / 2);
    let row_outer_y = first_y + (pad_width_nm / 2);
    let courtyard_x = row_outer_x.max(spec.body_width_nm / 2)
        + spec.density_level.courtyard_excess_nm();
    let courtyard_y = row_outer_y.max(spec.body_length_nm / 2)
        + spec.density_level.courtyard_excess_nm();

    let basis = IpcFootprintBasis {
        family: IPC_7351_FAMILY.to_string(),
        revision: IPC_7351_REVISION.to_string(),
        density_level: spec.density_level,
        package_family: "soic".to_string(),
        package_code: spec.package_code.clone(),
        source_dimensions: IpcSourceDimensions {
            body_length_nm: spec.body_length_nm,
            body_width_nm: spec.body_width_nm,
            terminal_length_nm: spec.terminal_length_nm,
            terminal_width_nm: spec.terminal_width_nm,
        },
        pin_count: Some(spec.pin_count),
        pitch_nm: Some(spec.pitch_nm),
        lead_span_nm: Some(spec.lead_span_nm),
        source_j_values: j,
        courtyard_excess_nm: spec.density_level.courtyard_excess_nm(),
        mask_expansion_nm: spec.mask_expansion_nm,
        paste_reduction_nm: spec.paste_reduction_nm.abs(),
        derivation_version: SOIC_DERIVATION_VERSION.to_string(),
        naming_basis: format!("IPC-7351B SOIC density {}", spec.density_level.code()),
    };
    let name = spec.name.unwrap_or_else(|| {
        format!(
            "{}_IPC7351B_{}",
            basis.package_code,
            spec.density_level.code()
        )
    });

    Ok(GeneratedIpcFootprint {
        footprint: Footprint {
            uuid: spec.footprint_uuid,
            name,
            package: spec.package_uuid,
            pads,
            courtyard: rectangular_polygon(courtyard_x, courtyard_y),
            silkscreen: Vec::new(),
            fab: Vec::new(),
            assembly: Vec::new(),
            mechanical: Vec::new(),
            models_3d: Vec::new(),
            standards_basis: Some(basis.naming_basis.clone()),
            ipc_basis: Some(basis),
            process_aperture_policy: Some("ipc_derived".to_string()),
            tags: HashSet::from(["ipc_7351b".to_string(), "soic".to_string()]),
        },
        padstack,
    })
}

fn validate_two_terminal_spec(spec: &IpcTwoTerminalChipSpec) -> Result<(), String> {
    if spec.metric_code.trim().is_empty() {
        return Err("IPC chip metric code must be non-empty".to_string());
    }
    if spec.dimensions.body_length_nm <= 0 {
        return Err("IPC chip body_length_nm must be positive".to_string());
    }
    if spec.dimensions.body_width_nm <= 0 {
        return Err("IPC chip body_width_nm must be positive".to_string());
    }
    if spec.dimensions.terminal_length_nm <= 0 {
        return Err("IPC chip terminal_length_nm must be positive".to_string());
    }
    if spec.dimensions.terminal_width_nm <= 0 {
        return Err("IPC chip terminal_width_nm must be positive".to_string());
    }
    if spec.mask_expansion_nm < 0 {
        return Err("IPC chip mask_expansion_nm must not be negative".to_string());
    }
    if spec.paste_reduction_nm < 0 {
        return Err("IPC chip paste_reduction_nm must not be negative".to_string());
    }
    Ok(())
}

fn validate_soic_spec(spec: &IpcSoicSpec) -> Result<(), String> {
    if spec.package_code.trim().is_empty() {
        return Err("IPC SOIC package code must be non-empty".to_string());
    }
    if spec.pin_count < 4 || spec.pin_count % 2 != 0 {
        return Err("IPC SOIC pin_count must be an even value of at least 4".to_string());
    }
    if spec.pad_uuids.len() != spec.pin_count as usize {
        return Err("IPC SOIC pad_uuids length must equal pin_count".to_string());
    }
    if spec.pitch_nm <= 0 {
        return Err("IPC SOIC pitch_nm must be positive".to_string());
    }
    if spec.body_length_nm <= 0 {
        return Err("IPC SOIC body_length_nm must be positive".to_string());
    }
    if spec.body_width_nm <= 0 {
        return Err("IPC SOIC body_width_nm must be positive".to_string());
    }
    if spec.lead_span_nm <= 0 {
        return Err("IPC SOIC lead_span_nm must be positive".to_string());
    }
    if spec.terminal_length_nm <= 0 {
        return Err("IPC SOIC terminal_length_nm must be positive".to_string());
    }
    if spec.terminal_width_nm <= 0 {
        return Err("IPC SOIC terminal_width_nm must be positive".to_string());
    }
    if spec.mask_expansion_nm < 0 {
        return Err("IPC SOIC mask_expansion_nm must not be negative".to_string());
    }
    if spec.paste_reduction_nm < 0 {
        return Err("IPC SOIC paste_reduction_nm must not be negative".to_string());
    }
    Ok(())
}

fn rectangular_polygon(half_x: i64, half_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(-half_x, -half_y),
        Point::new(half_x, -half_y),
        Point::new(half_x, half_y),
        Point::new(-half_x, half_y),
    ])
}
