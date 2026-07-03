//! The `datum.library` terminal verb family (2 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_library.py` schemas via
//! `tools_catalog_datum.py` aliases), the Python bridge argv builders
//! (`server_runtime.py`), and cross-checked against the clap definitions in
//! `crates/cli/src/cli_args_project_query_plan.rs`.
//!
//! Only the two verbs advertised in the GUI terminal command catalog are
//! registered here; the rest of the `datum.library` family migrates later.
//!
//! KNOWN DIVERGENCE (kept, reported): the historical GUI terminal template is
//! `query pool-library-objects <path> ...`, but the real clap surface (and the
//! Python bridge) is `project query <path> pool-library-objects ...` — the
//! top-level `query` subcommand has no `pool-library-objects` and the GUI form
//! falls into the legacy imported-query compatibility path and fails. The
//! dispatch argv below is the clap-correct form; `terminal_argv_override`
//! preserves the historical GUI template byte-for-byte until the GUI command
//! is corrected.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: true,
    doc: "Project root directory",
    default_json: None,
};

const POOL: ParamSpec = ParamSpec {
    name: "pool",
    ty: ParamType::Str,
    required: false,
    doc: "Project-local pool path",
    default_json: None,
};

const KIND: ParamSpec = ParamSpec {
    name: "kind",
    ty: ParamType::Str,
    required: false,
    doc: "Pool-library object kind: units, symbols, entities, parts, packages, footprints, padstacks, or pin_pad_maps",
    default_json: None,
};

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.library.list_objects",
        summary: "List resolver-discovered native pool-library objects.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_pool_library_objects",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit("pool-library-objects"),
                ArgvToken::Flag { flag: "--pool", param: "pool" },
                ArgvToken::Flag { flag: "--kind", param: "kind" },
                ArgvToken::Flag { flag: "--object", param: "object" },
                ArgvToken::Switch { flag: "--include-payload", param: "include_payload" },
            ],
        },
        params: &[
            PATH,
            POOL,
            KIND,
            ParamSpec {
                name: "object",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional pool-library object UUID filter",
                default_json: None,
            },
            ParamSpec {
                name: "include_payload",
                ty: ParamType::Bool,
                required: false,
                doc: "Include materialized object payloads in the listing",
                default_json: None,
            },
        ],
        schema_json_override: Some(
            r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]},"object":{"type":["string","null"]},"include_payload":{"type":["boolean","null"]}},"required":["path"]}"#,
        ),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool"],
        terminal_argv_override: Some(&[
            ArgvToken::Lit("query"),
            ArgvToken::Lit("pool-library-objects"),
            ArgvToken::Param("path"),
            ArgvToken::Flag { flag: "--pool", param: "pool" },
        ]),
    },
    VerbSpec {
        id: "datum.library.show_object",
        summary: "Show one resolver-discovered native pool-library object with its materialized payload.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "show_pool_library_object",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit("pool-library-objects"),
                ArgvToken::Flag { flag: "--pool", param: "pool" },
                ArgvToken::Flag { flag: "--kind", param: "kind" },
                ArgvToken::Flag { flag: "--object", param: "object" },
                ArgvToken::Lit("--include-payload"),
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "object",
                ty: ParamType::Uuid,
                required: true,
                doc: "Pool-library object UUID to show",
                default_json: None,
            },
            POOL,
            KIND,
        ],
        schema_json_override: Some(
            r#"{"type":"object","properties":{"path":{"type":"string"},"object":{"type":"string"},"pool":{"type":["string","null"]},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]}},"required":["path","object"]}"#,
        ),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool", "kind"],
        terminal_argv_override: Some(&[
            ArgvToken::Lit("query"),
            ArgvToken::Lit("pool-library-objects"),
            ArgvToken::Param("path"),
            ArgvToken::Flag { flag: "--pool", param: "pool" },
            ArgvToken::Flag { flag: "--kind", param: "kind" },
            ArgvToken::Flag { flag: "--object", param: "object" },
            ArgvToken::Lit("--include-payload"),
        ]),
    },
];
