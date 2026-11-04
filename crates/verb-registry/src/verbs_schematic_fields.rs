//! Shared, revision-qualified schematic field editing public caller.
use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const fn parameter(
    name: &'static str,
    ty: ParamType,
    required: bool,
    doc: &'static str,
) -> ParamSpec {
    ParamSpec {
        name,
        ty,
        required,
        doc,
        default_json: None,
    }
}

pub(crate) static VERBS: &[VerbSpec] = &[VerbSpec {
    id: "datum.schematic.set_symbol_field",
    summary: "Edit a Symbol reference or value through the shared native editor. Requires observed revision and tip (UUID or none for empty). Clear Value stores empty text; Reference cannot be cleared. Automated component-bound edits retain Proposal requirements.",
    status: VerbStatus::Public,
    replacements: &[],
    retirement: None,
    dispatch: Dispatch::Cli {
        method: "set_symbol_field",
        argv: &[
            ArgvToken::Lit("project"),
            ArgvToken::Lit("set-symbol-field"),
            ArgvToken::Flag {
                flag: "--root",
                param: "path",
            },
            ArgvToken::Flag {
                flag: "--symbol",
                param: "symbol",
            },
            ArgvToken::Flag {
                flag: "--field",
                param: "field",
            },
            ArgvToken::Flag {
                flag: "--expected-model-revision",
                param: "expected_model_revision",
            },
            ArgvToken::Flag {
                flag: "--expected-tip-transaction",
                param: "expected_tip_transaction",
            },
            ArgvToken::Flag {
                flag: "--value",
                param: "value",
            },
            ArgvToken::Switch {
                flag: "--clear",
                param: "clear",
            },
            ArgvToken::Repeated {
                flag: "--instance-path",
                param: "instance_path",
            },
        ],
    },
    params: &[
        parameter("path", ParamType::Str, true, "Project root directory."),
        parameter("symbol", ParamType::Uuid, true, "Selected Symbol UUID."),
        parameter("field", ParamType::Str, true, "reference or value."),
        parameter(
            "expected_model_revision",
            ParamType::Str,
            true,
            "Observed nonempty model revision.",
        ),
        parameter(
            "expected_tip_transaction",
            ParamType::Str,
            true,
            "Observed tip UUID, or none for an empty journal. Omission is not an empty assertion.",
        ),
        parameter(
            "value",
            ParamType::Str,
            false,
            "Literal field text; mutually exclusive with clear.",
        ),
        parameter(
            "clear",
            ParamType::Bool,
            false,
            "Clear Value to empty text; Reference clearing refuses.",
        ),
        parameter(
            "instance_path",
            ParamType::StrList,
            false,
            "Ordered sheet-instance UUID path; defaults to root.",
        ),
    ],
    schema_json_override: Some(
        r#"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"field":{"type":"string","enum":["reference","value"]},"expected_model_revision":{"type":"string","minLength":1},"expected_tip_transaction":{"type":"string","minLength":1},"value":{"type":"string"},"clear":{"type":"boolean"},"instance_path":{"type":"array","items":{"type":"string"}}},"required":["path","symbol","field","expected_model_revision","expected_tip_transaction"]}"#,
    ),
    write_surface: None,
    terminal: false,
    terminal_optional_params: &[],
    terminal_argv_override: None,
}];
