use super::preferences_daemon::authorize_mcp_through_daemon;
use super::*;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use eda_engine::preferences::repository::GenerationRef;
use eda_engine::preferences::{
    AuthorizeMcpPreferenceApplyV1, GlobalPreferencesProductService, HeadExpectationV1,
    InstalledPreferenceLocationProvider, PreferenceActorKindV1, PreferenceActorV1,
    PreferenceContextV1, PreferenceMutationRequestV1, PreferenceProductPayloadV1,
    PreferenceProductRequestV1, PreferenceProductResponseV1, PreferenceProposalActionV1,
    PreferenceProposalV1, PreferenceQueryV1, PreferenceSchemaRefV1, ProjectUnitsSourceV1,
};
use serde::Deserialize;

pub(crate) fn execute_preferences_command(
    format: &OutputFormat,
    action: PreferencesCommands,
) -> Result<(String, i32)> {
    // Host-authorization of an MCP apply has always reached the owning daemon.
    if let PreferencesCommands::Proposal {
        action:
            PreferencesProposalCommands::AuthorizeMcp {
                proposal_json,
                mcp_session,
            },
    } = action
    {
        return execute_mcp_authorization(format, &proposal_json, mcp_session);
    }

    let session_id = format!("cli:{}:{}", std::process::id(), Uuid::new_v4());

    // Proposal operations participate in owning-daemon routing like every other ordinary request
    // (§2.1). The committing `accept_apply` discovers a reachable owner and routes its commit
    // through the daemon's single writer -- never a local commit with an owner present; standalone
    // it commits locally, unchanged. The side-effect-free creation ops (prepare/validate/reject)
    // take no writer lease and stay on the local path.
    if let PreferencesCommands::Proposal { action } = action {
        let response = if matches!(action, PreferencesProposalCommands::AcceptApply { .. }) {
            Backend::discover()?.confirmed_proposal_apply(action, &session_id)?
        } else {
            execute_proposal(&mut open_local_service()?, action, &session_id)?
        };
        return Ok((
            render_output(format, &response),
            if response.ok { 0 } else { 2 },
        ));
    }

    // Ordinary query/Set/Reset: route through a reachable owning daemon; operate standalone ONLY
    // when no owner is reachable at initial discovery. Once an owner is selected there is no
    // fallback to a local writer.
    let mut backend = Backend::discover()?;

    let response = match action {
        PreferencesCommands::Describe => backend.query(
            "datum.preferences.describe",
            PreferenceQueryV1::Describe,
            &session_id,
        ),
        PreferencesCommands::List { section } => backend.query(
            "datum.preferences.list",
            PreferenceQueryV1::List { section },
            &session_id,
        ),
        PreferencesCommands::Get { key } => backend.query(
            "datum.preferences.get",
            PreferenceQueryV1::Get { key },
            &session_id,
        ),
        PreferencesCommands::Search { query } => backend.query(
            "datum.preferences.search",
            PreferenceQueryV1::Search { query },
            &session_id,
        ),
        PreferencesCommands::Explain { key } => backend.query(
            "datum.preferences.explain",
            PreferenceQueryV1::Explain { key },
            &session_id,
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
            backend.query(
                "datum.preferences.preview_project_units_seed",
                PreferenceQueryV1::PreviewProjectUnitsSeed { source },
                &session_id,
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
            backend.confirmed_mutation(
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
            backend.confirmed_mutation("datum.preferences.reset", mutation, &key, None, &session_id)
        }
        // AuthorizeMcp returned early; every other proposal variant took the local path above.
        PreferencesCommands::Proposal { .. } => unreachable!("proposal commands handled above"),
    }?;

    Ok((
        render_output(format, &response),
        if response.ok { 0 } else { 2 },
    ))
}

/// The ordinary-request execution target: a reachable owning daemon, or the in-process standalone
/// service when no owner is reachable at initial discovery.
enum Backend {
    /// A reachable owning daemon selected at initial discovery; every request is a typed RPC and
    /// there is no fallback to a local writer.
    Daemon { socket: PathBuf },
    /// No owner was reachable; the CLI holds the single local product service.
    Standalone {
        service: Box<GlobalPreferencesProductService>,
    },
}

impl Backend {
    /// Select the backend. A configured `DATUM_ENGINE_SOCKET`/`EDA_ENGINE_SOCKET` that accepts a
    /// connection selects that owning daemon. An absent socket or a connection refusal at this
    /// initial discovery is the ONLY thing that permits standalone operation; a later connection
    /// loss, RPC refusal, malformed reply or identity mismatch never selects a local writer.
    fn discover() -> Result<Self> {
        if let Some(socket) = owning_socket()
            && UnixStream::connect(&socket).is_ok()
        {
            return Ok(Backend::Daemon { socket });
        }
        Ok(Backend::Standalone {
            service: Box::new(open_local_service()?),
        })
    }

    /// Execute one typed product request, closing any daemon connection before returning so the
    /// serial socket server is never held across a human confirmation wait.
    fn run(
        &mut self,
        request: PreferenceProductRequestV1,
        actor: &PreferenceActorV1,
    ) -> Result<PreferenceProductResponseV1> {
        match self {
            Backend::Standalone { service } => Ok(service.execute(request, actor)),
            Backend::Daemon { socket } => execute_host_through_daemon(socket, request, actor),
        }
    }

    fn query(
        &mut self,
        schema: &str,
        query: PreferenceQueryV1,
        session_id: &str,
    ) -> Result<PreferenceProductResponseV1> {
        self.run(
            product_request(schema, PreferenceProductPayloadV1::Query(query)),
            &script_actor(session_id),
        )
    }

    fn confirmed_mutation(
        &mut self,
        schema: &str,
        mutation: PreferenceMutationRequestV1,
        key: &str,
        proposed: Option<&serde_json::Value>,
        session_id: &str,
    ) -> Result<PreferenceProductResponseV1> {
        // Inspect the current value for the confirmation summary. Any daemon connection for this
        // query is released before the APPLY prompt; the mutation reconnects afterward.
        let inspected = self.query(
            "datum.preferences.get",
            PreferenceQueryV1::Get {
                key: key.to_owned(),
            },
            session_id,
        )?;
        let context = inspected.context.clone();
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
                return Ok(match inspected.error {
                    Some(error) => product_error_response(schema, error),
                    None => invalid_product_response(context, schema),
                });
            }
        };
        if !confirm_on_foreground_tty(&summary) {
            return Ok(human_presence_refusal(
                context,
                schema,
                proposed.cloned(),
                None,
            ));
        }
        self.run(
            product_request(schema, PreferenceProductPayloadV1::Mutation(mutation)),
            &human_actor(session_id),
        )
    }

    /// The current product context, for a client-side refusal. Standalone reads it from the local
    /// service; against an owner it comes from a describe query (which also re-confirms the owner is
    /// reachable), its connection released before any confirmation wait.
    fn context(&mut self, session_id: &str) -> Result<PreferenceContextV1> {
        if let Backend::Standalone { service } = self {
            return Ok(service.context());
        }
        Ok(self
            .query(
                "datum.preferences.describe",
                PreferenceQueryV1::Describe,
                session_id,
            )?
            .context)
    }

    /// Apply a prepared proposal under a human's foreground-TTY confirmation. With a reachable owner
    /// the committing accept routes through the daemon's single writer (the request carries the
    /// AcceptAndApply mutation under the host route); standalone it commits through the local
    /// service. Never a local commit with an owner present. The TTY confirmation is performed
    /// client-side before the apply request is sent, and any describe connection is released first.
    fn confirmed_proposal_apply(
        &mut self,
        action: PreferencesProposalCommands,
        session_id: &str,
    ) -> Result<PreferenceProductResponseV1> {
        let PreferencesProposalCommands::AcceptApply { proposal_json } = action else {
            unreachable!("confirmed_proposal_apply handles accept_apply only")
        };
        let proposal: PreferenceProposalV1 = read_json(&proposal_json)?;
        let schema = "datum.preferences.proposal.accept_apply";
        let summary = format!(
            "Global · this device\nProposal: {}\nMutation: {}\nRationale: {}",
            proposal.proposal_id,
            serde_json::to_string(&proposal.mutation)?,
            proposal.rationale,
        );
        let context = self.context(session_id)?;
        if !confirm_on_foreground_tty(&summary) {
            return Ok(human_presence_refusal(
                context,
                schema,
                None,
                serde_json::to_value(&proposal).ok(),
            ));
        }
        self.run(
            product_request(
                schema,
                PreferenceProductPayloadV1::Proposal(Box::new(
                    PreferenceProposalActionV1::AcceptAndApply { proposal },
                )),
            ),
            &human_actor(session_id),
        )
    }
}

