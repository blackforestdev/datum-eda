use crate::schematic::PinElectricalType;

pub(crate) const PIN_ELECTRICAL_TAXONOMY_REVISION: &str = "LibraryPinElectricalType:v1";

pub(crate) fn canonical_pin_electrical_type_name(
    electrical_type: &PinElectricalType,
) -> &'static str {
    match electrical_type {
        PinElectricalType::Input => "input",
        PinElectricalType::Output => "output",
        PinElectricalType::Bidirectional => "bidirectional",
        PinElectricalType::Passive => "passive",
        PinElectricalType::PowerIn => "power_in",
        PinElectricalType::PowerOut => "power_out",
        PinElectricalType::OpenCollector => "open_collector",
        PinElectricalType::OpenEmitter => "open_emitter",
        PinElectricalType::TriState => "tri_state",
        PinElectricalType::NoConnect => "no_connect",
    }
}

pub(crate) fn is_conflicting_output(electrical_type: &PinElectricalType) -> bool {
    matches!(
        electrical_type,
        PinElectricalType::Output | PinElectricalType::PowerOut
    )
}

pub(crate) fn is_explicit_driver(electrical_type: &PinElectricalType) -> bool {
    matches!(
        electrical_type,
        PinElectricalType::Output
            | PinElectricalType::PowerOut
            | PinElectricalType::OpenCollector
            | PinElectricalType::OpenEmitter
            | PinElectricalType::TriState
    )
}

pub(crate) fn is_input(electrical_type: &PinElectricalType) -> bool {
    matches!(electrical_type, PinElectricalType::Input)
}

pub(crate) fn is_passive(electrical_type: &PinElectricalType) -> bool {
    matches!(electrical_type, PinElectricalType::Passive)
}

pub(crate) fn is_power_input(electrical_type: &PinElectricalType) -> bool {
    matches!(electrical_type, PinElectricalType::PowerIn)
}

pub(crate) fn is_no_connect(electrical_type: &PinElectricalType) -> bool {
    matches!(electrical_type, PinElectricalType::NoConnect)
}
