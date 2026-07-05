//! The `datum.proposal` terminal verb family (20 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_proposals.py` /
//! `tools_catalog_output_jobs.py` schemas via `tools_catalog_datum.py`
//! aliases), the Python bridge argv builders (`server_runtime.py` /
//! `server_runtime_proposals.py`), and cross-checked against the clap
//! definitions in `crates/cli/src/cli_args_proposals.rs` /
//! `cli_args_proposal_library.rs` / `cli_args_project_proposals.rs`.
//!
//! Only the verbs advertised in the GUI terminal command catalog are
//! registered here; the rest of the `datum.proposal` family migrates later.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus, WriteSurface};

const PROPOSAL_METADATA_WRITE: WriteSurface = WriteSurface {
    class: "proposal_metadata_write",
    evidence: "writes only persisted proposal metadata for later review; does not mutate design shards",
};

const PROPOSAL_REVIEW_STATE_WRITE: WriteSurface = WriteSurface {
    class: "proposal_review_state_write",
    evidence: "updates persisted proposal review state without applying design mutations",
};

const PROPOSAL_GATEWAY_APPLY: WriteSurface = WriteSurface {
    class: "proposal_gateway_apply",
    evidence: "applies an accepted proposal through the generic proposal journal gateway",
};

const PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: true,
    doc: "Project root directory",
    default_json: None,
};