fn owning_socket() -> Option<PathBuf> {
    std::env::var_os("DATUM_ENGINE_SOCKET")
        .or_else(|| std::env::var_os("EDA_ENGINE_SOCKET"))
        .map(PathBuf::from)
}

fn open_local_service() -> Result<GlobalPreferencesProductService> {
    GlobalPreferencesProductService::open(
        &InstalledPreferenceLocationProvider,
        format!("datum-cli-{}", std::process::id()),
    )
    .map_err(|error| anyhow::anyhow!("open Global Preferences service: {error:?}"))
}

#[derive(Deserialize)]
struct HostDaemonResponse {
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<HostDaemonError>,
}

#[derive(Deserialize)]
struct HostDaemonError {
    code: i32,
    message: String,
}

/// Send one ordinary host product request to the selected owning daemon and return its typed
/// product envelope. The connection is opened and closed for this single request. A JSON-RPC error,
/// a missing/mismatched id, a malformed reply, or a product envelope answering a different schema
/// is surfaced as an error -- it NEVER falls back to a local writer. A product-level refusal (a
/// typed envelope with `ok == false`) is a successful reply and is returned as the command result.
fn execute_host_through_daemon(
    socket: &Path,
    request: PreferenceProductRequestV1,
    actor: &PreferenceActorV1,
) -> Result<PreferenceProductResponseV1> {
    let schema_name = request.schema.name.clone();
    let id = serde_json::json!(actor.invocation_id);
    let mut stream = UnixStream::connect(socket)
        .with_context(|| format!("connect owning Datum daemon at {}", socket.display()))?;
    let wire = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "preferences.host_product",
        "params": {"request": request, "actor": actor},
    });
    serde_json::to_writer(&mut stream, &wire).context("encode daemon preferences request")?;
    stream
        .write_all(b"\n")
        .context("send daemon preferences request")?;
    stream.flush().context("flush daemon preferences request")?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .context("read daemon preferences response")?;
    let response: HostDaemonResponse =
        serde_json::from_str(&line).context("decode daemon preferences response")?;
    if let Some(error) = response.error {
        anyhow::bail!(
            "owning daemon rejected the preferences request ({}): {}",
            error.code,
            error.message
        );
    }
    if response.id != Some(id) {
        anyhow::bail!("owning daemon reply did not match the request id");
    }
    let envelope: PreferenceProductResponseV1 = serde_json::from_value(
        response
            .result
            .context("daemon preferences response omitted result")?,
    )
    .context("decode typed daemon preferences envelope")?;
    if envelope.schema.name != schema_name {
        anyhow::bail!(
            "owning daemon answered a different schema: {} (requested {})",
            envelope.schema.name,
            schema_name
        );
    }
    Ok(envelope)
}

