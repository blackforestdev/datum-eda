//! Exact unit-resolution adapter shared by CLI and MCP.

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const OPTIONAL: &[&str] = &[
    "canonical_nm",
    "expression",
    "quantity",
    "unit",
    "system",
    "field",
    "project_id",
];

pub(crate) static VERBS: &[VerbSpec] = &[VerbSpec {
    id: "datum.units.resolve_length",
    summary: "Resolve an integer nanometer value or exact scalar-with-unit expression with explicit, machine-independent context.",
    status: VerbStatus::Public,
    replacements: &[],
    retirement: None,
    dispatch: Dispatch::Cli {
        method: "resolve_length",
        argv: &[
            ArgvToken::Lit("units"),
            ArgvToken::Lit("resolve-length"),
            ArgvToken::Flag {
                flag: "--canonical-nm",
                param: "canonical_nm",
            },
            ArgvToken::Flag {
                flag: "--expression",
                param: "expression",
            },
            ArgvToken::Flag {
                flag: "--quantity",
                param: "quantity",
            },
            ArgvToken::Flag {
                flag: "--unit",
                param: "unit",
            },
            ArgvToken::Flag {
                flag: "--system",
                param: "system",
            },
            ArgvToken::Flag {
                flag: "--field",
                param: "field",
            },
            ArgvToken::Flag {
                flag: "--project-id",
                param: "project_id",
            },
        ],
    },
    params: &[
        ParamSpec {
            name: "canonical_nm",
            ty: ParamType::Int,
            required: false,
            doc: "Existing canonical integer nanometer value",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "expression",
            ty: ParamType::Str,
            required: false,
            doc: "Locale-independent scalar with an optional supported unit suffix",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "quantity",
            ty: ParamType::Str,
            required: false,
            doc: "Explicit bare-value quantity: board, drill, or schematic",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "unit",
            ty: ParamType::Str,
            required: false,
            doc: "Explicit bare-value unit: nm, um, µm, mm, mil, or in",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "system",
            ty: ParamType::Str,
            required: false,
            doc: "Explicit measurement system: metric or imperial",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "field",
            ty: ParamType::Str,
            required: false,
            doc: "Stable field identity for a contextual bare value",
            default_json: Some("null"),
        },
        ParamSpec {
            name: "project_id",
            ty: ParamType::Str,
            required: false,
            doc: "Optional Project identity retained in provenance",
            default_json: Some("null"),
        },
    ],
    schema_json_override: None,
    write_surface: None,
    terminal: true,
    terminal_optional_params: OPTIONAL,
    terminal_argv_override: None,
}];
