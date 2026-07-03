//! The `datum.query` terminal verb family (1 verb), transcribed from the
//! hand-written MCP catalog (`tools_catalog_datum.py`), the Python bridge argv
//! builder (`server_runtime.py`), and cross-checked against the clap
//! definitions in `crates/cli/src/cli_args_project_query_plan.rs`.
//!
//! Only the verb advertised in the GUI terminal command catalog is registered
//! here; the rest of the `datum.query` family migrates later.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

pub(crate) static VERBS: &[VerbSpec] = &[VerbSpec {
    id: "datum.query.source_shards",
    summary: "Canonical Datum read-only query alias for one native project path.",
    status: VerbStatus::Public,
    replacements: &[],
    retirement: None,
    dispatch: Dispatch::Cli {
        method: "get_source_shards",
        argv: &[
            ArgvToken::Lit("project"),
            ArgvToken::Lit("query"),
            ArgvToken::Param("path"),
            ArgvToken::Lit("resolve-debug"),
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
    terminal: true,
    terminal_optional_params: &[],
    terminal_argv_override: None,
}];
