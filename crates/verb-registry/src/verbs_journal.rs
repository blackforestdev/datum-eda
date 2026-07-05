//! The `datum.journal` verb family (4 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_journal.py` schemas via
//! `tools_catalog_datum.py` aliases), the Python bridge argv builders
//! (`server_runtime.py`), and cross-checked against the clap definitions in
//! `crates/cli/src/cli_args_journal.rs` / `cli_args_project_journal.rs`.
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

const EXPECTED_MODEL_REVISION: ParamSpec = ParamSpec {
    name: "expected_model_revision",
    ty: ParamType::Str,
    required: false,
    doc: "Refuse the compensating transaction unless the resolved model revision matches this value",
    default_json: None,
};

const EXPECTED_TIP_TRANSACTION: ParamSpec = ParamSpec {
    name: "expected_tip_transaction",
    ty: ParamType::Uuid,
    required: false,
    doc: "Refuse the compensating transaction unless the current journal tip has this transaction UUID",
    default_json: None,
};

/// Exact hand-written MCP schema (`tools_catalog_journal.py`): the optional
/// guard parameters are declared non-nullable `string` there, which the
/// ParamSpec derivation (optional => nullable) cannot express.
const UNDO_REDO_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"expected_model_revision":{"type":"string"},"expected_tip_transaction":{"type":"string"}},"required":["path"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.journal.list",
        summary: "Return the resolver-backed native project transaction journal summary.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_journal_list",
            argv: &[
                ArgvToken::Lit("journal"),
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
        id: "datum.journal.redo",
        summary: "Apply one native project journal redo as a compensating transaction, optionally guarded by model revision or journal tip.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "journal_redo",
            argv: &[
                ArgvToken::Lit("journal"),
                ArgvToken::Lit("redo"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--expected-model-revision",
                    param: "expected_model_revision",
                },
                ArgvToken::Flag {
                    flag: "--expected-tip-transaction",
                    param: "expected_tip_transaction",
                },
            ],
        },
        params: &[PATH, EXPECTED_MODEL_REVISION, EXPECTED_TIP_TRANSACTION],
        schema_json_override: Some(UNDO_REDO_SCHEMA),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.journal.show",
        summary: "Return one full native project transaction journal record by transaction UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_journal_transaction",
            argv: &[
                ArgvToken::Lit("journal"),
                ArgvToken::Lit("show"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--transaction",
                    param: "transaction",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "transaction",
                ty: ParamType::Uuid,
                required: true,
                doc: "Transaction UUID",
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
        id: "datum.journal.undo",
        summary: "Apply one native project journal undo as a compensating transaction, optionally guarded by model revision or journal tip.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "journal_undo",
            argv: &[
                ArgvToken::Lit("journal"),
                ArgvToken::Lit("undo"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--expected-model-revision",
                    param: "expected_model_revision",
                },
                ArgvToken::Flag {
                    flag: "--expected-tip-transaction",
                    param: "expected_tip_transaction",
                },
            ],
        },
        params: &[PATH, EXPECTED_MODEL_REVISION, EXPECTED_TIP_TRANSACTION],
        schema_json_override: Some(UNDO_REDO_SCHEMA),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
