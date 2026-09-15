use clap::ValueEnum;

#[derive(Clone, ValueEnum)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum FailOn {
    Info,
    Warning,
    Error,
}
