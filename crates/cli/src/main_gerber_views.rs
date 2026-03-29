use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberOutlineExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) outline_vertex_count: usize,
    pub(crate) outline_closed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberCopperExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) pad_count: usize,
    pub(crate) track_count: usize,
    pub(crate) zone_count: usize,
    pub(crate) via_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSoldermaskExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) pad_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPasteExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) pad_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberOutlineValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) outline_vertex_count: usize,
    pub(crate) outline_closed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberCopperValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) pad_count: usize,
    pub(crate) track_count: usize,
    pub(crate) zone_count: usize,
    pub(crate) via_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSoldermaskValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) pad_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPasteValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) pad_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberGeometryEntryView {
    pub(crate) kind: String,
    pub(crate) geometry: String,
    pub(crate) count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberOutlineComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) expected_outline_count: usize,
    pub(crate) actual_geometry_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) missing: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberCopperComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) expected_pad_count: usize,
    pub(crate) actual_pad_count: usize,
    pub(crate) expected_track_count: usize,
    pub(crate) actual_track_count: usize,
    pub(crate) expected_zone_count: usize,
    pub(crate) actual_zone_count: usize,
    pub(crate) expected_via_count: usize,
    pub(crate) actual_via_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) missing: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberSoldermaskComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) expected_pad_count: usize,
    pub(crate) actual_pad_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) missing: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPasteComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) source_copper_layer: i32,
    pub(crate) expected_pad_count: usize,
    pub(crate) actual_pad_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) missing: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPlanArtifactView {
    pub(crate) kind: String,
    pub(crate) layer_id: Option<i32>,
    pub(crate) layer_name: Option<String>,
    pub(crate) filename: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPlanView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) prefix: String,
    pub(crate) outline_vertex_count: usize,
    pub(crate) outline_closed: bool,
    pub(crate) copper_layers: usize,
    pub(crate) soldermask_layers: usize,
    pub(crate) silkscreen_layers: usize,
    pub(crate) paste_layers: usize,
    pub(crate) mechanical_layers: usize,
    pub(crate) artifacts: Vec<NativeProjectGerberPlanArtifactView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberPlanComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) present_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) extra: Vec<String>,
}
