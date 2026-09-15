//! The `datum.route` MCP verb family (21 verbs), transcribed from the
//! hand-written Datum MCP aliases and Python CLI bridge methods.
//!
//! Entries are kept compact with schema overrides because this family exposes
//! enum-rich artifact and strategy surfaces whose public schemas already exist.

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus, WriteSurface};

const JOURNALED_ROUTE_APPLY: WriteSurface = WriteSurface {
    class: "journaled_route_apply",
    evidence: "routes through route-apply proposal/journal gateway",
};

const PROPOSAL_ARTIFACT_APPLY: WriteSurface = WriteSurface {
    class: "proposal_artifact_apply",
    evidence: "applies embedded route proposal artifact through substrate apply path",
};

macro_rules! p {
    ($name:literal, $ty:ident, required) => {
        ParamSpec {
            name: $name,
            ty: ParamType::$ty,
            required: true,
            doc: $name,
            default_json: None,
        }
    };
    ($name:literal, $ty:ident, optional) => {
        ParamSpec {
            name: $name,
            ty: ParamType::$ty,
            required: false,
            doc: $name,
            default_json: None,
        }
    };
}

macro_rules! route_verb {
    (
        $id:literal,
        $summary:literal,
        $method:literal,
        [$($argv:expr),* $(,)?],
        [$($param:expr),* $(,)?],
        $schema:expr,
        $write_surface:expr $(,)?
    ) => {
        VerbSpec {
            id: $id,
            summary: $summary,
            status: VerbStatus::Public,
            replacements: &[],
            retirement: None,
            dispatch: Dispatch::Cli {
                method: $method,
                argv: &[$($argv),*],
            },
            params: &[$($param),*],
            schema_json_override: Some($schema),
            write_surface: $write_surface,
            terminal: false,
            terminal_optional_params: &[],
            terminal_argv_override: None,
        }
    };
}

const PATH: ParamSpec = p!("path", Str, required);
const NET: ParamSpec = p!("net_uuid", Uuid, required);
const FROM_ANCHOR: ParamSpec = p!("from_anchor_pad_uuid", Uuid, required);
const TO_ANCHOR: ParamSpec = p!("to_anchor_pad_uuid", Uuid, required);
const PROFILE: ParamSpec = p!("profile", Str, optional);
const CANDIDATE: ParamSpec = p!("candidate", Str, required);
const POLICY: ParamSpec = p!("policy", Str, optional);
const ARTIFACT: ParamSpec = p!("artifact", Str, required);

const ROUTE_PREFIX: &[ArgvToken] = &[
    ArgvToken::Lit("project"),
    ArgvToken::Param("path"),
    ArgvToken::Flag {
        flag: "--net",
        param: "net_uuid",
    },
    ArgvToken::Flag {
        flag: "--from-anchor",
        param: "from_anchor_pad_uuid",
    },
    ArgvToken::Flag {
        flag: "--to-anchor",
        param: "to_anchor_pad_uuid",
    },
];

