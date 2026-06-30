use crate::schematic::PinElectricalType;

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
