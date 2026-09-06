use super::preferences_daemon::authorize_mcp_through_daemon;
use super::*;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::os::fd::AsRawFd;

use eda_engine::preferences::repository::GenerationRef;
use eda_engine::preferences::{
    AuthorizeMcpPreferenceApplyV1, GlobalPreferencesProductService, HeadExpectationV1,
    InstalledPreferenceLocationProvider, PreferenceActorKindV1, PreferenceActorV1,
    PreferenceMutationRequestV1, PreferenceProductPayloadV1, PreferenceProductRequestV1,
    PreferenceProductResponseV1, PreferenceProposalActionV1, PreferenceProposalV1,
    PreferenceQueryV1, PreferenceSchemaRefV1, ProjectUnitsSourceV1,
};

pub(crate) fn execute_preferences_command(
    format: &OutputFormat,
    action: PreferencesCommands,
) -> Result<(String, i32)> {
    let action = match action {
        PreferencesCommands::Proposal {
            action:
                PreferencesProposalCommands::AuthorizeMcp {
                    proposal_json,
                    mcp_session,
                },
        } => return execute_mcp_authorization(format, &proposal_json, mcp_session),
        action => action,
    };
    let mut service = GlobalPreferencesProductService::open(
        &InstalledPreferenceLocationProvider,
        format!("datum-cli-{}", std::process::id()),
    )
    .map_err(|error| anyhow::anyhow!("open Global Preferences service: {error:?}"))?;
    let session_id = format!("cli:{}:{}", std::process::id(), Uuid::new_v4());

    let response = match action {
        PreferencesCommands::Describe => execute_query(
            &mut service,
            "datum.preferences.describe",
            PreferenceQueryV1::Describe,
            &script_actor(&session_id),
        ),
        PreferencesCommands::List { section } => execute_query(
            &mut service,
            "datum.preferences.list",
            PreferenceQueryV1::List { section },
            &script_actor(&session_id),
        ),
        PreferencesCommands::Get { key } => execute_query(
            &mut service,
            "datum.preferences.get",
            PreferenceQueryV1::Get { key },
            &script_actor(&session_id),
        ),
        PreferencesCommands::Search { query } => execute_query(
            &mut service,
            "datum.preferences.search",
            PreferenceQueryV1::Search { query },
            &script_actor(&session_id),
        ),
        PreferencesCommands::Explain { key } => execute_query(
            &mut service,
            "datum.preferences.explain",
            PreferenceQueryV1::Explain { key },
            &script_actor(&session_id),
        ),
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
            execute_query(
                &mut service,
                "datum.preferences.preview_project_units_seed",
                PreferenceQueryV1::PreviewProjectUnitsSeed { source },
                &script_actor(&session_id),
            )
        }
        PreferencesCommands::Set {
            key,
            value_json,
            expected,
            reason,
            request_id,
        } => {
            let value: serde_json::Value = serde_json::from_str(&value_json)
                .context("--value-json must be one complete JSON value")?;
            let mutation = PreferenceMutationRequestV1::SetUser {
                key: key.clone(),
                value: value.clone(),
                expected: parse_expectation(&expected)?,
                request_id,
                reason,
            };
            execute_confirmed_mutation(
                &mut service,
                "datum.preferences.set",
                mutation,
                &key,
                Some(&value),
                &session_id,
            )
        }
        PreferencesCommands::Reset {
            key,
            expected,
            reason,
            request_id,
        } => {
            let mutation = PreferenceMutationRequestV1::ResetUser {
                key: key.clone(),
                expected: parse_expectation(&expected)?,
                request_id,
                reason,
            };
            execute_confirmed_mutation(
                &mut service,
                "datum.preferences.reset",
                mutation,
                &key,
                None,
                &session_id,
            )
        }
        PreferencesCommands::Proposal { action } => {
            execute_proposal(&mut service, action, &session_id)?
        }
    };
    Ok((
        render_output(format, &response),
        if response.ok { 0 } else { 2 },
    ))
}

fn execute_query(
    service: &mut GlobalPreferencesProductService,
    schema: &str,
    query: PreferenceQueryV1,
    actor: &PreferenceActorV1,
) -> PreferenceProductResponseV1 {
    service.execute(
        product_request(schema, PreferenceProductPayloadV1::Query(query)),
        actor,
    )
}

