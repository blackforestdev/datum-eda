//! One typed schematic edit request per native verb, with its verb/params
//! adapter. Extracted whole from `schematic_authoring.rs`; the public path
//! `schematic_authoring::SchematicEditRequest` is re-exported unchanged.
use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchematicEditRequest {
    DesignEdit {
        sheet: Uuid,
        instance_path: Vec<Uuid>,
        edit: eda_engine::api::native_write::design_clipboard_edit::DesignEditIntent,
    },
    Text {
        sheet: Uuid,
        text: Option<Uuid>,
        instance_path: Vec<Uuid>,
        value: Option<String>,
        x_nm: Option<i64>,
        y_nm: Option<i64>,
        rotation_deg: Option<i32>,
    },
    CustomField {
        symbol: Uuid,
        instance_path: Vec<Uuid>,
        field_id: Option<Uuid>,
        key: Option<String>,
        value: Option<String>,
        clear: bool,
        visible: Option<bool>,
        position: Option<eda_engine::ir::geometry::Point>,
    },
    AddBusEntry {
        sheet: Uuid,
        bus: Uuid,
        instance_path: Vec<Uuid>,
        wire: Uuid,
        at: PointNm,
        wire_end: eda_engine::api::native_write::schematic_bus::WireEnd,
        member: Option<String>,
    },
    DeleteBusEntry {
        bus: Uuid,
        bus_entry: Uuid,
        instance_path: Vec<Uuid>,
    },
    CreateBus {
        sheet: Uuid,
        bus: Option<Uuid>,
        instance_path: Vec<Uuid>,
        name: String,
        members: Vec<String>,
        points: Option<Vec<eda_engine::ir::geometry::Point>>,
    },
    PlacePort {
        sheet: Uuid,
        port: Option<Uuid>,
        instance_path: Vec<Uuid>,
        name: String,
        direction: PortDirection,
        x_nm: i64,
        y_nm: i64,
    },
    EditLabel {
        label: Uuid,
        instance_path: Vec<Uuid>,
        name: String,
        kind: LabelKind,
    },
    EditSymbol {
        symbol: Uuid,
        instance_path: Vec<Uuid>,
        reference: String,
        value: String,
    },
    DeleteObject {
        object: Uuid,
        instance_path: Vec<Uuid>,
        kind: selected::DeleteKind,
    },
    CreateSheet {
        sheet: Uuid,
        name: String,
    },
    PlaceSymbol {
        sheet: Uuid,
        #[serde(default)]
        instance_path: Vec<Uuid>,
        library_symbol: Uuid,
        #[serde(default)]
        explicit_part: Option<PartGateSelection>,
        #[serde(default)]
        component: Option<
            eda_engine::api::native_write::schematic_library::part_placement::CompletePartSelection,
        >,
        reference: String,
        value: String,
        x_nm: i64,
        y_nm: i64,
        #[serde(default)]
        rotation_deg: i32,
        #[serde(default)]
        mirrored: bool,
    },
    DrawWire {
        sheet: Uuid,
        #[serde(default)]
        instance_path: Vec<Uuid>,
        points: Vec<eda_engine::ir::geometry::Point>,
    },
    PlaceMarker {
        sheet: Uuid,
        #[serde(default)]
        instance_path: Vec<Uuid>,
        kind: MarkerKind,
        x_nm: i64,
        y_nm: i64,
        symbol: Option<Uuid>,
        pin: Option<Uuid>,
    },
    PlaceLabel {
        sheet: Uuid,
        instance_path: Vec<Uuid>,
        kind: LabelKind,
        name: String,
        x_nm: i64,
        y_nm: i64,
    },
}
impl SchematicEditRequest {
    pub fn verb_params(&self) -> (&'static str, serde_json::Value) {
        match self {
            Self::DesignEdit {
                sheet,
                instance_path,
                edit,
            } => (
                "datum.schematic.move",
                serde_json::json!({"sheet":sheet,"instance_path":instance_path,"design_edit":edit}),
            ),
            Self::Text {
                sheet,
                text,
                instance_path,
                value,
                x_nm,
                y_nm,
                rotation_deg,
            } => {
                let mut params = serde_json::json!({"sheet":sheet,"instance_path":instance_path});
                if let Some(id) = text {
                    params["text"] = serde_json::json!(id);
                }
                if let Some(value) = value {
                    params["value"] = serde_json::json!(value);
                }
                if let Some(x) = x_nm {
                    params["x_nm"] = serde_json::json!(x);
                }
                if let Some(y) = y_nm {
                    params["y_nm"] = serde_json::json!(y);
                }
                if let Some(rotation) = rotation_deg {
                    params["rotation_deg"] = serde_json::json!(rotation);
                }
                ("datum.schematic.place_text", params)
            }
            Self::CustomField {
                symbol,
                instance_path,
                field_id,
                key,
                value,
                clear,
                visible,
                position,
            } => {
                let mut params = serde_json::json!({"symbol":symbol,"instance_path":instance_path,"field":"custom"});
                if let Some(id) = field_id {
                    params["field_id"] = serde_json::json!(id);
                }
                if let Some(key) = key {
                    params["key"] = serde_json::json!(key);
                }
                if let Some(value) = value {
                    params["value"] = serde_json::json!(value);
                }
                if *clear {
                    params["clear"] = serde_json::json!(true);
                }
                if let Some(visible) = visible {
                    params["visible"] = serde_json::json!(visible);
                }
                if let Some(position) = position {
                    params["position"] = serde_json::json!(position);
                }
                ("datum.schematic.set_symbol_field", params)
            }
            Self::AddBusEntry {
                sheet,
                bus,
                instance_path,
                wire,
                at,
                wire_end,
                member,
            } => (
                "datum.schematic.create_bus",
                serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":instance_path,"entries":[{"wire":wire,"x_nm":at.x,"y_nm":at.y,"wire_end":wire_end,"member":member}]}),
            ),
            Self::DeleteBusEntry {
                bus,
                bus_entry,
                instance_path,
            } => (
                "datum.schematic.delete_bus_entry",
                serde_json::json!({"bus":bus,"bus_entry":bus_entry,"instance_path":instance_path}),
            ),
            Self::CreateBus {
                sheet,
                bus,
                instance_path,
                name,
                members,
                points,
            } => {
                let mut params = serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":instance_path,"name":name,"members":members});
                if let Some(points) = points {
                    params["points"] = serde_json::json!(points);
                }
                ("datum.schematic.create_bus", params)
            }
            Self::PlacePort {
                sheet,
                port,
                instance_path,
                name,
                direction,
                x_nm,
                y_nm,
            } => (
                "datum.schematic.place_port",
                serde_json::json!({"sheet":sheet,"port":port,"instance_path":instance_path,"name":name,"direction":match direction {
                    PortDirection::Input => "input", PortDirection::Output => "output",
                    PortDirection::Bidirectional => "bidirectional", PortDirection::Passive => "passive",
                },"x_nm":x_nm,"y_nm":y_nm}),
            ),
            Self::EditLabel {
                label,
                instance_path,
                name,
                kind,
            } => (
                "datum.schematic.rename_label",
                serde_json::json!({"label":label,"instance_path":instance_path,"name":name,"kind": match kind {
                    LabelKind::Local => "local", LabelKind::Global => "global", LabelKind::Hierarchical => "hierarchical", LabelKind::Power => "power",
                }}),
            ),
            Self::EditSymbol {
                symbol,
                instance_path,
                reference,
                value,
            } => (
                "datum.schematic.set_symbol_reference",
                serde_json::json!({"symbol":symbol,"instance_path":instance_path,"reference":reference,"value":value}),
            ),
            Self::DeleteObject {
                object,
                instance_path,
                kind,
            } => {
                let (verb, key) = kind.verb_and_key();
                let mut params = serde_json::json!({"instance_path":instance_path});
                params[key] = serde_json::json!(object);
                (verb, params)
            }
            Self::CreateSheet { sheet, name } => (
                "datum.schematic.create_sheet",
                serde_json::json!({"sheet":sheet,"name":name}),
            ),
            Self::PlaceSymbol {
                sheet,
                instance_path,
                library_symbol,
                explicit_part,
                component,
                reference,
                value,
                x_nm,
                y_nm,
                rotation_deg,
                mirrored,
            } => (
                "datum.schematic.place_symbol",
                serde_json::json!({"sheet":sheet,"instance_path":instance_path,"library_symbol":library_symbol,"explicit_part":explicit_part,"component":component,"reference":reference,"value":value,"x_nm":x_nm,"y_nm":y_nm,"rotation_deg":rotation_deg,"mirrored":mirrored}),
            ),
            Self::DrawWire {
                sheet,
                instance_path,
                points,
            } => (
                "datum.schematic.draw_wire",
                serde_json::json!({"sheet":sheet,"instance_path":instance_path,"points":points}),
            ),
            Self::PlaceMarker {
                sheet,
                instance_path,
                kind,
                x_nm,
                y_nm,
                symbol,
                pin,
            } => (
                "datum.schematic.place_marker",
                serde_json::json!({"sheet":sheet,"instance_path":instance_path,"kind":kind.token(),"x_nm":x_nm,"y_nm":y_nm,"symbol":symbol,"pin":pin}),
            ),
            Self::PlaceLabel {
                sheet,
                instance_path,
                kind,
                name,
                x_nm,
                y_nm,
            } => (
                "datum.schematic.place_label",
                serde_json::json!({"sheet":sheet,"instance_path":instance_path,"kind": match kind {
                    LabelKind::Local => "local", LabelKind::Global => "global",
                    LabelKind::Hierarchical => "hierarchical", LabelKind::Power => "power",
                },"name":name,"x_nm":x_nm,"y_nm":y_nm}),
            ),
        }
    }
}
