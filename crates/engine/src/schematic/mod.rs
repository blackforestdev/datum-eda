use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::{Arc, Point};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schematic {
    pub uuid: Uuid,
    pub sheets: HashMap<Uuid, Sheet>,
    pub sheet_definitions: HashMap<Uuid, SheetDefinition>,
    pub sheet_instances: HashMap<Uuid, SheetInstance>,
    pub variants: HashMap<Uuid, Variant>,
    pub waivers: Vec<CheckWaiver>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sheet {
    pub uuid: Uuid,
    pub name: String,
    pub frame: Option<SheetFrame>,
    pub symbols: HashMap<Uuid, PlacedSymbol>,
    pub wires: HashMap<Uuid, SchematicWire>,
    pub junctions: HashMap<Uuid, Junction>,
    pub labels: HashMap<Uuid, NetLabel>,
    pub buses: HashMap<Uuid, Bus>,
    pub bus_entries: HashMap<Uuid, BusEntry>,
    pub ports: HashMap<Uuid, HierarchicalPort>,
    pub noconnects: HashMap<Uuid, NoConnectMarker>,
    pub texts: HashMap<Uuid, SchematicText>,
    pub drawings: HashMap<Uuid, SchematicPrimitive>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetFrame {
    pub uuid: Uuid,
    pub title: String,
    pub revision: Option<String>,
    pub company: Option<String>,
    pub page_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetInstance {
    pub uuid: Uuid,
    pub definition: Uuid,
    pub parent_sheet: Option<Uuid>,
    pub position: Point,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetDefinition {
    pub uuid: Uuid,
    pub root_sheet: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedSymbol {
    pub uuid: Uuid,
    pub part: Option<Uuid>,
    pub entity: Option<Uuid>,
    pub gate: Option<Uuid>,
    pub lib_id: Option<String>,
    pub reference: String,
    pub value: String,
    pub fields: Vec<SymbolField>,
    pub pins: Vec<SymbolPin>,
    pub position: Point,
    pub rotation: i32,
    pub mirrored: bool,
    pub unit_selection: Option<String>,
    pub display_mode: SymbolDisplayMode,
    pub pin_overrides: Vec<PinDisplayOverride>,
    pub hidden_power_behavior: HiddenPowerBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolField {
    pub uuid: Uuid,
    pub key: String,
    pub value: String,
    pub position: Option<Point>,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolPin {
    pub uuid: Uuid,
    pub number: String,
    pub name: String,
    pub electrical_type: PinElectricalType,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinElectricalType {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolDisplayMode {
    LibraryDefault,
    ShowHiddenPins,
    HideOptionalPins,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinDisplayOverride {
    pub pin: Uuid,
    pub visible: bool,
    pub position: Option<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HiddenPowerBehavior {
    SourceDefinedImplicit,
    ExplicitPowerObject,
    PreservedAsImportedMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicWire {
    pub uuid: Uuid,
    pub from: Point,
    pub to: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Junction {
    pub uuid: Uuid,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelKind {
    Local,
    Global,
    Hierarchical,
    Power,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetLabel {
    pub uuid: Uuid,
    pub kind: LabelKind,
    pub name: String,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bus {
    pub uuid: Uuid,
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusEntry {
    pub uuid: Uuid,
    pub bus: Uuid,
    pub wire: Option<Uuid>,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalPort {
    pub uuid: Uuid,
    pub name: String,
    pub direction: PortDirection,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
    Passive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoConnectMarker {
    pub uuid: Uuid,
    pub symbol: Uuid,
    pub pin: Uuid,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchematicPrimitive {
    Line {
        uuid: Uuid,
        from: Point,
        to: Point,
    },
    Rect {
        uuid: Uuid,
        min: Point,
        max: Point,
    },
    Circle {
        uuid: Uuid,
        center: Point,
        radius: i64,
    },
    Arc {
        uuid: Uuid,
        arc: Arc,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variant {
    pub uuid: Uuid,
    pub name: String,
    pub fitted_components: HashMap<Uuid, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicNetInfo {
    pub uuid: Uuid,
    pub name: String,
    pub class: Option<String>,
    pub pins: Vec<NetPinRef>,
    pub labels: usize,
    pub ports: usize,
    pub port_uuids: Vec<Uuid>,
    pub sheets: Vec<String>,
    pub semantic_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetPinRef {
    pub uuid: Uuid,
    pub component: String,
    pub pin: String,
    pub electrical_type: PinElectricalType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetSummary {
    pub uuid: Uuid,
    pub name: String,
    pub symbols: usize,
    pub ports: usize,
    pub labels: usize,
    pub buses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub reference: String,
    pub value: String,
    pub lib_id: Option<String>,
    pub position: Point,
    pub rotation: i32,
    pub mirrored: bool,
    pub part_uuid: Option<Uuid>,
    pub entity_uuid: Option<Uuid>,
    pub gate_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub name: String,
    pub direction: PortDirection,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub kind: LabelKind,
    pub name: String,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusEntryInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub bus: Uuid,
    pub wire: Option<Uuid>,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoConnectInfo {
    pub uuid: Uuid,
    pub sheet: Uuid,
    pub symbol: Uuid,
    pub pin: Uuid,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolFieldInfo {
    pub uuid: Uuid,
    pub symbol: Uuid,
    pub key: String,
    pub value: String,
    pub visible: bool,
    pub position: Option<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchyInfo {
    pub instances: Vec<SheetInstanceInfo>,
    pub links: Vec<HierarchicalLinkInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetInstanceInfo {
    pub uuid: Uuid,
    pub definition: Uuid,
    pub parent_sheet: Option<Uuid>,
    pub position: Point,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalLinkInfo {
    pub parent_sheet: Uuid,
    pub child_sheet: Uuid,
    pub parent_port: Uuid,
    pub child_port: Uuid,
    pub net: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityDiagnosticInfo {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub objects: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckDomain {
    ERC,
    DRC,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckWaiver {
    pub uuid: Uuid,
    pub domain: CheckDomain,
    pub target: WaiverTarget,
    pub rationale: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaiverTarget {
    Object(Uuid),
    RuleObject { rule: String, object: Uuid },
    RuleObjects { rule: String, objects: Vec<Uuid> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicSummary {
    pub sheet_count: usize,
    pub symbol_count: usize,
    pub net_label_count: usize,
    pub port_count: usize,
}

impl Schematic {
    pub fn summary(&self) -> SchematicSummary {
        let mut symbol_count = 0usize;
        let mut net_label_count = 0usize;
        let mut port_count = 0usize;

        for sheet in self.sheets.values() {
            symbol_count += sheet.symbols.len();
            net_label_count += sheet.labels.len();
            port_count += sheet.ports.len();
        }

        SchematicSummary {
            sheet_count: self.sheets.len(),
            symbol_count,
            net_label_count,
            port_count,
        }
    }

    pub fn sheet_summaries(&self) -> Vec<SheetSummary> {
        let mut sheets: Vec<_> = self
            .sheets
            .values()
            .map(|sheet| SheetSummary {
                uuid: sheet.uuid,
                name: sheet.name.clone(),
                symbols: sheet.symbols.len(),
                ports: sheet.ports.len(),
                labels: sheet.labels.len(),
                buses: sheet.buses.len(),
            })
            .collect();
        sheets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        sheets
    }

    pub fn labels(&self, sheet: Option<&Uuid>) -> Vec<LabelInfo> {
        let mut labels = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            labels.extend(current.labels.values().map(|label| LabelInfo {
                uuid: label.uuid,
                sheet: current.uuid,
                kind: label.kind.clone(),
                name: label.name.clone(),
                position: label.position,
            }));
        }
        labels.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        labels
    }

    pub fn symbols(&self, sheet: Option<&Uuid>) -> Vec<SymbolInfo> {
        let mut symbols = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            symbols.extend(current.symbols.values().map(|symbol| SymbolInfo {
                uuid: symbol.uuid,
                sheet: current.uuid,
                reference: symbol.reference.clone(),
                value: symbol.value.clone(),
                lib_id: symbol.lib_id.clone(),
                position: symbol.position,
                rotation: symbol.rotation,
                mirrored: symbol.mirrored,
                part_uuid: symbol.part,
                entity_uuid: symbol.entity,
                gate_uuid: symbol.gate,
            }));
        }
        symbols.sort_by(|a, b| {
            a.reference
                .cmp(&b.reference)
                .then_with(|| a.uuid.cmp(&b.uuid))
        });
        symbols
    }

    pub fn ports(&self, sheet: Option<&Uuid>) -> Vec<PortInfo> {
        let mut ports = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            ports.extend(current.ports.values().map(|port| PortInfo {
                uuid: port.uuid,
                sheet: current.uuid,
                name: port.name.clone(),
                direction: port.direction.clone(),
                position: port.position,
            }));
        }
        ports.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        ports
    }

    pub fn buses(&self, sheet: Option<&Uuid>) -> Vec<BusInfo> {
        let mut buses = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            buses.extend(current.buses.values().map(|bus| BusInfo {
                uuid: bus.uuid,
                sheet: current.uuid,
                name: bus.name.clone(),
                members: bus.members.clone(),
            }));
        }
        buses.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        buses
    }

    pub fn bus_entries(&self, sheet: Option<&Uuid>) -> Vec<BusEntryInfo> {
        let mut entries = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            entries.extend(current.bus_entries.values().map(|entry| BusEntryInfo {
                uuid: entry.uuid,
                sheet: current.uuid,
                bus: entry.bus,
                wire: entry.wire,
                position: entry.position,
            }));
        }
        entries.sort_by(|a, b| a.uuid.cmp(&b.uuid));
        entries
    }

    pub fn noconnects(&self, sheet: Option<&Uuid>) -> Vec<NoConnectInfo> {
        let mut noconnects = Vec::new();
        for current in self.iter_selected_sheets(sheet) {
            noconnects.extend(current.noconnects.values().map(|marker| NoConnectInfo {
                uuid: marker.uuid,
                sheet: current.uuid,
                symbol: marker.symbol,
                pin: marker.pin,
                position: marker.position,
            }));
        }
        noconnects.sort_by(|a, b| a.uuid.cmp(&b.uuid));
        noconnects
    }

    pub fn hierarchy(&self) -> HierarchyInfo {
        let mut instances: Vec<_> = self
            .sheet_instances
            .values()
            .map(|instance| SheetInstanceInfo {
                uuid: instance.uuid,
                definition: instance.definition,
                parent_sheet: instance.parent_sheet,
                position: instance.position,
                name: instance.name.clone(),
            })
            .collect();
        instances.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));

        HierarchyInfo {
            instances,
            // Port/link import is not implemented yet, so hierarchy links
            // remain empty until connectivity-aware schematic import lands.
            links: Vec::new(),
        }
    }

    fn iter_selected_sheets<'a>(&'a self, sheet: Option<&Uuid>) -> Vec<&'a Sheet> {
        match sheet {
            Some(target) => self.sheets.get(target).into_iter().collect(),
            None => {
                let mut sheets: Vec<_> = self.sheets.values().collect();
                sheets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
                sheets
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[path = "mod_tests_schematic.rs"]
    mod mod_tests_schematic;
}