fn execute_confirmed_mutation(
    service: &mut GlobalPreferencesProductService,
    schema: &str,
    mutation: PreferenceMutationRequestV1,
    key: &str,
    proposed: Option<&serde_json::Value>,
    session_id: &str,
) -> PreferenceProductResponseV1 {
    let inspected = execute_query(
        service,
        "datum.preferences.get",
        PreferenceQueryV1::Get {
            key: key.to_owned(),
        },
        &script_actor(session_id),
    );
    let summary = match inspected.result {
        Some(eda_engine::preferences::PreferenceProductResultV1::Query(
            eda_engine::preferences::PreferenceQueryResultV1::Get(result),
        )) => format!(
            "Global · this device\nSetting: {}\nCurrent: {}\nProposed: {}\nTakes effect: {}",
            result.value.label,
            result
                .value
                .effective_value
                .map_or_else(|| "unresolved".to_owned(), |value| value.to_string()),
            proposed.map_or_else(
                || "Reset to inherited/default authority".to_owned(),
                serde_json::Value::to_string
            ),
            result.value.effect_timing,
        ),
        _ => {
            return match inspected.error {
                Some(error) => product_error_response(schema, error),
                None => invalid_product_response(service, schema),
            };
        }
    };
    if !confirm_on_foreground_tty(&summary) {
        return human_presence_refusal(service, schema, proposed.cloned(), None);
    }
    service.execute(
        product_request(schema, PreferenceProductPayloadV1::Mutation(mutation)),
        &human_actor(session_id),
    )
}

fn execute_proposal(
    service: &mut GlobalPreferencesProductService,
    action: PreferencesProposalCommands,
    session_id: &str,
) -> Result<PreferenceProductResponseV1> {
    let (schema, proposal_action, human_confirmation) = match action {
        PreferencesProposalCommands::Prepare {
            request_json,
            rationale,
        } => (
            "datum.preferences.proposal.prepare",
            PreferenceProposalActionV1::Prepare {
                mutation: read_json(&request_json)?,
                rationale,
            },
            false,
        ),
        PreferencesProposalCommands::Validate { proposal_json } => (
            "datum.preferences.proposal.validate",
            PreferenceProposalActionV1::Validate {
                proposal: read_json(&proposal_json)?,
            },
            false,
        ),
        PreferencesProposalCommands::AcceptApply { proposal_json } => (
            "datum.preferences.proposal.accept_apply",
            PreferenceProposalActionV1::AcceptAndApply {
                proposal: read_json(&proposal_json)?,
            },
            true,
        ),
        PreferencesProposalCommands::Reject { proposal_json } => (
            "datum.preferences.proposal.reject",
            PreferenceProposalActionV1::Reject {
                proposal: read_json(&proposal_json)?,
            },
            false,
        ),
        PreferencesProposalCommands::AuthorizeMcp { .. } => {
            unreachable!("authorize-mcp is routed to the owning daemon")
        }
    };
    let actor = if human_confirmation {
        let proposal = match &proposal_action {
            PreferenceProposalActionV1::AcceptAndApply { proposal } => proposal,
            _ => unreachable!(),
        };
        let summary = format!(
            "Global · this device\nProposal: {}\nMutation: {}\nRationale: {}",
            proposal.proposal_id,
            serde_json::to_string(&proposal.mutation)?,
            proposal.rationale,
        );
        if !confirm_on_foreground_tty(&summary) {
            return Ok(human_presence_refusal(
                service,
                schema,
                None,
                serde_json::to_value(proposal).ok(),
            ));
        }
        human_actor(session_id)
    } else {
        script_actor(session_id)
    };
    Ok(service.execute(
        product_request(
            schema,
            PreferenceProductPayloadV1::Proposal(proposal_action),
        ),
        &actor,
    ))
}

fn execute_mcp_authorization(
    format: &OutputFormat,
    proposal_path: &Path,
    mcp_session: String,
) -> Result<(String, i32)> {
    let proposal: PreferenceProposalV1 = read_json(proposal_path)?;
    let summary = format!(
        "Global · this device\nAuthorize MCP session: {}\nProposal: {}\nMutation: {}\nRationale: {}",
        mcp_session,
        proposal.proposal_id,
        serde_json::to_string(&proposal.mutation)?,
        proposal.rationale,
    );
    if !confirm_on_foreground_tty(&summary) {
        anyhow::bail!("foreground /dev/tty confirmation is required");
    }
    let actor = human_actor(&format!("cli:{}:{}", std::process::id(), Uuid::new_v4()));
    let result = authorize_mcp_through_daemon(
        AuthorizeMcpPreferenceApplyV1 {
            proposal_id: proposal.proposal_id,
            proposal_digest: proposal.proposal_digest,
            originating_mcp_session: mcp_session,
        },
        actor,
    )?;
    Ok((render_output(format, &result), 0))
}

