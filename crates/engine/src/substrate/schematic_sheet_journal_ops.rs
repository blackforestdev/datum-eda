use super::schematic_sheet_maps::{
    insert_sheet_map_value, remove_sheet_map_value, sheet_map_value, sheet_uuid,
};
use super::{EngineError, Operation};

pub(super) fn apply_schematic_sheet_operation(
    sheet_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetSchematicSheetName { sheet_id, name }
            if sheet_uuid(sheet_value) == Some(*sheet_id) =>
        {
            set_sheet_name(sheet_value, name)?;
            Ok(true)
        }
        Operation::CreateSchematicWire {
            sheet_id,
            wire_id,
            wire,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "wires", *wire_id, wire.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicWire {
            sheet_id, wire_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "wires", *wire_id)?;
            Ok(true)
        }
        Operation::CreateSchematicJunction {
            sheet_id,
            junction_id,
            junction,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "junctions", *junction_id, junction.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "junctions", *junction_id)?;
            Ok(true)
        }
        Operation::CreateSchematicNoConnect {
            sheet_id,
            noconnect_id,
            noconnect,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "noconnects", *noconnect_id, noconnect.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicNoConnect {
            sheet_id,
            noconnect_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "noconnects", *noconnect_id)?;
            Ok(true)
        }
        Operation::CreateSchematicLabel {
            sheet_id,
            label_id,
            label,
        }
        | Operation::SetSchematicLabel {
            sheet_id,
            label_id,
            label,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "labels", *label_id, label.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicLabel {
            sheet_id, label_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "labels", *label_id)?;
            Ok(true)
        }
        Operation::CreateSchematicPort {
            sheet_id,
            port_id,
            port,
        }
        | Operation::SetSchematicPort {
            sheet_id,
            port_id,
            port,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "ports", *port_id, port.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicPort {
            sheet_id, port_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "ports", *port_id)?;
            Ok(true)
        }
        Operation::CreateSchematicBus {
            sheet_id,
            bus_id,
            bus,
        }
        | Operation::SetSchematicBus {
            sheet_id,
            bus_id,
            bus,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "buses", *bus_id, bus.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicBus {
            sheet_id, bus_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "buses", *bus_id)?;
            Ok(true)
        }
        Operation::CreateSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            bus_entry,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "bus_entries", *bus_entry_id, bus_entry.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "bus_entries", *bus_entry_id)?;
            Ok(true)
        }
        Operation::CreateSchematicText {
            sheet_id,
            text_id,
            text,
        }
        | Operation::SetSchematicText {
            sheet_id,
            text_id,
            text,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicText {
            sheet_id, text_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "texts", *text_id)?;
            Ok(true)
        }
        Operation::CreateSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        }
        | Operation::SetSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "drawings", *drawing_id, drawing.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicDrawing {
            sheet_id,
            drawing_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "drawings", *drawing_id)?;
            Ok(true)
        }
        Operation::CreateSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        }
        | Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            insert_sheet_map_value(sheet_value, "symbols", *symbol_id, symbol.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicSymbol {
            sheet_id,
            symbol_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            remove_sheet_map_value(sheet_value, "symbols", *symbol_id)?;
            Ok(true)
        }
        Operation::CreateSchematicWire { .. } | Operation::DeleteSchematicWire { .. } => Ok(false),
        Operation::SetSchematicSheetName { .. } => Ok(false),
        Operation::CreateSchematicJunction { .. } | Operation::DeleteSchematicJunction { .. } => {
            Ok(false)
        }
        Operation::CreateSchematicNoConnect { .. } | Operation::DeleteSchematicNoConnect { .. } => {
            Ok(false)
        }
        Operation::CreateSchematicLabel { .. }
        | Operation::SetSchematicLabel { .. }
        | Operation::DeleteSchematicLabel { .. } => Ok(false),
        Operation::CreateSchematicPort { .. }
        | Operation::SetSchematicPort { .. }
        | Operation::DeleteSchematicPort { .. } => Ok(false),
        Operation::CreateSchematicBus { .. }
        | Operation::SetSchematicBus { .. }
        | Operation::DeleteSchematicBus { .. }
        | Operation::CreateSchematicBusEntry { .. }
        | Operation::DeleteSchematicBusEntry { .. } => Ok(false),
        Operation::CreateSchematicText { .. }
        | Operation::SetSchematicText { .. }
        | Operation::DeleteSchematicText { .. } => Ok(false),
        Operation::CreateSchematicDrawing { .. }
        | Operation::SetSchematicDrawing { .. }
        | Operation::DeleteSchematicDrawing { .. } => Ok(false),
        Operation::CreateSchematicSymbol { .. }
        | Operation::SetSchematicSymbol { .. }
        | Operation::DeleteSchematicSymbol { .. } => Ok(false),
        _ => Ok(false),
    }
}

