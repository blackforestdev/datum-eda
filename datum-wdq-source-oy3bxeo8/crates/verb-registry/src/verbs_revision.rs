//! Engine-owned Product Revision Engine discovery and query verbs.

use crate::{Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const QUERY_PARAMS: &[ParamSpec] = &[
    ParamSpec {
        name: "project_root",
        ty: ParamType::Str,
        required: true,
        doc: "Native Project root whose revision authority is queried",
        default_json: None,
    },
    ParamSpec {
        name: "query",
        ty: ParamType::Str,
        required: true,
        doc: "Stable query name returned by datum.revision.catalog",
        default_json: None,
    },
    ParamSpec {
        name: "as_of_sequence",
        ty: ParamType::Int,
        required: false,
        doc: "Inclusive historical authority event sequence",
        default_json: None,
    },
    ParamSpec {
        name: "expected_model_revision",
        ty: ParamType::Str,
        required: false,
        doc: "Exact Design model revision context fence",
        default_json: None,
    },
];

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.revision.catalog",
        summary: "Return the complete engine-owned revision operation, query, refusal, proposal, and evidence inventory.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "revision.catalog",
        },
        params: &[],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.revision.query",
        summary: "Query current or historical typed revision authority through the engine-owned service.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "revision.query",
        },
        params: QUERY_PARAMS,
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
