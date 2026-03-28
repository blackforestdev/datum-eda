#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativePortDirectionArg {
    Input,
    Output,
    Bidirectional,
    Passive,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeSymbolDisplayModeArg {
    LibraryDefault,
    ShowHiddenPins,
    HideOptionalPins,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeHiddenPowerBehaviorArg {
    SourceDefinedImplicit,
    ExplicitPowerObject,
    PreservedAsImportedMetadata,
}
