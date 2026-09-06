use super::*;

use eda_engine::preferences::repository::GenerationRef;
use eda_engine::preferences::{
    GlobalPreferencesProductService, InstalledPreferenceLocationProvider, PreferenceQueryV1,
    ProjectUnitsSourceV1,
};

pub(crate) fn execute_preferences_command(
    format: &OutputFormat,
    action: PreferencesCommands,
) -> Result<(String, i32)> {
    let service = GlobalPreferencesProductService::open(
        &InstalledPreferenceLocationProvider,
        format!("datum-cli-{}", std::process::id()),
    )
    .map_err(|error| anyhow::anyhow!("open Global Preferences service: {error:?}"))?;
    let query = match action {
        PreferencesCommands::Describe => PreferenceQueryV1::Describe,
        PreferencesCommands::List { section } => PreferenceQueryV1::List { section },
        PreferencesCommands::Get { key } => PreferenceQueryV1::Get { key },
        PreferencesCommands::Search { query } => PreferenceQueryV1::Search { query },
        PreferencesCommands::Explain { key } => PreferenceQueryV1::Explain { key },
        PreferencesCommands::PreviewProjectUnitsSeed { source, expected } => {
            let source = match source {
                PreferencesSeedSource::Global => ProjectUnitsSourceV1::Global {
                    expected_generation: expected
                        .map(|value| serde_json::from_str::<GenerationRef>(&value))
                        .transpose()
                        .context("--expected must be a full GenerationRef JSON value")?,
                },
                PreferencesSeedSource::Factory => ProjectUnitsSourceV1::Factory {
                    profile_id: "datum.units.factory.v1".to_owned(),
                },
            };
            PreferenceQueryV1::PreviewProjectUnitsSeed { source }
        }
    };
    match service.query(query) {
        Ok(result) => Ok((render_output(format, &result), 0)),
        Err(error) => Ok((
            render_output(format, &error),
            preference_error_exit(&error.code),
        )),
    }
}

fn preference_error_exit(code: &eda_engine::preferences::PreferenceErrorCodeV1) -> i32 {
    use eda_engine::preferences::PreferenceErrorCodeV1::*;
    match code {
        InvalidRequest
        | InvalidQuery
        | UnknownSection
        | UnknownPreferenceKey
        | ReservedPreferenceKey
        | InvalidPreferenceValue
        | IneligibleSource
        | GenerationRequired
        | UnsupportedSchemaVersion => 2,
        UnauthorizedActor
        | HumanPresenceRequired
        | ProposalRequired
        | MissingAcceptance
        | AcceptanceMismatch
        | AcceptanceExpired
        | AcceptanceConsumed => 3,
        StaleGeneration | ProposalStale | ProposalInvalid | IdempotencyConflict => 4,
        WriterConflict
        | RepositoryUnreadable
        | MigrationRequired
        | RepositoryIo
        | SeedSourceUnavailable
        | SeedIncomplete
        | ProjectTargetExists
        | GenesisPublishFailed => 5,
    }
}