const PROPOSAL_TARGET: ParamSpec = ParamSpec {
    name: "proposal",
    ty: ParamType::Uuid,
    required: true,
    doc: "Persisted proposal UUID",
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

const PROPOSAL_ID_ARGV: ArgvToken = ArgvToken::Flag {
    flag: "--proposal",
    param: "proposal",
};
const RATIONALE_ARGV: ArgvToken = ArgvToken::Flag {
    flag: "--rationale",
    param: "rationale",
};

macro_rules! proposal_param {
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
    ($name:literal, $ty:ident, default $default:literal) => {
        ParamSpec {
            name: $name,
            ty: ParamType::$ty,
            required: false,
            doc: $name,
            default_json: Some($default),
        }
    };
}

macro_rules! non_terminal_proposal_verb {
    (
        $id:literal,
        $summary:literal,
        $method:literal,
        [$($argv:expr),* $(,)?],
        [$($param:expr),* $(,)?],
        $schema:expr $(,)?
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
            write_surface: Some(PROPOSAL_METADATA_WRITE),
            terminal: false,
            terminal_optional_params: &[],
            terminal_argv_override: None,
        }
    };
}

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.proposal.accept_apply",
        summary: "Accept one draft native-project proposal and apply it through the generic proposal gateway.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "accept_apply_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("accept-apply"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_GATEWAY_APPLY),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.apply",
        summary: "Apply one accepted persisted native-project proposal through the generic proposal gateway.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "apply_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("apply"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_GATEWAY_APPLY),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.create_manufacturing_plan",
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
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--variant",
                    param: "variant",
                },
                ArgvToken::Flag {
                    flag: "--panel-projection",
                    param: "panel_projection",
                },
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
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.create_output_job",
        summary: "Create a draft proposal for an OutputJob creation without mutating the OutputJob shard.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_output_job_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("create-output-job"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
                ArgvToken::Flag {
                    flag: "--include",
                    param: "include",
                },
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--manufacturing-plan",
                    param: "manufacturing_plan",
                },
                ArgvToken::Flag {
                    flag: "--variant",
                    param: "variant",
                },
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
                doc: "Deterministic output prefix this job will generate",
                default_json: None,
            },
            ParamSpec {
                name: "include",
                ty: ParamType::Str,
                required: true,
                doc: "Artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Human-readable output job name",
                default_json: None,
            },
            ParamSpec {
                name: "manufacturing_plan",
                ty: ParamType::Uuid,
                required: false,
                doc: "Manufacturing plan UUID this output job executes",
                default_json: None,
            },
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: false,
                doc: "Preferred output directory for generated artifacts",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
            ParamSpec {
                name: "variant",
                ty: ParamType::Uuid,
                required: false,
                doc: "Variant overlay UUID this output job targets",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.create_panel_projection",
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
                ArgvToken::Flag {
                    flag: "--key",
                    param: "key",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--board",
                    param: "board",
                },
                ArgvToken::Flag {
                    flag: "--x-nm",
                    param: "x_nm",
                },
                ArgvToken::Flag {
                    flag: "--y-nm",
                    param: "y_nm",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
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
        schema_json_override: Some(
            r#"{"type":"object","properties":{"path":{"type":"string"},"key":{"type":"string"},"name":{"type":["string","null"]},"board":{"type":["string","null"]},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"rotation_deg":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","key"]}"#,
        ),
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.create_pool_pin_pad_map",
        summary: "Create a non-mutating draft proposal to author one first-class native pool PinPadMap.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_pin_pad_map_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("create-pool-pin-pad-map"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--map",
                    param: "map",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Switch {
                    flag: "--set-default",
                    param: "set_default",
                },
                ArgvToken::Repeated {
                    flag: "--entry",
                    param: "entries",
                },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "map",
                ty: ParamType::Uuid,
                required: true,
                doc: "PinPadMap UUID",
                default_json: None,
            },
            ParamSpec {
                name: "part",
                ty: ParamType::Uuid,
                required: true,
                doc: "Part UUID this PinPadMap binds",
                default_json: None,
            },
            ParamSpec {
                name: "entries",
                ty: ParamType::StrList,
                required: true,
                doc: "Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous",
                default_json: None,
            },
            ParamSpec {
                name: "footprint",
                ty: ParamType::Uuid,
                required: false,
                doc: "Optional Footprint UUID; if omitted mappings target package pads",
                default_json: None,
            },
            ParamSpec {
                name: "set_default",
                ty: ParamType::Bool,
                required: false,
                doc: "Also set this map as the part default_pin_pad_map in the same proposal batch",
                default_json: Some("false"),
            },
            ParamSpec {
                name: "pool",
                ty: ParamType::Str,
                required: false,
                doc: "Project-local pool path",
                default_json: Some("\"pool\""),
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &["pool", "rationale"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.defer",
        summary: "Defer one draft native-project proposal without applying it.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "defer_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("defer"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_REVIEW_STATE_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.delete_manufacturing_plan",
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
                ArgvToken::Flag {
                    flag: "--manufacturing-plan",
                    param: "manufacturing_plan",
                },
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
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.delete_output_job",
        summary: "Create a draft proposal for deleting one OutputJob without mutating the OutputJob shard.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_output_job_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("delete-output-job"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-job",
                    param: "output_job",
                },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_job",
                ty: ParamType::Uuid,
                required: true,
                doc: "OutputJob UUID",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.delete_panel_projection",
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
                ArgvToken::Flag {
                    flag: "--panel-projection",
                    param: "panel_projection",
                },
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
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.list",
        summary: "Read resolver-discovered proposal records for a native project.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_proposals",
            argv: &[
                ArgvToken::Lit("proposal"),
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
        id: "datum.proposal.preview",
        summary: "Preview one persisted native-project proposal's classified diff without writing shards.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "preview_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("preview"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.reject",
        summary: "Reject one draft native-project proposal without applying it.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "reject_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("reject"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_REVIEW_STATE_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.review",
        summary: "Review one persisted native-project proposal as accepted, deferred, or rejected.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "review_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("review"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
                ArgvToken::Flag {
                    flag: "--status",
                    param: "status",
                },
            ],
        },
        params: &[
            PATH,
            PROPOSAL_TARGET,
            ParamSpec {
                name: "status",
                ty: ParamType::Str,
                required: true,
                doc: "Review status to persist: accepted, deferred, or rejected",
                default_json: None,
            },
        ],
        schema_json_override: Some(
            r#"{"type":"object","properties":{"path":{"type":"string"},"proposal":{"type":"string"},"status":{"type":"string","enum":["accepted","deferred","rejected"]}},"required":["path","proposal","status"]}"#,
        ),
        write_surface: Some(PROPOSAL_REVIEW_STATE_WRITE),
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.set_pool_pin_pad_map",
        summary: "Create a non-mutating draft proposal to update first-class native pool PinPadMap mappings.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_pin_pad_map_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("set-pool-pin-pad-map"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--map",
                    param: "map",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--entry",
                    param: "entries",
                },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "map",
                ty: ParamType::Uuid,
                required: true,
                doc: "PinPadMap UUID",
                default_json: None,
            },
            ParamSpec {
                name: "mode",
                ty: ParamType::Str,
                required: false,
                doc: "Merge listed mappings or replace the full mapping table",
                default_json: Some("\"merge\""),
            },
            ParamSpec {
                name: "entries",
                ty: ParamType::StrList,
                required: true,
                doc: "Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous",
                default_json: None,
            },
            ParamSpec {
                name: "pool",
                ty: ParamType::Str,
                required: false,
                doc: "Project-local pool path",
                default_json: Some("\"pool\""),
            },
            PROPOSAL_ID,
            RATIONALE,
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &["pool", "mode", "rationale"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.show",
        summary: "Show one persisted native-project proposal plus validation state.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "show_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("show"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.update_manufacturing_plan",
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
                ArgvToken::Flag {
                    flag: "--manufacturing-plan",
                    param: "manufacturing_plan",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
                ArgvToken::Flag {
                    flag: "--variant",
                    param: "variant",
                },
                ArgvToken::Switch {
                    flag: "--clear-variant",
                    param: "clear_variant",
                },
                ArgvToken::Flag {
                    flag: "--panel-projection",
                    param: "panel_projection",
                },
                ArgvToken::Switch {
                    flag: "--clear-panel-projection",
                    param: "clear_panel_projection",
                },
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
        terminal: true,
        terminal_optional_params: &["name"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.update_output_job",
        summary: "Create a draft proposal for an OutputJob settings update without mutating the OutputJob shard.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "update_output_job_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("update-output-job"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-job",
                    param: "output_job",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                ArgvToken::Flag {
                    flag: "--manufacturing-plan",
                    param: "manufacturing_plan",
                },
                ArgvToken::Flag {
                    flag: "--variant",
                    param: "variant",
                },
                ArgvToken::Switch {
                    flag: "--clear-manufacturing-plan",
                    param: "clear_manufacturing_plan",
                },
                ArgvToken::Switch {
                    flag: "--clear-variant",
                    param: "clear_variant",
                },
                ArgvToken::Switch {
                    flag: "--clear-output-dir",
                    param: "clear_output_dir",
                },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_job",
                ty: ParamType::Uuid,
                required: true,
                doc: "OutputJob UUID",
                default_json: None,
            },
            ParamSpec {
                name: "name",
                ty: ParamType::Str,
                required: false,
                doc: "Replacement human-readable output job name",
                default_json: None,
            },
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: false,
                doc: "Replacement preferred output directory for generated artifacts",
                default_json: None,
            },
            ParamSpec {
                name: "manufacturing_plan",
                ty: ParamType::Uuid,
                required: false,
                doc: "Replacement manufacturing plan UUID this output job executes",
                default_json: None,
            },
            ParamSpec {
                name: "clear_manufacturing_plan",
                ty: ParamType::Bool,
                required: false,
                doc: "Clear any linked manufacturing plan",
                default_json: None,
            },
            ParamSpec {
                name: "clear_output_dir",
                ty: ParamType::Bool,
                required: false,
                doc: "Clear any stored output directory so launchers use their default",
                default_json: None,
            },
            PROPOSAL_ID,
            RATIONALE,
            ParamSpec {
                name: "variant",
                ty: ParamType::Uuid,
                required: false,
                doc: "Replacement variant overlay UUID this output job targets",
                default_json: None,
            },
            ParamSpec {
                name: "clear_variant",
                ty: ParamType::Bool,
                required: false,
                doc: "Clear any linked variant",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: true,
        terminal_optional_params: &["name"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.update_panel_projection",
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
                ArgvToken::Flag {
                    flag: "--panel-projection",
                    param: "panel_projection",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--board",
                    param: "board",
                },
                ArgvToken::Flag {
                    flag: "--x-nm",
                    param: "x_nm",
                },
                ArgvToken::Flag {
                    flag: "--y-nm",
                    param: "y_nm",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
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
        terminal: true,
        terminal_optional_params: &["name"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.proposal.validate",
        summary: "Validate one persisted native-project proposal against the current model revision.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "validate_proposal",
            argv: &[
                ArgvToken::Lit("proposal"),
                ArgvToken::Lit("validate"),
                ArgvToken::Param("path"),
                PROPOSAL_ID_ARGV,
            ],
        },
        params: &[PATH, PROPOSAL_TARGET],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    non_terminal_proposal_verb!(
        "datum.proposal.add_pool_footprint_silkscreen_circle",
        "Create a non-mutating draft proposal to append one native pool footprint silkscreen circle primitive.",
        "add_pool_footprint_silkscreen_circle_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("add-pool-footprint-silkscreen-circle"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--center-x-nm",
                param: "center_x_nm"
            },
            ArgvToken::Flag {
                flag: "--center-y-nm",
                param: "center_y_nm"
            },
            ArgvToken::Flag {
                flag: "--radius-nm",
                param: "radius_nm"
            },
            ArgvToken::Flag {
                flag: "--width-nm",
                param: "width_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("center_x_nm", Int, required),
            proposal_param!("center_y_nm", Int, required),
            proposal_param!("radius_nm", Int, required),
            proposal_param!("width_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"width_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","center_x_nm","center_y_nm","radius_nm","width_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.add_pool_footprint_silkscreen_line",
        "Create a non-mutating draft proposal to append one native pool footprint silkscreen line primitive.",
        "add_pool_footprint_silkscreen_line_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("add-pool-footprint-silkscreen-line"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--from-x-nm",
                param: "from_x_nm"
            },
            ArgvToken::Flag {
                flag: "--from-y-nm",
                param: "from_y_nm"
            },
            ArgvToken::Flag {
                flag: "--to-x-nm",
                param: "to_x_nm"
            },
            ArgvToken::Flag {
                flag: "--to-y-nm",
                param: "to_y_nm"
            },
            ArgvToken::Flag {
                flag: "--width-nm",
                param: "width_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("from_x_nm", Int, required),
            proposal_param!("from_y_nm", Int, required),
            proposal_param!("to_x_nm", Int, required),
            proposal_param!("to_y_nm", Int, required),
            proposal_param!("width_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"width_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","from_x_nm","from_y_nm","to_x_nm","to_y_nm","width_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.add_pool_footprint_silkscreen_polygon",
        "Create a non-mutating draft proposal to append one native pool footprint silkscreen polygon or polyline primitive.",
        "add_pool_footprint_silkscreen_polygon_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("add-pool-footprint-silkscreen-polygon"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--vertices",
                param: "vertices"
            },
            ArgvToken::Flag {
                flag: "--closed",
                param: "closed"
            },
            ArgvToken::Flag {
                flag: "--width-nm",
                param: "width_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("vertices", Str, required),
            proposal_param!("closed", Bool, required),
            proposal_param!("width_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"vertices":{"type":"string"},"closed":{"type":"boolean"},"width_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","vertices","closed","width_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.add_pool_footprint_silkscreen_rect",
        "Create a non-mutating draft proposal to append one native pool footprint silkscreen rectangle primitive.",
        "add_pool_footprint_silkscreen_rect_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("add-pool-footprint-silkscreen-rect"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--min-x-nm",
                param: "min_x_nm"
            },
            ArgvToken::Flag {
                flag: "--min-y-nm",
                param: "min_y_nm"
            },
            ArgvToken::Flag {
                flag: "--max-x-nm",
                param: "max_x_nm"
            },
            ArgvToken::Flag {
                flag: "--max-y-nm",
                param: "max_y_nm"
            },
            ArgvToken::Flag {
                flag: "--width-nm",
                param: "width_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("min_x_nm", Int, required),
            proposal_param!("min_y_nm", Int, required),
            proposal_param!("max_x_nm", Int, required),
            proposal_param!("max_y_nm", Int, required),
            proposal_param!("width_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"width_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","min_x_nm","min_y_nm","max_x_nm","max_y_nm","width_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create",
        "Create a non-mutating draft proposal from an OperationBatch JSON file.",
        "create_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--batch",
                param: "batch"
            },
            ArgvToken::Flag {
                flag: "--rationale",
                param: "rationale"
            },
            PROPOSAL_ID_ARGV,
            ArgvToken::Flag {
                flag: "--source",
                param: "source"
            },
            ArgvToken::Repeated {
                flag: "--check-run",
                param: "checks_run"
            },
            ArgvToken::Repeated {
                flag: "--finding-fingerprint",
                param: "finding_fingerprints"
            },
        ],
        [
            PATH,
            proposal_param!("batch", Str, required),
            proposal_param!("rationale", Str, required),
            PROPOSAL_ID,
            proposal_param!("source", Str, default "\"tool\""),
            proposal_param!("checks_run", StrList, default "[]"),
            proposal_param!("finding_fingerprints", StrList, default "[]"),
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"batch":{"type":"string"},"rationale":{"type":"string"},"proposal":{"type":"string"},"source":{"type":"string","enum":["manual","cli","tool","assistant","check","import"]},"checks_run":{"type":"array","items":{"type":"string"}},"finding_fingerprints":{"type":"array","items":{"type":"string"}}},"required":["path","batch","rationale"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_board_component_replacement",
        "Create a non-mutating draft proposal to replace one native-project board component package, part, and/or value.",
        "create_board_component_replacement_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-board-component-replacement"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--component",
                param: "component"
            },
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--part",
                param: "part"
            },
            ArgvToken::Flag {
                flag: "--value",
                param: "value"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("component", Uuid, required),
            proposal_param!("package", Uuid, optional),
            proposal_param!("part", Uuid, optional),
            proposal_param!("value", Str, optional),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"package":{"type":["string","null"]},"part":{"type":["string","null"]},"value":{"type":["string","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","component"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_board_component_replacement_plan",
        "Create one non-mutating draft proposal from replacement-plan shaped component selections.",
        "create_board_component_replacement_plan_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-board-component-replacement-plan"),
            ArgvToken::Param("path"),
            ArgvToken::Repeated {
                flag: "--selection",
                param: "selections"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("selections", Json, required),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"selections":{"type":"array","items":{"type":"object","properties":{"uuid":{"type":"string"},"package_uuid":{"type":["string","null"]},"part_uuid":{"type":["string","null"]},"value":{"type":["string","null"]}},"required":["uuid"]}},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","selections"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_board_component_replacements",
        "Create one non-mutating draft proposal to replace multiple native-project board component package, part, and/or value sets.",
        "create_board_component_replacements_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-board-component-replacements"),
            ArgvToken::Param("path"),
            ArgvToken::Repeated {
                flag: "--replacement",
                param: "replacements"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("replacements", Json, required),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"replacements":{"type":"array","items":{"type":"object","properties":{"component":{"type":"string"},"package":{"type":["string","null"]},"part":{"type":["string","null"]},"value":{"type":["string","null"]}},"required":["component"]}},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","replacements"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_draw_wire",
        "Create a non-mutating draft proposal to draw one native-project schematic wire.",
        "create_draw_wire_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-draw-wire"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--sheet",
                param: "sheet"
            },
            ArgvToken::Flag {
                flag: "--from-x-nm",
                param: "from_x_nm"
            },
            ArgvToken::Flag {
                flag: "--from-y-nm",
                param: "from_y_nm"
            },
            ArgvToken::Flag {
                flag: "--to-x-nm",
                param: "to_x_nm"
            },
            ArgvToken::Flag {
                flag: "--to-y-nm",
                param: "to_y_nm"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("sheet", Uuid, required),
            proposal_param!("from_x_nm", Int, required),
            proposal_param!("from_y_nm", Int, required),
            proposal_param!("to_x_nm", Int, required),
            proposal_param!("to_y_nm", Int, required),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","sheet","from_x_nm","from_y_nm","to_x_nm","to_y_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_place_label",
        "Create a non-mutating draft proposal to place one native-project schematic label.",
        "create_place_label_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-place-label"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--sheet",
                param: "sheet"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--x-nm",
                param: "x_nm"
            },
            ArgvToken::Flag {
                flag: "--y-nm",
                param: "y_nm"
            },
            ArgvToken::Flag {
                flag: "--kind",
                param: "kind"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("sheet", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("x_nm", Int, required),
            proposal_param!("y_nm", Int, required),
            proposal_param!("kind", Str, optional),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"kind":{"type":["string","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","sheet","name","x_nm","y_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_place_symbol",
        "Create a non-mutating draft proposal to place one native-project schematic symbol.",
        "create_place_symbol_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-place-symbol"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--sheet",
                param: "sheet"
            },
            ArgvToken::Flag {
                flag: "--reference",
                param: "reference"
            },
            ArgvToken::Flag {
                flag: "--value",
                param: "value"
            },
            ArgvToken::Flag {
                flag: "--x-nm",
                param: "x_nm"
            },
            ArgvToken::Flag {
                flag: "--y-nm",
                param: "y_nm"
            },
            ArgvToken::Flag {
                flag: "--lib-id",
                param: "lib_id"
            },
            ArgvToken::Flag {
                flag: "--rotation-deg",
                param: "rotation_deg"
            },
            ArgvToken::Switch {
                flag: "--mirrored",
                param: "mirrored"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("sheet", Uuid, required),
            proposal_param!("reference", Str, required),
            proposal_param!("value", Str, required),
            proposal_param!("x_nm", Int, required),
            proposal_param!("y_nm", Int, required),
            proposal_param!("lib_id", Str, optional),
            proposal_param!("rotation_deg", Int, optional),
            proposal_param!("mirrored", Bool, optional),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"reference":{"type":"string"},"value":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"lib_id":{"type":["string","null"]},"rotation_deg":{"type":["integer","null"]},"mirrored":{"type":["boolean","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","sheet","reference","value","x_nm","y_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_entity",
        "Create a non-mutating draft proposal to author one native pool entity for an existing unit/symbol pair.",
        "create_pool_entity_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-entity"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--entity",
                param: "entity"
            },
            ArgvToken::Flag {
                flag: "--gate",
                param: "gate"
            },
            ArgvToken::Flag {
                flag: "--unit",
                param: "unit"
            },
            ArgvToken::Flag {
                flag: "--symbol",
                param: "symbol"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--prefix",
                param: "prefix"
            },
            ArgvToken::Flag {
                flag: "--manufacturer",
                param: "manufacturer"
            },
            ArgvToken::Flag {
                flag: "--gate-name",
                param: "gate_name"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("entity", Uuid, required),
            proposal_param!("gate", Uuid, required),
            proposal_param!("unit", Uuid, required),
            proposal_param!("symbol", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("prefix", Str, required),
            proposal_param!("manufacturer", Str, default "\"\""),
            proposal_param!("gate_name", Str, default "\"A\""),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"entity":{"type":"string"},"gate":{"type":"string"},"unit":{"type":"string"},"symbol":{"type":"string"},"name":{"type":"string"},"prefix":{"type":"string"},"manufacturer":{"type":["string","null"]},"gate_name":{"type":["string","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","entity","gate","unit","symbol","name","prefix"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_footprint",
        "Create a non-mutating draft proposal to author one native pool footprint.",
        "create_pool_footprint_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-footprint"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("package", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"package":{"type":"string"},"name":{"type":"string"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","package","name"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_library_object",
        "Create a non-mutating draft proposal to author one raw native pool-library object.",
        "create_pool_library_object_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-library-object"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--kind",
                param: "kind"
            },
            ArgvToken::Flag {
                flag: "--object",
                param: "object"
            },
            ArgvToken::Flag {
                flag: "--from-json",
                param: "from_json"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("kind", Str, required),
            proposal_param!("object", Uuid, required),
            proposal_param!("from_json", Str, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"kind":{"type":"string"},"object":{"type":"string"},"from_json":{"type":"string"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","kind","object","from_json"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_package",
        "Create a non-mutating draft proposal to author one native pool package body record; optional pad/padstack fields are legacy land-pattern compatibility input.",
        "create_pool_package_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-package"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--pad",
                param: "pad"
            },
            ArgvToken::Flag {
                flag: "--padstack",
                param: "padstack"
            },
            ArgvToken::Flag {
                flag: "--pad-name",
                param: "pad_name"
            },
            ArgvToken::Flag {
                flag: "--x-nm",
                param: "x_nm"
            },
            ArgvToken::Flag {
                flag: "--y-nm",
                param: "y_nm"
            },
            ArgvToken::Flag {
                flag: "--layer",
                param: "layer"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("package", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("pad", Uuid, optional),
            proposal_param!("padstack", Uuid, optional),
            proposal_param!("pad_name", Str, default "\"1\""),
            proposal_param!("x_nm", Int, default "0"),
            proposal_param!("y_nm", Int, default "0"),
            proposal_param!("layer", Int, default "1"),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"package":{"type":"string"},"name":{"type":"string"},"pad":{"type":["string","null"]},"padstack":{"type":["string","null"]},"pad_name":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","package","name"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_padstack",
        "Create a non-mutating draft proposal to author one native pool padstack.",
        "create_pool_padstack_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-padstack"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--padstack",
                param: "padstack"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--aperture",
                param: "aperture"
            },
            ArgvToken::Flag {
                flag: "--diameter-nm",
                param: "diameter_nm"
            },
            ArgvToken::Flag {
                flag: "--width-nm",
                param: "width_nm"
            },
            ArgvToken::Flag {
                flag: "--height-nm",
                param: "height_nm"
            },
            ArgvToken::Flag {
                flag: "--drill-nm",
                param: "drill_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("padstack", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("aperture", Str, optional),
            proposal_param!("diameter_nm", Int, optional),
            proposal_param!("width_nm", Int, optional),
            proposal_param!("height_nm", Int, optional),
            proposal_param!("drill_nm", Int, optional),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"padstack":{"type":"string"},"name":{"type":"string"},"aperture":{"type":["string","null"]},"diameter_nm":{"type":["integer","null"]},"width_nm":{"type":["integer","null"]},"height_nm":{"type":["integer","null"]},"drill_nm":{"type":["integer","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","padstack","name"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_symbol",
        "Create a non-mutating draft proposal to author one native pool symbol for an existing pool unit.",
        "create_pool_symbol_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-symbol"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--symbol",
                param: "symbol"
            },
            ArgvToken::Flag {
                flag: "--unit",
                param: "unit"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("symbol", Uuid, required),
            proposal_param!("unit", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"symbol":{"type":"string"},"unit":{"type":"string"},"name":{"type":"string"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","symbol","unit","name"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.create_pool_unit",
        "Create a non-mutating draft proposal to author one native pool unit.",
        "create_pool_unit_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("create-pool-unit"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--unit",
                param: "unit"
            },
            ArgvToken::Flag {
                flag: "--name",
                param: "name"
            },
            ArgvToken::Flag {
                flag: "--manufacturer",
                param: "manufacturer"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("unit", Uuid, required),
            proposal_param!("name", Str, required),
            proposal_param!("manufacturer", Str, default "\"\""),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"unit":{"type":"string"},"name":{"type":"string"},"manufacturer":{"type":["string","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","unit","name"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_footprint_courtyard_polygon",
        "Create a non-mutating draft proposal to set polygon native pool footprint courtyard geometry.",
        "set_pool_footprint_courtyard_polygon_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-footprint-courtyard-polygon"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--vertices",
                param: "vertices"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("vertices", Str, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"vertices":{"type":"string"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","vertices"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_footprint_courtyard_rect",
        "Create a non-mutating draft proposal to set rectangular native pool footprint courtyard geometry.",
        "set_pool_footprint_courtyard_rect_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-footprint-courtyard-rect"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--min-x-nm",
                param: "min_x_nm"
            },
            ArgvToken::Flag {
                flag: "--min-y-nm",
                param: "min_y_nm"
            },
            ArgvToken::Flag {
                flag: "--max-x-nm",
                param: "max_x_nm"
            },
            ArgvToken::Flag {
                flag: "--max-y-nm",
                param: "max_y_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("min_x_nm", Int, required),
            proposal_param!("min_y_nm", Int, required),
            proposal_param!("max_x_nm", Int, required),
            proposal_param!("max_y_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","min_x_nm","min_y_nm","max_x_nm","max_y_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_footprint_pad",
        "Create a non-mutating draft proposal to set one native pool footprint pad entry.",
        "set_pool_footprint_pad_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-footprint-pad"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--footprint",
                param: "footprint"
            },
            ArgvToken::Flag {
                flag: "--pad",
                param: "pad"
            },
            ArgvToken::Flag {
                flag: "--padstack",
                param: "padstack"
            },
            ArgvToken::Flag {
                flag: "--pad-name",
                param: "pad_name"
            },
            ArgvToken::Flag {
                flag: "--x-nm",
                param: "x_nm"
            },
            ArgvToken::Flag {
                flag: "--y-nm",
                param: "y_nm"
            },
            ArgvToken::Flag {
                flag: "--layer",
                param: "layer"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("footprint", Uuid, required),
            proposal_param!("pad", Uuid, required),
            proposal_param!("padstack", Uuid, required),
            proposal_param!("pad_name", Str, default "\"1\""),
            proposal_param!("x_nm", Int, default "0"),
            proposal_param!("y_nm", Int, default "0"),
            proposal_param!("layer", Int, default "1"),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"footprint":{"type":"string"},"pad":{"type":"string"},"padstack":{"type":"string"},"pad_name":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","footprint","pad","padstack"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_package_courtyard_polygon",
        "Create a non-mutating draft proposal to set polygon native pool package courtyard geometry.",
        "set_pool_package_courtyard_polygon_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-package-courtyard-polygon"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--vertices",
                param: "vertices"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("package", Uuid, required),
            proposal_param!("vertices", Str, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"package":{"type":"string"},"vertices":{"type":"string"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","package","vertices"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_package_courtyard_rect",
        "Create a non-mutating draft proposal to set rectangular native pool package courtyard geometry.",
        "set_pool_package_courtyard_rect_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-package-courtyard-rect"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--min-x-nm",
                param: "min_x_nm"
            },
            ArgvToken::Flag {
                flag: "--min-y-nm",
                param: "min_y_nm"
            },
            ArgvToken::Flag {
                flag: "--max-x-nm",
                param: "max_x_nm"
            },
            ArgvToken::Flag {
                flag: "--max-y-nm",
                param: "max_y_nm"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("package", Uuid, required),
            proposal_param!("min_x_nm", Int, required),
            proposal_param!("min_y_nm", Int, required),
            proposal_param!("max_x_nm", Int, required),
            proposal_param!("max_y_nm", Int, required),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"package":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","package","min_x_nm","min_y_nm","max_x_nm","max_y_nm"]}"#,
    ),
    non_terminal_proposal_verb!(
        "datum.proposal.set_pool_package_pad",
        "Create a non-mutating draft proposal to add one native pool package pad entry.",
        "set_pool_package_pad_proposal",
        [
            ArgvToken::Lit("proposal"),
            ArgvToken::Lit("set-pool-package-pad"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--package",
                param: "package"
            },
            ArgvToken::Flag {
                flag: "--pad",
                param: "pad"
            },
            ArgvToken::Flag {
                flag: "--padstack",
                param: "padstack"
            },
            ArgvToken::Flag {
                flag: "--pad-name",
                param: "pad_name"
            },
            ArgvToken::Flag {
                flag: "--x-nm",
                param: "x_nm"
            },
            ArgvToken::Flag {
                flag: "--y-nm",
                param: "y_nm"
            },
            ArgvToken::Flag {
                flag: "--layer",
                param: "layer"
            },
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool"
            },
            PROPOSAL_ID_ARGV,
            RATIONALE_ARGV,
        ],
        [
            PATH,
            proposal_param!("package", Uuid, required),
            proposal_param!("pad", Uuid, required),
            proposal_param!("padstack", Uuid, required),
            proposal_param!("pad_name", Str, default "\"1\""),
            proposal_param!("x_nm", Int, default "0"),
            proposal_param!("y_nm", Int, default "0"),
            proposal_param!("layer", Int, default "1"),
            proposal_param!("pool", Str, default "\"pool\""),
            PROPOSAL_ID,
            RATIONALE,
        ],
        r#"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"package":{"type":"string"},"pad":{"type":"string"},"padstack":{"type":"string"},"pad_name":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]}},"required":["path","package","pad","padstack"]}"#,
    ),
];
