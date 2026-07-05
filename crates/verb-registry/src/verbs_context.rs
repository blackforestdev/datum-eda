//! The `datum.context` verb family (4 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_datum.py` context schemas), the
//! Python bridge argv builders (`server_runtime.py`), and cross-checked
//! against the clap definitions in `crates/cli/src/args/context.rs`.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const SESSION: ParamSpec = ParamSpec {
    name: "session",
    ty: ParamType::Str,
    required: false,
    doc: "Expected terminal/session id; rejects mismatched discovery envelopes",
    default_json: None,
};

const PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: false,
    doc: "Explicit context/discovery JSON path",
    default_json: None,
};

const PROJECT_ROOT: ParamSpec = ParamSpec {
    name: "project_root",
    ty: ParamType::Str,
    required: false,
    doc: "Project root containing .datum/gui-terminal-context.json",
    default_json: None,
};

const CONTEXT_ENVELOPE_SUMMARY: &str = "Return the current Datum session/context envelope, including project identity, model revision, actor type, capabilities, visible artifacts/check runs, provenance seed, and refresh metadata.";

const CONTEXT_ENVELOPE_PARAMS: &[ParamSpec] = &[SESSION, PATH, PROJECT_ROOT];

const SESSION_EVENT_FILTER_PARAMS: &[ParamSpec] = &[
    SESSION,
    PATH,
    PROJECT_ROOT,
    ParamSpec {
        name: "event_kind",
        ty: ParamType::Str,
        required: false,
        doc: "Exact-match filter for the JSONL event kind field",
        default_json: None,
    },
    ParamSpec {
        name: "origin",
        ty: ParamType::Str,
        required: false,
        doc: "Exact-match filter for the event origin field",
        default_json: None,
    },
    ParamSpec {
        name: "command_id",
        ty: ParamType::Str,
        required: false,
        doc: "Exact-match filter for the event command_id field",
        default_json: None,
    },
    ParamSpec {
        name: "execution_id",
        ty: ParamType::Str,
        required: false,
        doc: "Exact-match filter for the event execution_id field",
        default_json: None,
    },
    ParamSpec {
        name: "limit",
        ty: ParamType::Int,
        required: false,
        doc: "Return only the newest N matching events",
        default_json: None,
    },
];

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.context.get",
        summary: CONTEXT_ENVELOPE_SUMMARY,
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "datum_context_get",
            argv: &[
                ArgvToken::Lit("context"),
                ArgvToken::Lit("get"),
                ArgvToken::Flag { flag: "--session", param: "session" },
                ArgvToken::Flag { flag: "--path", param: "path" },
                ArgvToken::Flag { flag: "--project-root", param: "project_root" },
            ],
        },
        params: CONTEXT_ENVELOPE_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.context.refresh",
        summary: CONTEXT_ENVELOPE_SUMMARY,
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "datum_context_refresh",
            argv: &[
                ArgvToken::Lit("context"),
                ArgvToken::Lit("refresh"),
                ArgvToken::Flag { flag: "--session", param: "session" },
                ArgvToken::Flag { flag: "--path", param: "path" },
                ArgvToken::Flag { flag: "--project-root", param: "project_root" },
            ],
        },
        params: CONTEXT_ENVELOPE_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.context.session_activity",
        summary: "Return a compact Datum tool-session activity summary for a terminal/session. The primary agent-facing result is executions[], with start/end/duration, lifecycle/exit status, and per-execution I/O totals/previews. Results can be filtered by event kind, origin, command id, or execution id.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "datum_context_session_activity",
            argv: &[
                ArgvToken::Lit("context"),
                ArgvToken::Lit("session-activity"),
                ArgvToken::Flag { flag: "--session", param: "session" },
                ArgvToken::Flag { flag: "--path", param: "path" },
                ArgvToken::Flag { flag: "--project-root", param: "project_root" },
                ArgvToken::Flag { flag: "--event-kind", param: "event_kind" },
                ArgvToken::Flag { flag: "--origin", param: "origin" },
                ArgvToken::Flag { flag: "--command-id", param: "command_id" },
                ArgvToken::Flag { flag: "--execution-id", param: "execution_id" },
                ArgvToken::Flag { flag: "--limit", param: "limit" },
            ],
        },
        params: SESSION_EVENT_FILTER_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.context.session_events",
        summary: "Return recorded Datum tool-session events for a terminal/session, optionally filtered by event kind, origin, command id, or execution id.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "datum_context_session_events",
            argv: &[
                ArgvToken::Lit("context"),
                ArgvToken::Lit("session-events"),
                ArgvToken::Flag { flag: "--session", param: "session" },
                ArgvToken::Flag { flag: "--path", param: "path" },
                ArgvToken::Flag { flag: "--project-root", param: "project_root" },
                ArgvToken::Flag { flag: "--event-kind", param: "event_kind" },
                ArgvToken::Flag { flag: "--origin", param: "origin" },
                ArgvToken::Flag { flag: "--command-id", param: "command_id" },
                ArgvToken::Flag { flag: "--execution-id", param: "execution_id" },
                ArgvToken::Flag { flag: "--limit", param: "limit" },
            ],
        },
        params: SESSION_EVENT_FILTER_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
