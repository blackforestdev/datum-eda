//! The `datum.manufacturing` verb family (6 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_output_jobs.py` schemas via
//! `tools_catalog_datum.py` aliases). Each verb is the manufacturing-named
//! twin of its `datum.proposal.*` counterpart: identical summary, dispatch
//! method, parameter order, and proposal-metadata write surface — only the
//! canonical id differs, and none is advertised in the GUI terminal catalog.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus, WriteSurface};

const PROPOSAL_METADATA_WRITE: WriteSurface = WriteSurface {
    class: "proposal_metadata_write",
    evidence: "writes only persisted proposal metadata for later review; does not mutate design shards",
};

const PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: true,
    doc: "Project root directory",
    default_json: None,
};

const PROPOSAL_ID: ParamSpec = ParamSpec {
    name: "proposal",
    ty: ParamType::Uuid,
    required: false,
    doc: "Optional stable proposal UUID",
    default_json: None,
};

const RATIONALE: ParamSpec = ParamSpec {
    name: "rationale",
    ty: ParamType::Str,
    required: false,
    doc: "Proposal review rationale",
    default_json: None,
};

const PROPOSAL_ID_ARGV: ArgvToken = ArgvToken::Flag { flag: "--proposal", param: "proposal" };
const RATIONALE_ARGV: ArgvToken = ArgvToken::Flag { flag: "--rationale", param: "rationale" };

