//! Round-trip gate: every `Dispatch::Cli` verb in the single-source verb
//! registry must render an argv that the real `datum-eda` clap surface
//! accepts.
//!
//! The CLI crate is binary-only, so instead of linking its private clap
//! definitions this test executes the actual `datum-eda` binary
//! (`CARGO_BIN_EXE_datum-eda`) with registry-rendered argv against a
//! nonexistent project path. Clap parse failures are detected via their
//! distinctive usage-error output; runtime "project not found" failures are
//! expected and fine.

use std::process::Command;

use datum_verb_registry::{ArgvToken, Dispatch, ParamSpec, ParamType, verbs};

/// Substrings clap 4 emits on argv parse/validation failures and never on
/// engine runtime errors.
const CLAP_PARSE_FAILURE_MARKERS: &[&str] = &[
    "Usage:",
    "unexpected argument",
    "unrecognized subcommand",
    "invalid value",
    "required arguments were not provided",
    "cannot be used with",
];

fn dummy_value(param: &ParamSpec) -> &'static str {
    match param.ty {
        ParamType::Str => "verb-registry-roundtrip-dummy",
        ParamType::Int => "1",
        ParamType::Bool => "true",
        ParamType::Uuid => "00000000-0000-0000-0000-000000000001",
        ParamType::StrList => "verb-registry-roundtrip-dummy",
        ParamType::Json => "{}",
    }
}

fn param_spec<'a>(params: &'a [ParamSpec], name: &str) -> &'a ParamSpec {
    params
        .iter()
        .find(|param| param.name == name)
        .unwrap_or_else(|| panic!("argv references undeclared param {name}"))
}

fn render_token(token: &ArgvToken, params: &[ParamSpec], argv: &mut Vec<String>) {
    match *token {
        ArgvToken::Lit(lit) => argv.push(lit.to_string()),
        ArgvToken::Param(name) => argv.push(dummy_value(param_spec(params, name)).to_string()),
        ArgvToken::Flag { flag, param } => {
            argv.push(flag.to_string());
            argv.push(dummy_value(param_spec(params, param)).to_string());
        }
        ArgvToken::Switch { flag, .. } => argv.push(flag.to_string()),
        ArgvToken::Repeated { flag, param } => {
            argv.push(flag.to_string());
            argv.push(dummy_value(param_spec(params, param)).to_string());
        }
    }
}

fn is_required(token: &ArgvToken, params: &[ParamSpec]) -> bool {
    match token.param_name() {
        Some(name) => param_spec(params, name).required,
        None => true,
    }
}

fn assert_clap_accepts(verb_id: &str, argv: &[String]) {
    let output = Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(argv)
        .output()
        .unwrap_or_else(|err| panic!("{verb_id}: failed to spawn datum-eda: {err}"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    for marker in CLAP_PARSE_FAILURE_MARKERS {
        assert!(
            !stderr.contains(marker),
            "{verb_id}: clap rejected registry argv {argv:?} (marker {marker:?}):\n{stderr}"
        );
    }
}

#[test]
fn every_cli_verb_argv_parses_against_the_real_clap_surface() {
    let mut checked = 0usize;
    for verb in verbs() {
        let Dispatch::Cli { argv, .. } = verb.dispatch else {
            continue;
        };
        // Required-only rendering (optional flags may conflict pairwise, e.g.
        // `artifact generate --output-job` vs `--include`).
        let mut base = Vec::new();
        for token in argv {
            if is_required(token, verb.params) {
                render_token(token, verb.params, &mut base);
            }
        }
        assert_clap_accepts(verb.id, &base);
        // Each optional token individually on top of the required base.
        for token in argv {
            if !is_required(token, verb.params) {
                let mut extended = base.clone();
                render_token(token, verb.params, &mut extended);
                assert_clap_accepts(verb.id, &extended);
            }
        }
        checked += 1;
    }
    assert!(checked > 0, "registry contained no Dispatch::Cli verbs");
}