fn product_request(
    schema: &str,
    payload: PreferenceProductPayloadV1,
) -> PreferenceProductRequestV1 {
    PreferenceProductRequestV1 {
        schema: PreferenceSchemaRefV1 {
            name: schema.to_owned(),
            version: 1,
        },
        payload,
    }
}

fn parse_expectation(value: &str) -> Result<HeadExpectationV1> {
    serde_json::from_str(value).context("--expected must be a full HeadExpectationV1 JSON value")
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    if path.as_os_str() == "-" {
        serde_json::from_reader(std::io::stdin().lock()).context("read JSON from stdin")
    } else {
        let file = std::fs::File::open(path)
            .with_context(|| format!("open JSON input {}", path.display()))?;
        serde_json::from_reader(file)
            .with_context(|| format!("parse JSON input {}", path.display()))
    }
}

fn script_actor(session_id: &str) -> PreferenceActorV1 {
    actor(PreferenceActorKindV1::ScriptAgent, session_id)
}
fn human_actor(session_id: &str) -> PreferenceActorV1 {
    actor(PreferenceActorKindV1::HumanCli, session_id)
}
fn actor(kind: PreferenceActorKindV1, session_id: &str) -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind,
        session_id: session_id.to_owned(),
        local_actor_id: "unavailable".to_owned(),
        invocation_id: Uuid::new_v4(),
    }
}

fn confirm_on_foreground_tty(summary: &str) -> bool {
    let Ok(mut tty) = OpenOptions::new().read(true).write(true).open("/dev/tty") else {
        return false;
    };
    if !tty.is_terminal() || !owns_foreground_tty(tty.as_raw_fd()) {
        return false;
    }
    if writeln!(tty, "{summary}\nType APPLY to confirm:").is_err() || tty.flush().is_err() {
        return false;
    }
    let mut line = String::new();
    BufReader::new(tty).read_line(&mut line).is_ok() && line.trim() == "APPLY"
}

#[cfg(unix)]
fn owns_foreground_tty(fd: std::os::raw::c_int) -> bool {
    unsafe extern "C" {
        fn tcgetpgrp(fd: std::os::raw::c_int) -> std::os::raw::c_int;
        fn getpgrp() -> std::os::raw::c_int;
    }
    // SAFETY: both calls consume only a live integer descriptor/process and retain no pointers.
    unsafe {
        let foreground = tcgetpgrp(fd);
        foreground >= 0 && foreground == getpgrp()
    }
}

#[cfg(not(unix))]
fn owns_foreground_tty(_fd: std::os::raw::c_int) -> bool {
    false
}

fn human_presence_refusal(
    service: &GlobalPreferencesProductService,
    schema: &str,
    preserved_draft: Option<serde_json::Value>,
    preserved_proposal: Option<serde_json::Value>,
) -> PreferenceProductResponseV1 {
    product_error_response(
        schema,
        eda_engine::preferences::PreferenceErrorV1 {
            code: eda_engine::preferences::PreferenceErrorCodeV1::HumanPresenceRequired,
            message: "Foreground /dev/tty confirmation is required".to_owned(),
            details: Default::default(),
            current_context: service.context(),
            preserved_draft,
            preserved_proposal,
        },
    )
}

fn product_error_response(
    schema: &str,
    error: eda_engine::preferences::PreferenceErrorV1,
) -> PreferenceProductResponseV1 {
    PreferenceProductResponseV1 {
        ok: false,
        schema: PreferenceSchemaRefV1 {
            name: schema.to_owned(),
            version: 1,
        },
        context: error.current_context.clone(),
        result: None,
        error: Some(error),
    }
}

fn invalid_product_response(
    service: &GlobalPreferencesProductService,
    schema: &str,
) -> PreferenceProductResponseV1 {
    product_error_response(
        schema,
        eda_engine::preferences::PreferenceErrorV1 {
            code: eda_engine::preferences::PreferenceErrorCodeV1::InvalidRequest,
            message: "Preference inspection returned an unexpected result".to_owned(),
            details: Default::default(),
            current_context: service.context(),
            preserved_draft: None,
            preserved_proposal: None,
        },
    )
}
