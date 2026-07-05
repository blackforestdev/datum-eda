//! The `datum.check` verb family (10 verbs), transcribed from the hand-written
//! MCP catalog (`tools_catalog_checks.py` schemas via `tools_catalog_datum.py`
//! aliases), the Python bridge argv builders (`server_runtime.py`), and
//! cross-checked against the clap definitions in
//! `crates/cli/src/cli_args_check.rs`. `explain_violation` dispatches straight
//! to the daemon JSON-RPC method and is not a GUI terminal verb.
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

const FINGERPRINT: ParamSpec = ParamSpec {
    name: "fingerprint",
    ty: ParamType::Str,
    required: true,
    doc: "Stable CheckFinding fingerprint",
    default_json: None,
};

const CHECK_RUN_SUMMARY: &str =
    "Run a native project CheckRun profile and persist a resolver-owned CheckRun evidence artifact.";

const CHECK_RUN_ARGV: &[ArgvToken] = &[
    ArgvToken::Lit("check"),
    ArgvToken::Lit("run"),
    ArgvToken::Param("path"),
    ArgvToken::Flag { flag: "--profile", param: "profile" },
];

const CHECK_RUN_PARAMS: &[ParamSpec] = &[
    PATH,
    ParamSpec {
        name: "profile",
        ty: ParamType::Str,
        required: false,
        doc: "Check profile id: native-combined, erc, drc, standards, manufacturing, or release",
        default_json: None,
    },
];

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.check.accept_deviation",
        summary: "Accept a fingerprint-scoped check finding as a deviation through the native project journal.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "accept_deviation",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("accept-deviation"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--fingerprint", param: "fingerprint" },
                ArgvToken::Flag { flag: "--rationale", param: "rationale" },
                ArgvToken::Flag { flag: "--accepted-by", param: "accepted_by" },
            ],
        },
        params: &[
            PATH,
            FINGERPRINT,
            ParamSpec {
                name: "rationale",
                ty: ParamType::Str,
                required: true,
                doc: "Deviation rationale recorded in the authored project",
                default_json: None,
            },
            ParamSpec {
                name: "accepted_by",
                ty: ParamType::Str,
                required: false,
                doc: "Optional actor/user recorded as accepting the deviation",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.explain_violation",
        summary: "Explain an ERC/DRC/check finding by stable fingerprint, with legacy positional index fallback for compatibility.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc { method: "explain_violation" },
        params: &[
            ParamSpec {
                name: "domain",
                ty: ParamType::Str,
                required: true,
                doc: "Finding domain: erc or drc",
                default_json: None,
            },
            ParamSpec {
                name: "index",
                ty: ParamType::Int,
                required: false,
                doc: "Legacy positional finding index fallback",
                default_json: None,
            },
            ParamSpec {
                name: "fingerprint",
                ty: ParamType::Str,
                required: false,
                doc: "Stable CheckFinding fingerprint",
                default_json: None,
            },
        ],
        // Exact hand-written MCP schema: the `domain` enum cannot be
        // expressed through `ParamSpec` types.
        schema_json_override: Some(
            r#"{"type":"object","properties":{"domain":{"type":"string","enum":["erc","drc"]},"index":{"type":["integer","null"]},"fingerprint":{"type":["string","null"]}},"required":["domain"]}"#,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.fill_zones",
        summary: "Persist honest generated ZoneFill evidence for native board zones. The bounded solver fills closed same-net zones, supports one rectangular foreign pad/via cutout with positive netclass clearance, and records Unsupported evidence for thermals, keepouts, unresolved pads, tracks, or general pour cases.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "fill_zones",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("fill-zones"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--zone", param: "zone" },
                ArgvToken::Flag { flag: "--net", param: "net" },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "zone",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional Zone UUID to fill",
                default_json: None,
            },
            ParamSpec {
                name: "net",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional Net UUID to fill",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.list",
        summary: "List resolver-discovered persisted CheckRun evidence artifacts.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_check_runs",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("list"),
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
    VerbSpec {
        id: "datum.check.profiles",
        summary: "List supported native-project CheckRun profiles.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_check_profiles",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("profiles"),
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
    VerbSpec {
        id: "datum.check.repair_standards",
        summary: "Generate non-mutating draft standards-repair proposals from persisted CheckRun findings, including process-aperture, geometry, and supported ZoneFill evidence repairs.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "generate_standards_repair_proposals",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("repair-standards"),
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
    VerbSpec {
        id: "datum.check.run",
        summary: CHECK_RUN_SUMMARY,
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli { method: "get_check_run", argv: CHECK_RUN_ARGV },
        params: CHECK_RUN_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.run_profile",
        summary: CHECK_RUN_SUMMARY,
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli { method: "get_check_run", argv: CHECK_RUN_ARGV },
        params: CHECK_RUN_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        // Same CLI surface as datum.check.run; the profile-scoped alias
        // advertises the `--profile` flag in its terminal template.
        terminal_optional_params: &["profile"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.show",
        summary: "Show one resolver-discovered persisted CheckRun evidence artifact by UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "show_check_run",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("show"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--check-run", param: "check_run" },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "check_run",
                ty: ParamType::Uuid,
                required: true,
                doc: "CheckRun UUID to inspect",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.check.waive",
        summary: "Author a fingerprint-scoped check finding waiver through the native project journal.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "waive_finding",
            argv: &[
                ArgvToken::Lit("check"),
                ArgvToken::Lit("waive"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--fingerprint", param: "fingerprint" },
                ArgvToken::Flag { flag: "--rationale", param: "rationale" },
                ArgvToken::Flag { flag: "--created-by", param: "created_by" },
            ],
        },
        params: &[
            PATH,
            FINGERPRINT,
            ParamSpec {
                name: "rationale",
                ty: ParamType::Str,
                required: true,
                doc: "Waiver rationale recorded in the authored project",
                default_json: None,
            },
            ParamSpec {
                name: "created_by",
                ty: ParamType::Str,
                required: false,
                doc: "Optional actor/user recorded on the waiver",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
