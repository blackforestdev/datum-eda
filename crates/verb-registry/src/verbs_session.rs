//! The `datum.session` verb family (4 verbs), transcribed from the
//! hand-written legacy-canonical alias dicts (`tools_catalog_data.py`
//! `_LEGACY_FLAT_TOOL_SPECS` via `tools_catalog_legacy_aliases.py`).
//!
//! `open`/`close`/`save` bridge the one-time imported converter session and
//! dispatch straight to daemon JSON-RPC methods; `validate` is CLI-bridged
//! like `datum.project.validate` (`server_runtime.py`).
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.session.close",
        summary: "Close the current in-memory project session.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc { method: "close_project" },
        params: &[],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.session.open",
        summary: "Import a KiCad or Eagle design into the engine session.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc { method: "open_project" },
        params: &[ParamSpec {
            name: "path",
            ty: ParamType::Str,
            required: true,
            doc: "KiCad or Eagle design file to import into the converter session",
            default_json: None,
        }],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.session.save",
        summary: "Save the current imported design to a path or back to its original file.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc { method: "save" },
        params: &[ParamSpec {
            name: "path",
            ty: ParamType::Str,
            required: false,
            doc: "Optional save path; defaults to the originally imported file",
            default_json: None,
        }],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.session.validate",
        summary: "Validate one native project directory for required files, supported schema versions, duplicate UUID consistency, and non-dangling persisted references.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "validate_project",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("validate"),
                ArgvToken::Param("path"),
            ],
        },
        params: &[ParamSpec {
            name: "path",
            ty: ParamType::Str,
            required: true,
            doc: "Project root directory",
            default_json: None,
        }],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
