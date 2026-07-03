//! The `datum.project` terminal verb family (3 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_datum.py` /
//! `tools_catalog_library.py` schemas), the Python bridge argv builders
//! (`server_runtime.py` / `server_runtime_library.py`), and cross-checked
//! against the clap definitions in `crates/cli/src/cli_args_project_commands.rs`
//! and `cli_args_project_library_pin_pad_map.rs`.
//!
//! Only the verbs advertised in the GUI terminal command catalog are
//! registered here; the rest of the `datum.project` family migrates later.
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
    doc: "Project-local pool path; defaults to pool",
    default_json: Some("\"pool\""),
};

const MAP: ParamSpec = ParamSpec {
    name: "map",
    ty: ParamType::Uuid,
    required: true,
    doc: "PinPadMap UUID",
    default_json: None,
};

const ENTRIES: ParamSpec = ParamSpec {
    name: "entries",
    ty: ParamType::StrList,
    required: true,
    doc: "Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous",
    default_json: None,
};

/// Exact hand-written MCP schema: `entries` items are pad/gate/pin objects,
/// which `ParamSpec` types cannot express.
const CREATE_PIN_PAD_MAP_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"map":{"type":"string"},"part":{"type":"string"},"footprint":{"type":["string","null"]},"entries":{"type":"array","items":{"type":"object","properties":{"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["pad","pin"]}},"set_default":{"type":"boolean"}},"required":["path","map","part","entries"]}"#;

const SET_PIN_PAD_MAP_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"map":{"type":"string"},"mode":{"type":"string"},"entries":{"type":"array","items":{"type":"object","properties":{"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["pad","pin"]}}},"required":["path","map","entries"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.project.create_pool_pin_pad_map",
        summary: "Create one first-class native pool PinPadMap through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_pin_pad_map",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-pin-pad-map"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--pool", param: "pool" },
                ArgvToken::Flag { flag: "--map", param: "map" },
                ArgvToken::Flag { flag: "--part", param: "part" },
                ArgvToken::Flag { flag: "--footprint", param: "footprint" },
                ArgvToken::Switch { flag: "--set-default", param: "set_default" },
                ArgvToken::Repeated { flag: "--entry", param: "entries" },
            ],
        },
        params: &[
            PATH,
            POOL,
            MAP,
            ParamSpec {
                name: "part",
                ty: ParamType::Uuid,
                required: true,
                doc: "Part UUID this PinPadMap binds",
                default_json: None,
            },
            ENTRIES,
            ParamSpec {
                name: "footprint",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional Footprint UUID; if omitted mappings target package pads",
                default_json: Some("null"),
            },
            ParamSpec {
                name: "set_default",
                ty: ParamType::Bool,
                required: false,
                doc: "Also set this map as the part default_pin_pad_map in the same journal batch",
                default_json: Some("false"),
            },
        ],
        schema_json_override: Some(CREATE_PIN_PAD_MAP_SCHEMA),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.project.set_pool_pin_pad_map",
        summary: "Update first-class native pool PinPadMap mappings through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_pin_pad_map",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-pin-pad-map"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--pool", param: "pool" },
                ArgvToken::Flag { flag: "--map", param: "map" },
                ArgvToken::Flag { flag: "--mode", param: "mode" },
                ArgvToken::Repeated { flag: "--entry", param: "entries" },
            ],
        },
        params: &[
            PATH,
            POOL,
            MAP,
            ParamSpec {
                name: "mode",
                ty: ParamType::Str,
                required: false,
                doc: "Merge listed mappings or replace the full mapping table",
                default_json: Some("\"merge\""),
            },
            ENTRIES,
        ],
        schema_json_override: Some(SET_PIN_PAD_MAP_SCHEMA),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool", "mode"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.project.validate",
        summary: "Validate one native project directory for required files, schema versions, UUID consistency, and persisted references.",
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
        params: &[PATH],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
