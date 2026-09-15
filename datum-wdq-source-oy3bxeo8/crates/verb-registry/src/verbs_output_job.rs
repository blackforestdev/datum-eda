//! The `datum.output_job` verb family (5 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_output_jobs.py` schemas via
//! `tools_catalog_datum.py` aliases). `create`/`update`/`delete` are the
//! output-job-named twins of their `datum.proposal.*` counterparts;
//! `create_gerber_set` presets `include=gerber-set` through dispatch
//! defaults; `run` executes one authored OutputJob through the canonical
//! artifact-generate CLI path. None is advertised in the GUI terminal
//! catalog.
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

const PROPOSAL_ID_ARGV: ArgvToken = ArgvToken::Flag {
    flag: "--proposal",
    param: "proposal",
};
const RATIONALE_ARGV: ArgvToken = ArgvToken::Flag {
    flag: "--rationale",
    param: "rationale",
};

/// Exact hand-written MCP schema (`create_gerber_output_job`): the preset
/// `include` dispatch default and the `proposal`/`rationale` dispatch-only
/// arguments are not advertised as schema properties there.
const CREATE_GERBER_SET_SCHEMA: &str = r#"{"type":"object","properties":{"path":{"type":"string"},"prefix":{"type":"string"},"name":{"type":["string","null"]},"manufacturing_plan":{"type":["string","null"]},"variant":{"type":["string","null"]},"output_dir":{"type":["string","null"]}},"required":["path","prefix"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.output_job.create",
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
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.output_job.create_gerber_set",
        summary: "Create a draft proposal for the deterministic Gerber-set OutputJob without mutating the OutputJob shard.",
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
                // `include` is always injected as the gerber-set preset via
                // the dispatch default, so it is a literal argv pair here.
                ArgvToken::Lit("--include"),
                ArgvToken::Lit("gerber-set"),
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--manufacturing-plan",
                    param: "manufacturing_plan",
                },
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                PROPOSAL_ID_ARGV,
                RATIONALE_ARGV,
                ArgvToken::Flag {
                    flag: "--variant",
                    param: "variant",
                },
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
                required: false,
                doc: "Artifact include scope preset; always the deterministic gerber-set",
                default_json: Some("\"gerber-set\""),
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
        schema_json_override: Some(CREATE_GERBER_SET_SCHEMA),
        write_surface: Some(PROPOSAL_METADATA_WRITE),
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.output_job.delete",
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
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.output_job.run",
        summary: "Execute one authored OutputJob using its stored include, prefix, and output directory settings.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "run_output_job",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("generate"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-job",
                    param: "output_job",
                },
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_job",
                ty: ParamType::Uuid,
                required: true,
                doc: "OutputJob UUID to execute",
                default_json: None,
            },
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: false,
                doc: "Output directory override for this run",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.output_job.update",
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
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
