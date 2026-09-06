//! Product-facing `project new` adapter for the GP-CM03 genesis service.

use super::*;
use eda_engine::preferences::repository::GenerationRef;
use eda_engine::preferences::{
    GlobalPreferencesProductService, InstalledPreferenceLocationProvider, PreferenceActorKindV1,
    PreferenceActorV1, PreferenceErrorCodeV1, PreferenceErrorV1, ProjectGenesisRequestV1,
    ProjectGenesisResultV1, ProjectUnitsSourceV1,
};

impl ProjectNewArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let effective_format = if self.json {
            OutputFormat::Json
        } else {
            format.clone()
        };
        match execute(self) {
            Ok(result) => Ok((render_success(&effective_format, &result), 0)),
            Err(error) => Ok((render_refusal(&effective_format, &error), 2)),
        }
    }
}

fn execute(args: ProjectNewArgs) -> Result<ProjectGenesisResultV1, PreferenceErrorV1> {
    let name = args.name.unwrap_or_else(|| {
        args.path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_owned()
    });
    let writer = format!("datum-cli-{}", std::process::id());
    let service = match args.units_source {
        ProjectUnitsSourceArg::Global => {
            GlobalPreferencesProductService::open(&InstalledPreferenceLocationProvider, &writer)?
        }
        ProjectUnitsSourceArg::Factory => GlobalPreferencesProductService::factory_only(&writer)?,
    };
    let units_source = match args.units_source {
        ProjectUnitsSourceArg::Global => ProjectUnitsSourceV1::Global {
            expected_generation: args
                .expected_preferences
                .as_deref()
                .map(serde_json::from_str::<GenerationRef>)
                .transpose()
                .map_err(|error| invalid_request(&service, error.to_string()))?,
        },
        ProjectUnitsSourceArg::Factory if args.expected_preferences.is_none() => {
            ProjectUnitsSourceV1::Factory {
                profile_id: "datum.units.factory.v1".to_owned(),
            }
        }
        ProjectUnitsSourceArg::Factory => {
            return Err(invalid_request(
                &service,
                "--expected-preferences is valid only with --units-source global".to_owned(),
            ));
        }
    };
    let actor = PreferenceActorV1 {
        kind: PreferenceActorKindV1::HumanCli,
        session_id: format!("cli:{}:{}", std::process::id(), Uuid::new_v4()),
        local_actor_id: std::env::var("USER").unwrap_or_else(|_| "unavailable".to_owned()),
        invocation_id: Uuid::new_v4(),
    };
    service.create_project(
        ProjectGenesisRequestV1 {
            request_id: args.request_id.unwrap_or_else(Uuid::new_v4),
            destination: args.path,
            project_name: name,
            project_id: args.project_id,
            units_source,
        },
        &actor,
    )
}

fn invalid_request(service: &GlobalPreferencesProductService, reason: String) -> PreferenceErrorV1 {
    PreferenceErrorV1 {
        code: PreferenceErrorCodeV1::InvalidRequest,
        message: "Project genesis request is invalid".to_owned(),
        details: BTreeMap::from([("reason".to_owned(), serde_json::json!(reason))]),
        current_context: service.context(),
        preserved_draft: None,
        preserved_proposal: None,
    }
}

fn render_success(format: &OutputFormat, result: &ProjectGenesisResultV1) -> String {
    match format {
        OutputFormat::Json => render_output(format, result),
        OutputFormat::Text => [
            format!("project_root: {}", result.project_root_identity),
            format!("project_id: {}", result.project_id),
            format!("request_id: {}", result.request_id),
            format!("genesis_request_digest: {}", result.genesis_request_digest),
            format!("units_source: {:?}", result.units_receipt.source),
            format!("units_seed_items: {}", result.units_receipt.items.len()),
        ]
        .join("\n"),
    }
}

fn render_refusal(format: &OutputFormat, error: &PreferenceErrorV1) -> String {
    match format {
        OutputFormat::Json => render_output(format, error),
        OutputFormat::Text => format!("error: {:?}: {}", error.code, error.message),
    }
}
