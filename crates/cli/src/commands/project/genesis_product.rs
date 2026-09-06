//! Product-facing `project new` adapter for the GP-CM03 genesis service.

use super::*;
use eda_engine::preferences::repository::GenerationRef;
use eda_engine::preferences::{
    GlobalPreferencesProductService, InstalledPreferenceLocationProvider, PreferenceActorKindV1,
    PreferenceActorV1, PreferenceErrorCodeV1, PreferenceErrorV1, PreferenceSchemaRefV1,
    ProjectGenesisRequestV1, ProjectGenesisResponseV1, ProjectUnitsSourceV1,
};

impl ProjectNewArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let effective_format = if self.json {
            OutputFormat::Json
        } else {
            format.clone()
        };
        let response = execute(self);
        let code = if response.ok { 0 } else { 2 };
        Ok((render_response(&effective_format, &response), code))
    }
}

fn execute(args: ProjectNewArgs) -> ProjectGenesisResponseV1 {
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
            match GlobalPreferencesProductService::open(
                &InstalledPreferenceLocationProvider,
                &writer,
            ) {
                Ok(service) => service,
                Err(error) => return failure_response(error),
            }
        }
        ProjectUnitsSourceArg::Factory => {
            match GlobalPreferencesProductService::factory_only(&writer) {
                Ok(service) => service,
                Err(error) => return failure_response(error),
            }
        }
    };
    let units_source = match args.units_source {
        ProjectUnitsSourceArg::Global => ProjectUnitsSourceV1::Global {
            expected_generation: match args
                .expected_preferences
                .as_deref()
                .map(serde_json::from_str::<GenerationRef>)
                .transpose()
            {
                Ok(value) => value,
                Err(error) => {
                    return failure_response(invalid_request(&service, error.to_string()));
                }
            },
        },
        ProjectUnitsSourceArg::Factory if args.expected_preferences.is_none() => {
            ProjectUnitsSourceV1::Factory {
                profile_id: "datum.units.factory.v1".to_owned(),
            }
        }
        ProjectUnitsSourceArg::Factory => {
            return failure_response(invalid_request(
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
    service.execute_project_genesis(
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

fn failure_response(error: PreferenceErrorV1) -> ProjectGenesisResponseV1 {
    ProjectGenesisResponseV1 {
        ok: false,
        schema: PreferenceSchemaRefV1 {
            name: "datum.project.new".to_owned(),
            version: 1,
        },
        context: error.current_context.as_ref().clone(),
        result: None,
        error: Some(error),
    }
}

fn invalid_request(service: &GlobalPreferencesProductService, reason: String) -> PreferenceErrorV1 {
    PreferenceErrorV1 {
        code: PreferenceErrorCodeV1::InvalidRequest,
        message: "Project genesis request is invalid".to_owned(),
        details: Box::new(BTreeMap::from([(
            "reason".to_owned(),
            serde_json::json!(reason),
        )])),
        current_context: Box::new(service.context()),
        preserved_draft: None,
        preserved_proposal: None,
    }
}

fn render_response(format: &OutputFormat, response: &ProjectGenesisResponseV1) -> String {
    match format {
        OutputFormat::Json => render_output(format, response),
        OutputFormat::Text => match (&response.result, &response.error) {
            (Some(result), None) => [
                format!("project_root: {}", result.project_root_identity),
                format!("project_id: {}", result.project_id),
                format!("request_id: {}", result.request_id),
                format!("genesis_request_digest: {}", result.genesis_request_digest),
                format!("units_source: {:?}", result.units_receipt.source),
                format!("units_seed_items: {}", result.units_receipt.items.len()),
            ]
            .join("\n"),
            (None, Some(error)) => {
                let code = serde_json::to_value(&error.code)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "invalid_request".to_owned());
                format!("{code}: {}", error.message)
            }
            _ => "invalid_project_genesis_envelope".to_owned(),
        },
    }
}