/// Exact hand-written MCP schema: `x_nm`/`y_nm`/`rotation_deg` are optional
/// but declared non-nullable there, which the ParamSpec derivation (optional
/// => nullable) cannot express.
const CREATE_PANEL_PROJECTION_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"key":{"type":"string"},"name":{"type":["string","null"]},"board":{"type":["string","null"]},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"rotation_deg":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","key"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.manufacturing.create_panel_projection",
        summary: "Create a draft proposal for a PanelProjection creation without mutating panel shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_panel_projection_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("create-panel-projection"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--key", param: "key" },
                ArgvToken::Flag { flag: "--name", param: "name" },
                ArgvToken::Flag { flag: "--board", param: "board" },
                ArgvToken::Flag { flag: "--x-nm", param: "x_nm" },
                ArgvToken::Flag { flag: "--y-nm", param: "y_nm" },
                ArgvToken::Flag { flag: "--rotation-deg", param: "rotation_deg" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "key",
                ty: ParamType::Str,
                required: true,
                doc: "Deterministic panel key",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Human-readable panel projection name",
                default_json: None,
            },
            ParamSpec {
                name: "board",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional board UUID for the first panel instance; defaults to current board",
                default_json: None,
            },
            ParamSpec {
                name: "x_nm",
                ty: ParamType::Int,
                required: false,
                doc: "First board instance X offset in nanometers",
                default_json: None,
            },
            ParamSpec {
                name: "y_nm",
                ty: ParamType::Int,
                required: false,
                doc: "First board instance Y offset in nanometers",
                default_json: None,
            },
            ParamSpec {
                name: "rotation_deg",
                ty: ParamType::Int,
                required: false,
                doc: "First board instance rotation in degrees",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: Some(CREATE_PANEL_PROJECTION_SCHEMA),
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.manufacturing.create_plan",
        summary: "Create a draft proposal for a ManufacturingPlan creation without mutating manufacturing plan shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_manufacturing_plan_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("create-manufacturing-plan"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--prefix", param: "prefix" },
                ArgvToken::Flag { flag: "--name", param: "name" },
                ArgvToken::Flag { flag: "--variant", param: "variant" },
                ArgvToken::Flag { flag: "--panel-projection", param: "panel_projection" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "prefix",
                ty: ParamType::Str,
                required: true,
                doc: "Deterministic manufacturing artifact filename prefix",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Human-readable manufacturing plan name",
                default_json: None,
            },
            ParamSpec {
                name: "variant",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional variant UUID this plan targets",
                default_json: None,
            },
            ParamSpec {
                name: "panel_projection",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional panel projection UUID this plan targets instead of the board",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.manufacturing.delete_panel_projection",
        summary: "Create a draft proposal for deleting one PanelProjection without mutating panel shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_panel_projection_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("delete-panel-projection"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--panel-projection", param: "panel_projection" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "panel_projection",
                ty: ParamType::Uuid,
                required: true,
                doc: "PanelProjection UUID to delete",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.manufacturing.delete_plan",
        summary: "Create a draft proposal for deleting one ManufacturingPlan without mutating manufacturing plan shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_manufacturing_plan_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("delete-manufacturing-plan"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--manufacturing-plan", param: "manufacturing_plan" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "manufacturing_plan",
                ty: ParamType::Uuid,
                required: true,
                doc: "ManufacturingPlan UUID to delete",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.manufacturing.update_panel_projection",
        summary: "Create a draft proposal for a PanelProjection update without mutating panel shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "update_panel_projection_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("update-panel-projection"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--panel-projection", param: "panel_projection" },
                ArgvToken::Flag { flag: "--name", param: "name" },
                ArgvToken::Flag { flag: "--board", param: "board" },
                ArgvToken::Flag { flag: "--x-nm", param: "x_nm" },
                ArgvToken::Flag { flag: "--y-nm", param: "y_nm" },
                ArgvToken::Flag { flag: "--rotation-deg", param: "rotation_deg" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "panel_projection",
                ty: ParamType::Uuid,
                required: true,
                doc: "PanelProjection UUID to update",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Replacement human-readable panel projection name",
                default_json: None,
            },
            ParamSpec {
                name: "board",
                ty: ParamType::Uuid,
                required: false,
                doc: "Replacement board UUID for the first panel instance",
                default_json: None,
            },
            ParamSpec {
                name: "x_nm",
                ty: ParamType::Int,
                required: false,
                doc: "Replacement first board instance X offset in nanometers",
                default_json: None,
            },
            ParamSpec {
                name: "y_nm",
                ty: ParamType::Int,
                required: false,
                doc: "Replacement first board instance Y offset in nanometers",
                default_json: None,
            },
            ParamSpec {
                name: "rotation_deg",
                ty: ParamType::Int,
                required: false,
                doc: "Replacement first board instance rotation in degrees",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.manufacturing.update_plan",
        summary: "Create a draft proposal for a ManufacturingPlan update without mutating manufacturing plan shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "update_manufacturing_plan_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("update-manufacturing-plan"),
                ArgvToken::Param("path"),
                ArgvToken::Flag { flag: "--manufacturing-plan", param: "manufacturing_plan" },
                ArgvToken::Flag { flag: "--name", param: "name" },
                ArgvToken::Flag { flag: "--prefix", param: "prefix" },
                ArgvToken::Flag { flag: "--variant", param: "variant" },
                ArgvToken::Switch { flag: "--clear-variant", param: "clear_variant" },
                ArgvToken::Flag { flag: "--panel-projection", param: "panel_projection" },
                ArgvToken::Switch { flag: "--clear-panel-projection", param: "clear_panel_projection" },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "manufacturing_plan",
                ty: ParamType::Uuid,
                required: true,
                doc: "ManufacturingPlan UUID to update",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Replacement human-readable manufacturing plan name",
                default_json: None,
            },
            ParamSpec {
                name: "prefix",
                ty: ParamType::Str,
                required: false,
                doc: "Replacement deterministic manufacturing artifact filename prefix",
                default_json: None,
            },
            ParamSpec {
                name: "variant",
                ty: ParamType::Uuid,
                required: false,
                doc: "Replacement variant UUID this plan targets",
                default_json: None,
            },
            ParamSpec {
                name: "clear_variant",
                ty: ParamType::Bool,
                required: false,
                doc: "Clear the variant target",
                default_json: None,
            },
            ParamSpec {
                name: "panel_projection",
                ty: ParamType::Uuid,
                required: false,
                doc: "Replacement panel projection UUID this plan targets",
                default_json: None,
            },
            ParamSpec {
                name: "clear_panel_projection",
                ty: ParamType::Bool,
                required: false,
                doc: "Clear the panel target and retarget the current board",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
