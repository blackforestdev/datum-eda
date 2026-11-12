//! Typed shared alignment schema and its real CLI argument projection.
use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};
macro_rules! p {
    ($name:literal, $ty:ident, $required:expr, $default:expr) => {
        ParamSpec {
            name: $name,
            ty: ParamType::$ty,
            required: $required,
            doc: "Native alignment parameter.",
            default_json: $default,
        }
    };
}
pub(super) const VERB: VerbSpec = VerbSpec {
    id: "datum.schematic.align_symbols",
    summary: "Align complete native Symbol bodies in one qualified sheet occurrence.",
    status: VerbStatus::Public,
    replacements: &[],
    retirement: None,
    dispatch: Dispatch::Cli {
        method: "align_symbols",
        argv: &[
            ArgvToken::Lit("project"),
            ArgvToken::Lit("align-symbols"),
            ArgvToken::Flag {
                flag: "--root",
                param: "path",
            },
            ArgvToken::Repeated {
                flag: "--symbol",
                param: "symbols",
            },
            ArgvToken::Repeated {
                flag: "--instance",
                param: "instance_path",
            },
            ArgvToken::Flag {
                flag: "--axis",
                param: "axis",
            },
            ArgvToken::Flag {
                flag: "--position",
                param: "position",
            },
            ArgvToken::Flag {
                flag: "--op",
                param: "op",
            },
            ArgvToken::Flag {
                flag: "--reference",
                param: "reference",
            },
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
    params: &[
        p!("path", Str, true, None),
        p!("symbols", StrList, true, None),
        p!("instance_path", StrList, false, Some("[]")),
        p!("axis", Str, true, None),
        p!("position", Str, true, None),
        p!("op", Str, false, Some("\"align\"")),
        p!("reference", Str, false, Some("\"selection_bounds\"")),
        p!("expected_model_revision", Str, true, None),
        p!("expected_tip_transaction", Str, true, None),
    ],
    schema_json_override: Some(
        r#"{"type":"object","additionalProperties":false,"properties":{"path":{"type":"string"},"symbols":{"type":"array","minItems":2,"uniqueItems":true,"items":{"type":"string","format":"uuid"}},"instance_path":{"type":"array","items":{"type":"string","format":"uuid"},"default":[]},"axis":{"enum":["horizontal","vertical"]},"position":{"enum":["left","center","right","top","bottom"]},"op":{"enum":["align"],"default":"align"},"reference":{"enum":["selection_bounds"],"default":"selection_bounds"},"expected_model_revision":{"type":"string","minLength":1},"expected_tip_transaction":{"type":"string","description":"Observed transaction UUID or none for an empty journal"}},"required":["path","symbols","axis","position","expected_model_revision","expected_tip_transaction"]}"#,
    ),
    write_surface: None,
    terminal: false,
    terminal_optional_params: &[],
    terminal_argv_override: None,
};
