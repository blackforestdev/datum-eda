use eda_engine::schematic::{HiddenPowerBehavior, SymbolDisplayMode};

use super::{NativeHiddenPowerBehaviorArg, NativeSymbolDisplayModeArg};

pub(super) fn parse_native_symbol_display_mode(
    value: NativeSymbolDisplayModeArg,
) -> SymbolDisplayMode {
    match value {
        NativeSymbolDisplayModeArg::LibraryDefault => SymbolDisplayMode::LibraryDefault,
        NativeSymbolDisplayModeArg::ShowHiddenPins => SymbolDisplayMode::ShowHiddenPins,
        NativeSymbolDisplayModeArg::HideOptionalPins => SymbolDisplayMode::HideOptionalPins,
    }
}

pub(super) fn parse_native_hidden_power_behavior(
    value: NativeHiddenPowerBehaviorArg,
) -> HiddenPowerBehavior {
    match value {
        NativeHiddenPowerBehaviorArg::SourceDefinedImplicit => {
            HiddenPowerBehavior::SourceDefinedImplicit
        }
        NativeHiddenPowerBehaviorArg::ExplicitPowerObject => {
            HiddenPowerBehavior::ExplicitPowerObject
        }
        NativeHiddenPowerBehaviorArg::PreservedAsImportedMetadata => {
            HiddenPowerBehavior::PreservedAsImportedMetadata
        }
    }
}