const ROUTE_SELECTION_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"profile":{"type":"string","enum":["default","authored-copper-priority"]}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid"]}"#;
const ROUTE_STRATEGY_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid"]}"#;
const ROUTE_CANDIDATE_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"candidate":{"type":"string","enum":["route-path-candidate","route-path-candidate-via","route-path-candidate-two-via","route-path-candidate-three-via","route-path-candidate-four-via","route-path-candidate-five-via","route-path-candidate-six-via","route-path-candidate-authored-via-chain","route-path-candidate-orthogonal-dogleg","route-path-candidate-orthogonal-two-bend","route-path-candidate-orthogonal-graph","route-path-candidate-orthogonal-graph-via","route-path-candidate-orthogonal-graph-two-via","route-path-candidate-orthogonal-graph-three-via","route-path-candidate-orthogonal-graph-four-via","route-path-candidate-orthogonal-graph-five-via","route-path-candidate-orthogonal-graph-six-via","authored-copper-plus-one-gap","authored-copper-graph"]},"policy":{"type":"string","enum":["plain","zone_aware","obstacle_aware","zone_obstacle_aware","zone_obstacle_topology_aware","zone_obstacle_topology_layer_balance_aware"]}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid","candidate"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    route_verb!(
        "datum.route.apply",
        "Apply one accepted deterministic route candidate through the proposal journal gateway.",
        "route_apply",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-apply"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--candidate",
                param: "candidate"
            },
            ArgvToken::Flag {
                flag: "--policy",
                param: "policy"
            },
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR, CANDIDATE, POLICY],
        ROUTE_CANDIDATE_SCHEMA,
        Some(JOURNALED_ROUTE_APPLY),
    ),
    route_verb!(
        "datum.route.apply_proposal_artifact",
        "Apply one native route proposal artifact through the proposal journal gateway when it still matches the current live project state.",
        "apply_route_proposal_artifact",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("apply-route-proposal-artifact"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--artifact",
                param: "artifact"
            },
        ],
        [PATH, ARTIFACT],
        r#"{"type":"object","properties":{"path":{"type":"string"},"artifact":{"type":"string"}},"required":["path","artifact"]}"#,
        Some(PROPOSAL_ARTIFACT_APPLY),
    ),
    route_verb!(
        "datum.route.apply_selected",
        "Apply the currently selected deterministic route proposal through the proposal journal gateway.",
        "route_apply_selected",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-apply-selected"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--profile",
                param: "profile"
            },
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR, PROFILE],
        ROUTE_SELECTION_SCHEMA,
        Some(JOURNALED_ROUTE_APPLY),
    ),
    route_verb!(
        "datum.route.capture_strategy_baseline",
        "Materialize the generated-regression-fixture route-strategy suite, evaluate it, and save one reusable versioned batch-result baseline artifact; not a user design-authoring path.",
        "capture_route_strategy_curated_baseline",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("capture-route-strategy-curated-baseline"),
            ArgvToken::Flag {
                flag: "--out-dir",
                param: "out_dir"
            },
            ArgvToken::Flag {
                flag: "--manifest",
                param: "manifest"
            },
            ArgvToken::Flag {
                flag: "--result",
                param: "result"
            },
        ],
        [
            p!("out_dir", Str, required),
            p!("manifest", Str, optional),
            p!("result", Str, optional),
        ],
        r#"{"type":"object","properties":{"out_dir":{"type":"string"},"manifest":{"type":["string","null"]},"result":{"type":["string","null"]}},"required":["out_dir"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.compare_strategy_batch_result",
        "Compare two saved versioned route-strategy batch result artifacts by request_id and aggregate summary counts without live re-evaluation.",
        "compare_route_strategy_batch_result",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("compare-route-strategy-batch-result"),
            ArgvToken::Param("before"),
            ArgvToken::Param("after"),
        ],
        [p!("before", Str, required), p!("after", Str, required)],
        r#"{"type":"object","properties":{"before":{"type":"string"},"after":{"type":"string"}},"required":["before","after"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.explain_proposal",
        "Explain family-level selection and rejection for the current deterministic route proposal.",
        "route_proposal_explain",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-proposal-explain"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--profile",
                param: "profile"
            },
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR, PROFILE],
        ROUTE_SELECTION_SCHEMA,
        None,
    ),
    route_verb!(
        "datum.route.export_path_proposal",
        "Export one deterministic route proposal artifact from an accepted current route-path candidate family.",
        "export_route_path_proposal",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("export-route-path-proposal"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--candidate",
                param: "candidate"
            },
            ArgvToken::Flag {
                flag: "--policy",
                param: "policy"
            },
            ArgvToken::Flag {
                flag: "--out",
                param: "out"
            },
        ],
        [
            PATH,
            NET,
            FROM_ANCHOR,
            TO_ANCHOR,
            CANDIDATE,
            POLICY,
            p!("out", Str, required),
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"candidate":{"type":"string","enum":["route-path-candidate","route-path-candidate-via","route-path-candidate-two-via","route-path-candidate-three-via","route-path-candidate-four-via","route-path-candidate-five-via","route-path-candidate-six-via","route-path-candidate-authored-via-chain","route-path-candidate-orthogonal-dogleg","route-path-candidate-orthogonal-two-bend","route-path-candidate-orthogonal-graph","route-path-candidate-orthogonal-graph-via","route-path-candidate-orthogonal-graph-two-via","route-path-candidate-orthogonal-graph-three-via","route-path-candidate-orthogonal-graph-four-via","route-path-candidate-orthogonal-graph-five-via","route-path-candidate-orthogonal-graph-six-via","authored-copper-plus-one-gap","authored-copper-graph"]},"policy":{"type":"string","enum":["plain","zone_aware","obstacle_aware","zone_obstacle_aware","zone_obstacle_topology_aware","zone_obstacle_topology_layer_balance_aware"]},"out":{"type":"string"}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid","candidate","out"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.export_proposal",
        "Export the currently selected deterministic route proposal as a native route proposal artifact.",
        "export_route_proposal",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("export-route-proposal"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--out",
                param: "out"
            },
            ArgvToken::Flag {
                flag: "--profile",
                param: "profile"
            },
        ],
        [
            PATH,
            NET,
            FROM_ANCHOR,
            TO_ANCHOR,
            p!("out", Str, required),
            PROFILE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"profile":{"type":"string","enum":["default","authored-copper-priority"]},"out":{"type":"string"}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid","out"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.gate_strategy_batch_result",
        "Evaluate two saved versioned route-strategy batch result artifacts against one explicit deterministic CI gate policy.",
        "gate_route_strategy_batch_result",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("gate-route-strategy-batch-result"),
            ArgvToken::Param("before"),
            ArgvToken::Param("after"),
            ArgvToken::Flag {
                flag: "--policy",
                param: "policy"
            },
        ],
        [
            p!("before", Str, required),
            p!("after", Str, required),
            POLICY
        ],
        r#"{"type":"object","properties":{"before":{"type":"string"},"after":{"type":"string"},"policy":{"type":"string","enum":["strict_identical","allow_aggregate_only","fail_on_recommendation_change"]}},"required":["before","after"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.inspect_proposal_artifact",
        "Inspect one native route proposal artifact without consulting live project state.",
        "inspect_route_proposal_artifact",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("inspect-route-proposal-artifact"),
            ArgvToken::Param("artifact"),
        ],
        [ARTIFACT],
        r#"{"type":"object","properties":{"artifact":{"type":"string"}},"required":["artifact"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.inspect_strategy_batch_result",
        "Inspect one saved versioned route-strategy batch result artifact and report summary counts, per-request outcomes, and malformed entries.",
        "inspect_route_strategy_batch_result",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("inspect-route-strategy-batch-result"),
            ArgvToken::Param("artifact"),
        ],
        [ARTIFACT],
        r#"{"type":"object","properties":{"artifact":{"type":"string"}},"required":["artifact"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.revalidate_proposal_artifact",
        "Revalidate one native route proposal artifact against the current live project state without applying it.",
        "revalidate_route_proposal_artifact",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("revalidate-route-proposal-artifact"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--artifact",
                param: "artifact"
            },
        ],
        [PATH, ARTIFACT],
        r#"{"type":"object","properties":{"path":{"type":"string"},"artifact":{"type":"string"}},"required":["path","artifact"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.review_proposal",
        "Review one selected deterministic route proposal or one saved route proposal artifact without mutating project state.",
        "review_route_proposal",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("review-route-proposal"),
            ArgvToken::Flag {
                flag: "--artifact",
                param: "artifact"
            },
        ],
        [
            p!("path", Str, optional),
            p!("net_uuid", Uuid, optional),
            p!("from_anchor_pad_uuid", Uuid, optional),
            p!("to_anchor_pad_uuid", Uuid, optional),
            PROFILE,
            ARTIFACT,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"profile":{"type":"string","enum":["default","authored-copper-priority"]},"artifact":{"type":"string"}},"oneOf":[{"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid"]},{"required":["artifact"]}]}"#,
        None,
    ),
    route_verb!(
        "datum.route.select_proposal",
        "Select the current deterministic route proposal for one net and anchor pair.",
        "route_proposal",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-proposal"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--profile",
                param: "profile"
            },
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR, PROFILE],
        ROUTE_SELECTION_SCHEMA,
        None,
    ),
    route_verb!(
        "datum.route.strategy_batch_evaluate",
        "Evaluate the current accepted M6 strategy surfaces across a versioned batch request manifest and return per-request evidence plus aggregate summary counts.",
        "route_strategy_batch_evaluate",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-strategy-batch-evaluate"),
            ArgvToken::Flag {
                flag: "--requests",
                param: "requests"
            },
        ],
        [p!("requests", Str, required)],
        r#"{"type":"object","properties":{"requests":{"type":"string"}},"required":["requests"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.strategy_compare",
        "Compare the accepted deterministic routing objectives/profiles, report the current live selector outcome for each, and recommend one profile under the approved comparison rule.",
        "route_strategy_compare",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-strategy-compare"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR],
        ROUTE_STRATEGY_SCHEMA,
        None,
    ),
    route_verb!(
        "datum.route.strategy_delta",
        "Report the bounded decision delta between the accepted deterministic routing objectives/profiles using the current live selector outcomes and one explicit delta classification.",
        "route_strategy_delta",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-strategy-delta"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
        ],
        [PATH, NET, FROM_ANCHOR, TO_ANCHOR],
        ROUTE_STRATEGY_SCHEMA,
        None,
    ),
    route_verb!(
        "datum.route.strategy_report",
        "Report which accepted selector profile should be used for one deterministic routing objective and show the current live selector outcome under that profile.",
        "route_strategy_report",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("route-strategy-report"),
            ROUTE_PREFIX[1],
            ROUTE_PREFIX[2],
            ROUTE_PREFIX[3],
            ROUTE_PREFIX[4],
            ArgvToken::Flag {
                flag: "--objective",
                param: "objective"
            },
        ],
        [
            PATH,
            NET,
            FROM_ANCHOR,
            TO_ANCHOR,
            p!("objective", Str, optional),
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"net_uuid":{"type":"string"},"from_anchor_pad_uuid":{"type":"string"},"to_anchor_pad_uuid":{"type":"string"},"objective":{"type":"string","enum":["default","authored-copper-priority"]}},"required":["path","net_uuid","from_anchor_pad_uuid","to_anchor_pad_uuid"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.summarize_strategy_batch_results",
        "Summarize saved route-strategy batch result artifacts from one directory or explicit list, with optional baseline gate summary.",
        "summarize_route_strategy_batch_results",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("summarize-route-strategy-batch-results"),
            ArgvToken::Flag {
                flag: "--dir",
                param: "dir"
            },
            ArgvToken::Repeated {
                flag: "--artifact",
                param: "artifacts"
            },
            ArgvToken::Flag {
                flag: "--baseline",
                param: "baseline"
            },
            ArgvToken::Flag {
                flag: "--policy",
                param: "policy"
            },
        ],
        [
            p!("dir", Str, optional),
            p!("artifacts", StrList, optional),
            p!("baseline", Str, optional),
            POLICY,
        ],
        r#"{"type":"object","properties":{"dir":{"type":"string"},"artifacts":{"type":"array","items":{"type":"string"}},"baseline":{"type":"string"},"policy":{"type":"string","enum":["strict_identical","allow_aggregate_only","fail_on_recommendation_change"]}}}"#,
        None,
    ),
    route_verb!(
        "datum.route.validate_strategy_batch_result",
        "Validate one saved versioned route-strategy batch result artifact for supported version, required fields, and summary/result count integrity.",
        "validate_route_strategy_batch_result",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("validate-route-strategy-batch-result"),
            ArgvToken::Param("artifact"),
        ],
        [ARTIFACT],
        r#"{"type":"object","properties":{"artifact":{"type":"string"}},"required":["artifact"]}"#,
        None,
    ),
    route_verb!(
        "datum.route.write_strategy_fixture_suite",
        "Write one deterministic generated-regression-fixture native-project suite plus a versioned batch request manifest for repeated route-strategy batch evidence runs; not a user design-authoring path.",
        "write_route_strategy_curated_fixture_suite",
        [
            ArgvToken::Lit("project"),
            ArgvToken::Lit("write-route-strategy-curated-fixture-suite"),
            ArgvToken::Flag {
                flag: "--out-dir",
                param: "out_dir"
            },
            ArgvToken::Flag {
                flag: "--manifest",
                param: "manifest"
            },
        ],
        [p!("out_dir", Str, required), p!("manifest", Str, optional)],
        r#"{"type":"object","properties":{"out_dir":{"type":"string"},"manifest":{"type":["string","null"]}},"required":["out_dir"]}"#,
        None,
    ),
];