pub(super) fn inverse_schematic_sheet_operation(
    sheet_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetSchematicSheetName { sheet_id, name }
            if sheet_uuid(sheet_value) == Some(*sheet_id) =>
        {
            let previous = sheet_name(sheet_value)?.to_string();
            inverse_operations.push(Operation::SetSchematicSheetName {
                sheet_id: *sheet_id,
                name: previous,
            });
            set_sheet_name(sheet_value, name)?;
            Ok(true)
        }
        Operation::CreateSchematicWire {
            sheet_id,
            wire_id,
            wire,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicWire {
                sheet_id: *sheet_id,
                wire_id: *wire_id,
                wire: wire.clone(),
            });
            insert_sheet_map_value(sheet_value, "wires", *wire_id, wire.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicWire {
            sheet_id, wire_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "wires", *wire_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicWire {
                sheet_id: *sheet_id,
                wire_id: *wire_id,
                wire: previous,
            });
            remove_sheet_map_value(sheet_value, "wires", *wire_id)?;
            Ok(true)
        }
        Operation::CreateSchematicJunction {
            sheet_id,
            junction_id,
            junction,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicJunction {
                sheet_id: *sheet_id,
                junction_id: *junction_id,
                junction: junction.clone(),
            });
            insert_sheet_map_value(sheet_value, "junctions", *junction_id, junction.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "junctions", *junction_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicJunction {
                sheet_id: *sheet_id,
                junction_id: *junction_id,
                junction: previous,
            });
            remove_sheet_map_value(sheet_value, "junctions", *junction_id)?;
            Ok(true)
        }
        Operation::CreateSchematicNoConnect {
            sheet_id,
            noconnect_id,
            noconnect,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicNoConnect {
                sheet_id: *sheet_id,
                noconnect_id: *noconnect_id,
                noconnect: noconnect.clone(),
            });
            insert_sheet_map_value(sheet_value, "noconnects", *noconnect_id, noconnect.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicNoConnect {
            sheet_id,
            noconnect_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "noconnects", *noconnect_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicNoConnect {
                sheet_id: *sheet_id,
                noconnect_id: *noconnect_id,
                noconnect: previous,
            });
            remove_sheet_map_value(sheet_value, "noconnects", *noconnect_id)?;
            Ok(true)
        }
        Operation::CreateSchematicLabel {
            sheet_id,
            label_id,
            label,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicLabel {
                sheet_id: *sheet_id,
                label_id: *label_id,
                label: label.clone(),
            });
            insert_sheet_map_value(sheet_value, "labels", *label_id, label.clone())?;
            Ok(true)
        }
        Operation::SetSchematicLabel {
            sheet_id,
            label_id,
            label,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "labels", *label_id)?.clone();
            inverse_operations.push(Operation::SetSchematicLabel {
                sheet_id: *sheet_id,
                label_id: *label_id,
                label: previous,
            });
            insert_sheet_map_value(sheet_value, "labels", *label_id, label.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicLabel {
            sheet_id, label_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "labels", *label_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicLabel {
                sheet_id: *sheet_id,
                label_id: *label_id,
                label: previous,
            });
            remove_sheet_map_value(sheet_value, "labels", *label_id)?;
            Ok(true)
        }
        Operation::CreateSchematicPort {
            sheet_id,
            port_id,
            port,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicPort {
                sheet_id: *sheet_id,
                port_id: *port_id,
                port: port.clone(),
            });
            insert_sheet_map_value(sheet_value, "ports", *port_id, port.clone())?;
            Ok(true)
        }
        Operation::SetSchematicPort {
            sheet_id,
            port_id,
            port,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "ports", *port_id)?.clone();
            inverse_operations.push(Operation::SetSchematicPort {
                sheet_id: *sheet_id,
                port_id: *port_id,
                port: previous,
            });
            insert_sheet_map_value(sheet_value, "ports", *port_id, port.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicPort {
            sheet_id, port_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "ports", *port_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicPort {
                sheet_id: *sheet_id,
                port_id: *port_id,
                port: previous,
            });
            remove_sheet_map_value(sheet_value, "ports", *port_id)?;
            Ok(true)
        }
        Operation::CreateSchematicBus {
            sheet_id,
            bus_id,
            bus,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicBus {
                sheet_id: *sheet_id,
                bus_id: *bus_id,
                bus: bus.clone(),
            });
            insert_sheet_map_value(sheet_value, "buses", *bus_id, bus.clone())?;
            Ok(true)
        }
        Operation::SetSchematicBus {
            sheet_id,
            bus_id,
            bus,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "buses", *bus_id)?.clone();
            inverse_operations.push(Operation::SetSchematicBus {
                sheet_id: *sheet_id,
                bus_id: *bus_id,
                bus: previous,
            });
            insert_sheet_map_value(sheet_value, "buses", *bus_id, bus.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicBus {
            sheet_id, bus_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "buses", *bus_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicBus {
                sheet_id: *sheet_id,
                bus_id: *bus_id,
                bus: previous,
            });
            remove_sheet_map_value(sheet_value, "buses", *bus_id)?;
            Ok(true)
        }
        Operation::CreateSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            bus_entry,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicBusEntry {
                sheet_id: *sheet_id,
                bus_entry_id: *bus_entry_id,
                bus_entry: bus_entry.clone(),
            });
            insert_sheet_map_value(sheet_value, "bus_entries", *bus_entry_id, bus_entry.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "bus_entries", *bus_entry_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicBusEntry {
                sheet_id: *sheet_id,
                bus_entry_id: *bus_entry_id,
                bus_entry: previous,
            });
            remove_sheet_map_value(sheet_value, "bus_entries", *bus_entry_id)?;
            Ok(true)
        }
        Operation::CreateSchematicText {
            sheet_id,
            text_id,
            text,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicText {
                sheet_id: *sheet_id,
                text_id: *text_id,
                text: text.clone(),
            });
            insert_sheet_map_value(sheet_value, "texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::SetSchematicText {
            sheet_id,
            text_id,
            text,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "texts", *text_id)?.clone();
            inverse_operations.push(Operation::SetSchematicText {
                sheet_id: *sheet_id,
                text_id: *text_id,
                text: previous,
            });
            insert_sheet_map_value(sheet_value, "texts", *text_id, text.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicText {
            sheet_id, text_id, ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "texts", *text_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicText {
                sheet_id: *sheet_id,
                text_id: *text_id,
                text: previous,
            });
            remove_sheet_map_value(sheet_value, "texts", *text_id)?;
            Ok(true)
        }
        Operation::CreateSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicDrawing {
                sheet_id: *sheet_id,
                drawing_id: *drawing_id,
                drawing: drawing.clone(),
            });
            insert_sheet_map_value(sheet_value, "drawings", *drawing_id, drawing.clone())?;
            Ok(true)
        }
        Operation::SetSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "drawings", *drawing_id)?.clone();
            inverse_operations.push(Operation::SetSchematicDrawing {
                sheet_id: *sheet_id,
                drawing_id: *drawing_id,
                drawing: previous,
            });
            insert_sheet_map_value(sheet_value, "drawings", *drawing_id, drawing.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicDrawing {
            sheet_id,
            drawing_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "drawings", *drawing_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicDrawing {
                sheet_id: *sheet_id,
                drawing_id: *drawing_id,
                drawing: previous,
            });
            remove_sheet_map_value(sheet_value, "drawings", *drawing_id)?;
            Ok(true)
        }
        Operation::CreateSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            inverse_operations.push(Operation::DeleteSchematicSymbol {
                sheet_id: *sheet_id,
                symbol_id: *symbol_id,
                symbol: symbol.clone(),
            });
            insert_sheet_map_value(sheet_value, "symbols", *symbol_id, symbol.clone())?;
            Ok(true)
        }
        Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "symbols", *symbol_id)?.clone();
            inverse_operations.push(Operation::SetSchematicSymbol {
                sheet_id: *sheet_id,
                symbol_id: *symbol_id,
                symbol: previous,
            });
            insert_sheet_map_value(sheet_value, "symbols", *symbol_id, symbol.clone())?;
            Ok(true)
        }
        Operation::DeleteSchematicSymbol {
            sheet_id,
            symbol_id,
            ..
        } if sheet_uuid(sheet_value) == Some(*sheet_id) => {
            let previous = sheet_map_value(sheet_value, "symbols", *symbol_id)?.clone();
            inverse_operations.push(Operation::CreateSchematicSymbol {
                sheet_id: *sheet_id,
                symbol_id: *symbol_id,
                symbol: previous,
            });
            remove_sheet_map_value(sheet_value, "symbols", *symbol_id)?;
            Ok(true)
        }
        Operation::CreateSchematicWire { .. } | Operation::DeleteSchematicWire { .. } => Ok(false),
        Operation::SetSchematicSheetName { .. } => Ok(false),
        Operation::CreateSchematicJunction { .. } | Operation::DeleteSchematicJunction { .. } => {
            Ok(false)
        }
        Operation::CreateSchematicNoConnect { .. } | Operation::DeleteSchematicNoConnect { .. } => {
            Ok(false)
        }
        Operation::CreateSchematicLabel { .. }
        | Operation::SetSchematicLabel { .. }
        | Operation::DeleteSchematicLabel { .. } => Ok(false),
        Operation::CreateSchematicPort { .. }
        | Operation::SetSchematicPort { .. }
        | Operation::DeleteSchematicPort { .. } => Ok(false),
        Operation::CreateSchematicBus { .. }
        | Operation::SetSchematicBus { .. }
        | Operation::DeleteSchematicBus { .. }
        | Operation::CreateSchematicBusEntry { .. }
        | Operation::DeleteSchematicBusEntry { .. } => Ok(false),
        Operation::CreateSchematicText { .. }
        | Operation::SetSchematicText { .. }
        | Operation::DeleteSchematicText { .. } => Ok(false),
        Operation::CreateSchematicDrawing { .. }
        | Operation::SetSchematicDrawing { .. }
        | Operation::DeleteSchematicDrawing { .. } => Ok(false),
        Operation::CreateSchematicSymbol { .. }
        | Operation::SetSchematicSymbol { .. }
        | Operation::DeleteSchematicSymbol { .. } => Ok(false),
        _ => Ok(false),
    }
}

fn sheet_name(sheet_value: &serde_json::Value) -> Result<&str, EngineError> {
    sheet_value
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("schematic sheet missing name".to_string()))
}

fn set_sheet_name(sheet_value: &mut serde_json::Value, name: &str) -> Result<(), EngineError> {
    if name.trim().is_empty() {
        return Err(EngineError::Validation(
            "schematic sheet name must not be empty".to_string(),
        ));
    }
    let object = sheet_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("schematic sheet is not an object".to_string()))?;
    object.insert(
        "name".to_string(),
        serde_json::Value::String(name.trim().to_string()),
    );
    Ok(())
}