/// The side-effect-free proposal creation ops (prepare / validate / reject), which take no writer
/// lease and run as a script actor on the local service. The committing `accept_apply` is handled
/// by `Backend::confirmed_proposal_apply` (owner-routed or standalone), and `authorize-mcp` reaches
/// the owning daemon directly -- neither reaches here.
fn execute_proposal(
    service: &mut GlobalPreferencesProductService,
    action: PreferencesProposalCommands,
    session_id: &str,
) -> Result<PreferenceProductResponseV1> {
    let (schema, proposal_action) = match action {
        PreferencesProposalCommands::Prepare {
            request_json,
            rationale,
        } => (
            "datum.preferences.proposal.prepare",
            PreferenceProposalActionV1::Prepare {
                mutation: read_json(&request_json)?,
                rationale,
            },
        ),
        PreferencesProposalCommands::Validate { proposal_json } => (
            "datum.preferences.proposal.validate",
            PreferenceProposalActionV1::Validate {
                proposal: read_json(&proposal_json)?,
            },
        ),
        PreferencesProposalCommands::Reject { proposal_json } => (
            "datum.preferences.proposal.reject",
            PreferenceProposalActionV1::Reject {
                proposal: read_json(&proposal_json)?,
            },
        ),
        PreferencesProposalCommands::AcceptApply { .. } => {
            unreachable!("accept_apply is routed through Backend::confirmed_proposal_apply")
        }
        PreferencesProposalCommands::AuthorizeMcp { .. } => {
            unreachable!("authorize-mcp is routed to the owning daemon")
        }
    };
    Ok(service.execute(
        product_request(
            schema,
            PreferenceProductPayloadV1::Proposal(Box::new(proposal_action)),
        ),
        &script_actor(session_id),
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
    context: PreferenceContextV1,
    schema: &str,
    preserved_draft: Option<serde_json::Value>,
    preserved_proposal: Option<serde_json::Value>,
) -> PreferenceProductResponseV1 {
    product_error_response(
        schema,
        eda_engine::preferences::PreferenceErrorV1 {
            code: eda_engine::preferences::PreferenceErrorCodeV1::HumanPresenceRequired,
            message: "Foreground /dev/tty confirmation is required".to_owned(),
            details: Box::default(),
            current_context: Box::new(context),
            preserved_draft: preserved_draft.map(Box::new),
            preserved_proposal: preserved_proposal.map(Box::new),
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
        context: error.current_context.as_ref().clone(),
        result: None,
        error: Some(error),
    }
}

fn invalid_product_response(
    context: PreferenceContextV1,
    schema: &str,
) -> PreferenceProductResponseV1 {
    product_error_response(
        schema,
        eda_engine::preferences::PreferenceErrorV1 {
            code: eda_engine::preferences::PreferenceErrorCodeV1::InvalidRequest,
            message: "Preference inspection returned an unexpected result".to_owned(),
            details: Box::default(),
            current_context: Box::new(context),
            preserved_draft: None,
            preserved_proposal: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::thread;

    fn describe_request() -> PreferenceProductRequestV1 {
        product_request(
            "datum.preferences.describe",
            PreferenceProductPayloadV1::Query(PreferenceQueryV1::Describe),
        )
    }

    /// A one-shot mock owning daemon on a Unix socket that answers the single request with `reply`.
    /// When `reply`'s `id` is null the request id is echoed back (so the happy-id case matches).
    fn mock_owner(reply: serde_json::Value) -> (PathBuf, thread::JoinHandle<()>) {
        let root = std::env::temp_dir().join(format!("datum-host-route-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("daemon.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "preferences.host_product");
            let mut reply = reply;
            if reply.get("id").is_none_or(serde_json::Value::is_null) {
                reply["id"] = request["id"].clone();
            }
            let mut stream = stream;
            serde_json::to_writer(&mut stream, &reply).unwrap();
            stream.write_all(b"\n").unwrap();
        });
        (socket, handle)
    }

    #[test]
    fn host_route_surfaces_a_json_rpc_error_and_never_falls_back() {
        let (socket, handle) = mock_owner(serde_json::json!({
            "jsonrpc": "2.0", "id": null, "result": null,
            "error": {"code": -32601, "message": "method not found"},
        }));
        let outcome =
            execute_host_through_daemon(&socket, describe_request(), &script_actor("cli:test"));
        handle.join().unwrap();
        let error = outcome.expect_err("a daemon JSON-RPC error must surface, never fall back");
        assert!(error.to_string().contains("owning daemon rejected"));
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn host_route_rejects_a_reply_whose_id_does_not_match_the_request() {
        let (socket, handle) = mock_owner(serde_json::json!({
            "jsonrpc": "2.0", "id": "a-different-id", "result": {"ignored": true}, "error": null,
        }));
        let outcome =
            execute_host_through_daemon(&socket, describe_request(), &script_actor("cli:test"));
        handle.join().unwrap();
        let error = outcome.expect_err("a mismatched reply id must surface, never fall back");
        assert!(error.to_string().contains("did not match the request id"));
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn host_route_rejects_a_reply_answering_a_different_schema() {
        // A deserializable envelope (ok refusal needs no result) that answers a DIFFERENT schema
        // than requested: the identity check must reject it rather than trust a mismatched owner.
        let (socket, handle) = mock_owner(serde_json::json!({
            "jsonrpc": "2.0", "id": null, "error": null,
            "result": {
                "ok": false,
                "schema": {"name": "datum.preferences.get", "version": 1},
                "context": describe_context(),
                "result": null,
                "error": {
                    "code": "invalid_request", "message": "wrong schema", "details": {},
                    "current_context": describe_context(),
                    "preserved_draft": null, "preserved_proposal": null,
                },
            },
        }));
        let outcome =
            execute_host_through_daemon(&socket, describe_request(), &script_actor("cli:test"));
        handle.join().unwrap();
        let error =
            outcome.expect_err("a schema-mismatched envelope must surface, never fall back");
        assert!(error.to_string().contains("answered a different schema"));
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    /// The daemon's real context shape, so the schema-identity test's envelope deserializes.
    fn describe_context() -> serde_json::Value {
        serde_json::to_value(
            open_local_service_for_context()
                .map(|service| service.context())
                .expect("installed service context for the test envelope"),
        )
        .unwrap()
    }

    fn open_local_service_for_context() -> Result<GlobalPreferencesProductService> {
        let base = std::env::temp_dir().join(format!("datum-host-route-ctx-{}", Uuid::new_v4()));
        let provider = eda_engine::preferences::FixedPreferenceLocationProvider(
            eda_engine::preferences::PreferenceLocations {
                configuration_base: base.clone(),
                repository_root: base.join("preferences"),
                legacy_console_path: base.join("gui-preferences.json"),
            },
        );
        GlobalPreferencesProductService::open(&provider, "cli-test-context".to_owned())
            .map_err(|error| anyhow::anyhow!("open context service: {error:?}"))
    }
}
