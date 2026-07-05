//! The `datum.pool` imported-session read family (3 verbs), transcribed from
//! the legacy canonical alias catalog in `tools_catalog_data.py`.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const UUID: ParamSpec = ParamSpec {
    name: "uuid",
    ty: ParamType::Str,
    required: true,
    doc: "Pool object UUID",
    default_json: None,
};

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.pool.get_package",
        summary: "Return package/body metadata plus legacy package land-pattern compatibility fields for a UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "get_package",
        },
        params: &[UUID],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pool.get_part",
        summary: "Return detailed pool part metadata for a UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc { method: "get_part" },
        params: &[UUID],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pool.search",
        summary: "Search imported pool parts by keyword.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "search_pool",
        },
        params: &[ParamSpec {
            name: "query",
            ty: ParamType::Str,
            required: true,
            doc: "Pool search keyword",
            default_json: None,
        }],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
