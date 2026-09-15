//! The `datum.artifact` verb family (11 verbs), transcribed from the
//! hand-written MCP catalog (`tools_catalog_output_jobs.py` schemas), the
//! Python bridge argv builders (`server_runtime.py`), and cross-checked
//! against the clap definitions in `crates/cli/src/cli_args_artifact.rs` and
//! `cli_args_manufacturing.rs`.
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

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.artifact.cancel_output_job_run",
        summary: "Mark one existing OutputJobRun evidence record canceled.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "cancel_output_job_run",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("cancel-output-job-run"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--run",
                    param: "run",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "run",
                ty: ParamType::Uuid,
                required: true,
                doc: "OutputJobRun UUID to mark canceled",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.compare",
        summary: "Compare two resolver-discovered generated artifact metadata records for a native project.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "compare_artifacts",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("compare"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--before",
                    param: "before",
                },
                ArgvToken::Flag {
                    flag: "--after",
                    param: "after",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "before",
                ty: ParamType::Uuid,
                required: true,
                doc: "Baseline artifact UUID",
                default_json: None,
            },
            ParamSpec {
                name: "after",
                ty: ParamType::Uuid,
                required: true,
                doc: "Candidate artifact UUID",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.export_manufacturing_set",
        summary: "Export the current supported manufacturing set and persist resolver-owned artifact/run evidence.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "export_manufacturing_set",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("export-manufacturing-set"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: true,
                doc: "Directory to write the current supported manufacturing set into",
                default_json: None,
            },
            ParamSpec {
                name: "prefix",
                ty: ParamType::Str,
                required: false,
                doc: "Optional artifact filename prefix; defaults to the board name",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.files",
        summary: "Return generated files and production projection proofs for one artifact.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_artifact_files",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("files"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--artifact",
                    param: "artifact",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "artifact",
                ty: ParamType::Uuid,
                required: true,
                doc: "Artifact UUID to inspect",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.generate",
        summary: "Generate derived production artifacts from include scopes for a native project, or execute one authored OutputJob by id.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "generate_artifacts",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("generate"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                ArgvToken::Flag {
                    flag: "--include",
                    param: "include",
                },
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
                ArgvToken::Flag {
                    flag: "--output-job",
                    param: "output_job",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: false,
                doc: "Output directory for generated artifact files, or an authored OutputJob run override",
                default_json: None,
            },
            ParamSpec {
                name: "include",
                ty: ParamType::Str,
                required: false,
                doc: "Comma-separated include scopes: gerber-set, manufacturing-set, bom, pnp, drill, or all",
                default_json: None,
            },
            ParamSpec {
                name: "prefix",
                ty: ParamType::Str,
                required: false,
                doc: "Optional output filename prefix",
                default_json: None,
            },
            ParamSpec {
                name: "output_job",
                ty: ParamType::Uuid,
                required: false,
                doc: "Execute one authored OutputJob instead of direct include-scope generation; conflicts with include/prefix",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        // The GUI advertises `artifact generate` as the authored OutputJob
        // launcher, so `--output-job` is the one advertised optional flag.
        terminal_optional_params: &["output_job"],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.list",
        summary: "Return resolver-discovered generated artifact metadata for one native project.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_artifacts",
            argv: &[
                ArgvToken::Lit("artifact"),
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
        id: "datum.artifact.preview",
        summary: "Preview one generated artifact file through supported semantic readers.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "preview_artifact_file",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("preview"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--artifact",
                    param: "artifact",
                },
                ArgvToken::Flag {
                    flag: "--artifact-dir",
                    param: "artifact_dir",
                },
                ArgvToken::Flag {
                    flag: "--file",
                    param: "file",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "artifact",
                ty: ParamType::Uuid,
                required: true,
                doc: "Artifact UUID that owns the generated file",
                default_json: None,
            },
            ParamSpec {
                name: "artifact_dir",
                ty: ParamType::Str,
                required: false,
                doc: "Directory containing generated artifact files; defaults to stored artifact output_dir",
                default_json: None,
            },
            ParamSpec {
                name: "file",
                ty: ParamType::Str,
                required: true,
                doc: "Relative generated file path from the artifact metadata",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.show",
        summary: "Return one resolver-discovered generated artifact metadata record for a native project.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "show_artifact",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("show"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--artifact",
                    param: "artifact",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "artifact",
                ty: ParamType::Uuid,
                required: true,
                doc: "Artifact UUID to inspect",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.start_output_job_run",
        summary: "Persist a running OutputJobRun evidence record for one authored OutputJob.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "start_output_job_run",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("start-output-job-run"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-job",
                    param: "output_job",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_job",
                ty: ParamType::Uuid,
                required: true,
                doc: "OutputJob UUID to mark running",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.validate",
        summary: "Validate one resolver-discovered generated artifact metadata record for a native project.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "validate_artifact",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("validate"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--artifact",
                    param: "artifact",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "artifact",
                ty: ParamType::Uuid,
                required: true,
                doc: "Artifact UUID to validate",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.artifact.validate_manufacturing_set",
        summary: "Validate an exported manufacturing set against current project state and persisted artifact hashes.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "validate_manufacturing_set",
            argv: &[
                ArgvToken::Lit("artifact"),
                ArgvToken::Lit("validate-manufacturing-set"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--output-dir",
                    param: "output_dir",
                },
                ArgvToken::Flag {
                    flag: "--prefix",
                    param: "prefix",
                },
            ],
        },
        params: &[
            PATH,
            ParamSpec {
                name: "output_dir",
                ty: ParamType::Str,
                required: true,
                doc: "Directory to validate against the current supported manufacturing set",
                default_json: None,
            },
            ParamSpec {
                name: "prefix",
                ty: ParamType::Str,
                required: false,
                doc: "Optional artifact filename prefix; defaults to the board name",
                default_json: None,
            },
        ],
        schema_json_override: None,
        write_surface: None,
        terminal: true,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
