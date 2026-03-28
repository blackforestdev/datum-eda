use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::command_project_gerber_mechanical::{
    NativeComponentMechanicalArc, NativeComponentMechanicalCircle, NativeComponentMechanicalLine,
    NativeComponentMechanicalPolygon, NativeComponentMechanicalPolyline,
    NativeComponentMechanicalText,
};
use super::command_project_gerber_silkscreen::{
    NativeComponentSilkscreenArc, NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
    NativeComponentSilkscreenText,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeBoardRoot {
    pub(crate) schema_version: u32,
    pub(crate) uuid: Uuid,
    pub(crate) name: String,
    pub(crate) stackup: NativeStackup,
    pub(crate) outline: NativeOutline,
    #[serde(default)]
    pub(crate) packages: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) component_silkscreen: BTreeMap<String, Vec<NativeComponentSilkscreenLine>>,
    #[serde(default)]
    pub(crate) component_silkscreen_texts: BTreeMap<String, Vec<NativeComponentSilkscreenText>>,
    #[serde(default)]
    pub(crate) component_silkscreen_arcs: BTreeMap<String, Vec<NativeComponentSilkscreenArc>>,
    #[serde(default)]
    pub(crate) component_silkscreen_circles: BTreeMap<String, Vec<NativeComponentSilkscreenCircle>>,
    #[serde(default)]
    pub(crate) component_silkscreen_polygons:
        BTreeMap<String, Vec<NativeComponentSilkscreenPolygon>>,
    #[serde(default)]
    pub(crate) component_silkscreen_polylines:
        BTreeMap<String, Vec<NativeComponentSilkscreenPolyline>>,
    #[serde(default)]
    pub(crate) component_mechanical_lines: BTreeMap<String, Vec<NativeComponentMechanicalLine>>,
    #[serde(default)]
    pub(crate) component_mechanical_texts: BTreeMap<String, Vec<NativeComponentMechanicalText>>,
    #[serde(default)]
    pub(crate) component_mechanical_polygons:
        BTreeMap<String, Vec<NativeComponentMechanicalPolygon>>,
    #[serde(default)]
    pub(crate) component_mechanical_polylines:
        BTreeMap<String, Vec<NativeComponentMechanicalPolyline>>,
    #[serde(default)]
    pub(crate) component_mechanical_circles: BTreeMap<String, Vec<NativeComponentMechanicalCircle>>,
    #[serde(default)]
    pub(crate) component_mechanical_arcs: BTreeMap<String, Vec<NativeComponentMechanicalArc>>,
    #[serde(default)]
    pub(crate) pads: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) tracks: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) vias: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) zones: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) nets: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) net_classes: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) keepouts: Vec<serde_json::Value>,
    #[serde(default)]
    pub(crate) dimensions: Vec<serde_json::Value>,
    #[serde(default)]
    pub(crate) texts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeStackup {
    pub(crate) layers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativeOutline {
    pub(crate) vertices: Vec<NativePoint>,
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NativePoint {
    pub(crate) x: i64,
    pub(crate) y: i64,
}
