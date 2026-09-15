//! The `datum.component_instance` verb family (3 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_relationships.py` schemas), the
//! Python bridge argv builders (`server_runtime.py`), and cross-checked
//! against the clap definitions in `crates/cli/src/args/project_component_instances.rs`.
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

const SYMBOL: ParamSpec = ParamSpec {
    name: "symbol",
    ty: ParamType::Uuid,
    required: true,
    doc: "Schematic symbol UUID to bind",
    default_json: None,
};

const PACKAGE: ParamSpec = ParamSpec {
    name: "package",
    ty: ParamType::Uuid,
    required: true,
    doc: "Board package UUID to bind",
    default_json: None,
};

const COMPONENT_INSTANCE: ParamSpec = ParamSpec {
    name: "component_instance",
    ty: ParamType::Uuid,
    required: true,
    doc: "ComponentInstance UUID",
    default_json: None,
};

const SYMBOLS: ParamSpec = ParamSpec {
    name: "symbols",
    ty: ParamType::Uuid,
    required: false,
    doc: "Additional schematic symbol UUIDs for a multi-symbol binding",
    default_json: None,
};

const PART: ParamSpec = ParamSpec {
    name: "part",
    ty: ParamType::Uuid,
    required: false,
    doc: "Optional native pool part UUID",
    default_json: Some("null"),
};

const SYMBOL_ROLES: ParamSpec = ParamSpec {
    name: "symbol_roles",
    ty: ParamType::StrList,
    required: false,
    doc: "Optional per-symbol role metadata",
    default_json: Some("null"),
};

const PACKAGE_ROLES: ParamSpec = ParamSpec {
    name: "package_roles",
    ty: ParamType::StrList,
    required: false,
    doc: "Optional per-package role metadata",
    default_json: Some("null"),
};

/// Exact hand-written MCP schema: callers may provide either `symbol` or
/// `symbols`, and role metadata may be object, array, or null.
const BIND_COMPONENT_INSTANCE_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"package":{"type":"string"},"part":{"type":["string","null"]},"symbol_roles":{"type":["object","array","null"]},"package_roles":{"type":["object","array","null"]},"component_instance":{"type":["string","null"]},"symbols":{"type":"array","items":{"type":"string"},"minItems":1}},"required":["path","package"],"anyOf":[{"required":["symbol"]},{"required":["symbols"]}]}"#;

const SET_COMPONENT_INSTANCE_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"component_instance":{"type":"string"},"symbol":{"type":"string"},"package":{"type":"string"},"part":{"type":["string","null"]},"symbol_roles":{"type":["object","array","null"]},"package_roles":{"type":["object","array","null"]},"symbols":{"type":"array","items":{"type":"string"},"minItems":1}},"required":["path","component_instance","package"],"anyOf":[{"required":["symbol"]},{"required":["symbols"]}]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.component_instance.bind",
        summary: "Create a journaled ComponentInstance binding between one or more schematic symbols and one board package.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "bind_component_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("bind-component-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Repeated {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--component-instance",
                    param: "component_instance",
                },
                ArgvToken::Repeated {
                    flag: "--symbol",
                    param: "symbols",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Repeated {
                    flag: "--symbol-role",
                    param: "symbol_roles",
                },
                ArgvToken::Repeated {
                    flag: "--package-role",
                    param: "package_roles",
                },
            ],
        },
        params: &[
            PATH,
            SYMBOL,
            PACKAGE,
            ParamSpec {
                required: false,
                ..COMPONENT_INSTANCE
            },
            SYMBOLS,
            PART,
            SYMBOL_ROLES,
            PACKAGE_ROLES,
        ],
        schema_json_override: Some(BIND_COMPONENT_INSTANCE_SCHEMA),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.component_instance.delete",
        summary: "Delete one journaled ComponentInstance binding by UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_component_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-component-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component-instance",
                    param: "component_instance",
                },
            ],
        },
        params: &[PATH, COMPONENT_INSTANCE],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.component_instance.set",
        summary: "Update a journaled ComponentInstance symbol/package binding, preserving multi-symbol authored instances when `symbols` is supplied.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_component_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-component-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component-instance",
                    param: "component_instance",
                },
                ArgvToken::Repeated {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Repeated {
                    flag: "--symbol",
                    param: "symbols",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Repeated {
                    flag: "--symbol-role",
                    param: "symbol_roles",
                },
                ArgvToken::Repeated {
                    flag: "--package-role",
                    param: "package_roles",
                },
            ],
        },
        params: &[
            PATH,
            COMPONENT_INSTANCE,
            SYMBOL,
            PACKAGE,
            SYMBOLS,
            PART,
            SYMBOL_ROLES,
            PACKAGE_ROLES,
        ],
        schema_json_override: Some(SET_COMPONENT_INSTANCE_SCHEMA),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
