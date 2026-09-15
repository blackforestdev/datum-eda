//! The `datum.library` verb family (54 verbs), generated from the legacy
//! hand-written MCP catalog (`tools_catalog_library.py` schemas), the
//! Python bridge argv builders (`server_runtime.py` /
//! `server_runtime_library.py`), and cross-checked against the clap
//! definitions in `crates/cli/src/args/project.rs`.
//!
//! Entries are assembled sorted by id in `lib.rs`; this source preserves
//! the historical MCP catalog order for easier audit against
//! `tools_catalog_datum.py`.

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

macro_rules! p {
    ($name:literal, $ty:ident, $required:expr, $default:expr) => {
        ParamSpec {
            name: $name,
            ty: ParamType::$ty,
            required: $required,
            doc: "MCP parameter.",
            default_json: $default,
        }
    };
}

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.library.list_objects",
        summary: "List resolver-discovered native pool-library objects.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_pool_library_objects",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit("pool-library-objects"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
                ArgvToken::Flag {
                    flag: "--object",
                    param: "object",
                },
                ArgvToken::Switch {
                    flag: "--include-payload",
                    param: "include_payload",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, None),
            p!("kind", Str, false, None),
            p!("object", Uuid, false, None),
            p!("include_payload", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]},"object":{"type":["string","null"]},"include_payload":{"type":["boolean","null"]}},"required":["path"]}"###,
        ),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool"],
        terminal_argv_override: Some(&[
            ArgvToken::Lit("query"),
            ArgvToken::Lit("pool-library-objects"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool",
            },
        ]),
    },
    VerbSpec {
        id: "datum.library.show_object",
        summary: "Show one resolver-discovered native pool-library object with its materialized payload.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "show_pool_library_object",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit("pool-library-objects"),
                ArgvToken::Flag {
                    flag: "--object",
                    param: "object",
                },
                ArgvToken::Lit("--include-payload"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("object", Uuid, true, None),
            p!("pool", Str, false, None),
            p!("kind", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"object":{"type":"string"},"pool":{"type":["string","null"]},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]}},"required":["path","object"]}"###,
        ),
        write_surface: None,
        terminal: true,
        terminal_optional_params: &["pool", "kind"],
        terminal_argv_override: Some(&[
            ArgvToken::Lit("query"),
            ArgvToken::Lit("pool-library-objects"),
            ArgvToken::Param("path"),
            ArgvToken::Flag {
                flag: "--pool",
                param: "pool",
            },
            ArgvToken::Flag {
                flag: "--kind",
                param: "kind",
            },
            ArgvToken::Flag {
                flag: "--object",
                param: "object",
            },
            ArgvToken::Lit("--include-payload"),
        ]),
    },
    VerbSpec {
        id: "datum.library.pool_models",
        summary: "List and verify native pool behavioural-model blobs and attachment references.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "get_pool_model_blobs",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit("pool-models"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--role",
                    param: "role",
                },
                ArgvToken::Flag {
                    flag: "--sha256",
                    param: "sha256",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, None),
            p!("role", Str, false, None),
            p!("sha256", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"role":{"type":["string","null"]},"sha256":{"type":["string","null"]}},"required":["path"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.gc_pool_models",
        summary: "Dry-run or apply conservative garbage collection for orphaned native pool behavioural-model blobs.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "gc_pool_model_blobs",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("gc-pool-models"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--role",
                    param: "role",
                },
                ArgvToken::Flag {
                    flag: "--sha256",
                    param: "sha256",
                },
                ArgvToken::Switch {
                    flag: "--apply",
                    param: "apply",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, None),
            p!("role", Str, false, None),
            p!("sha256", Str, false, None),
            p!("apply", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":["string","null"]},"role":{"type":["string","null"]},"sha256":{"type":["string","null"]},"apply":{"type":["boolean","null"]}},"required":["path"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_object",
        summary: "Create one native pool-library object through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_library_object",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-library-object"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
                ArgvToken::Flag {
                    flag: "--object",
                    param: "object",
                },
                ArgvToken::Flag {
                    flag: "--from-json",
                    param: "from_json",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("kind", Str, true, None),
            p!("object", Uuid, true, None),
            p!("from_json", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]},"object":{"type":"string"},"from_json":{"type":"string"}},"required":["path","kind","object","from_json"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_unit",
        summary: "Create one typed native pool unit through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_unit",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-unit"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--unit",
                    param: "unit",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--manufacturer",
                    param: "manufacturer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("unit", Uuid, true, None),
            p!("name", Str, true, None),
            p!("manufacturer", Str, false, Some("\"\"")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"unit":{"type":"string"},"name":{"type":"string"},"manufacturer":{"type":"string"}},"required":["path","unit","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_unit_pin",
        summary: "Set one typed native pool unit pin entry.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_unit_pin",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-unit-pin"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--unit",
                    param: "unit",
                },
                ArgvToken::Flag {
                    flag: "--pin",
                    param: "pin",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--direction",
                    param: "direction",
                },
                ArgvToken::Flag {
                    flag: "--swap-group",
                    param: "swap_group",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("unit", Uuid, true, None),
            p!("pin", Uuid, true, None),
            p!("name", Str, true, None),
            p!("direction", Str, false, Some("\"Passive\"")),
            p!("swap_group", Int, false, Some("0")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"unit":{"type":"string"},"pin":{"type":"string"},"name":{"type":"string"},"direction":{"type":"string"},"swap_group":{"type":"integer"}},"required":["path","unit","pin","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_symbol",
        summary: "Create one typed native pool symbol for an existing pool unit through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-symbol"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--unit",
                    param: "unit",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("unit", Uuid, true, None),
            p!("name", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"unit":{"type":"string"},"name":{"type":"string"}},"required":["path","symbol","unit","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_line",
        summary: "Append one typed native pool symbol line primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_line",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-line"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--from-x-nm",
                    param: "from_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--from-y-nm",
                    param: "from_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-x-nm",
                    param: "to_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-y-nm",
                    param: "to_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","symbol","from_x_nm","from_y_nm","to_x_nm","to_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_rect",
        summary: "Append one typed native pool symbol rectangle primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--min-x-nm",
                    param: "min_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--min-y-nm",
                    param: "min_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-x-nm",
                    param: "max_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-y-nm",
                    param: "max_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","symbol","min_x_nm","min_y_nm","max_x_nm","max_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_circle",
        summary: "Append one typed native pool symbol circle primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_circle",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-circle"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--center-x-nm",
                    param: "center_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--center-y-nm",
                    param: "center_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--radius-nm",
                    param: "radius_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("center_x_nm", Int, true, None),
            p!("center_y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","symbol","center_x_nm","center_y_nm","radius_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_arc",
        summary: "Append one typed native pool symbol arc primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_arc",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-arc"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
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
                    flag: "--radius-nm",
                    param: "radius_nm",
                },
                ArgvToken::Flag {
                    flag: "--start-angle",
                    param: "start_angle",
                },
                ArgvToken::Flag {
                    flag: "--end-angle",
                    param: "end_angle",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("start_angle", Int, true, None),
            p!("end_angle", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"start_angle":{"type":"integer"},"end_angle":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","symbol","x_nm","y_nm","radius_nm","start_angle","end_angle","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_polygon",
        summary: "Append one typed native pool symbol polygon or polyline primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_polygon",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-polygon"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--vertices",
                    param: "vertices",
                },
                ArgvToken::Switch {
                    flag: "--closed",
                    param: "closed",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("vertices", Str, true, None),
            p!("closed", Bool, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"vertices":{"type":"string"},"closed":{"type":"boolean"},"width_nm":{"type":"integer"}},"required":["path","symbol","vertices","closed","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_symbol_text",
        summary: "Append one typed native pool symbol text primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_symbol_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-symbol-text"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
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
                    flag: "--rotation",
                    param: "rotation",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("text", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("rotation", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"text":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"rotation":{"type":"integer"}},"required":["path","symbol","text","x_nm","y_nm","rotation"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_symbol_pin_anchor",
        summary: "Set one typed native pool symbol pin anchor for a unit pin.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_symbol_pin_anchor",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-symbol-pin-anchor"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--pin",
                    param: "pin",
                },
                ArgvToken::Flag {
                    flag: "--x-nm",
                    param: "x_nm",
                },
                ArgvToken::Flag {
                    flag: "--y-nm",
                    param: "y_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("symbol", Uuid, true, None),
            p!("pin", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"symbol":{"type":"string"},"pin":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","symbol","pin","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_entity",
        summary: "Create one typed native pool entity with an initial gate over an existing unit and symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_entity",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-entity"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--entity",
                    param: "entity",
                },
                ArgvToken::Flag {
                    flag: "--gate",
                    param: "gate",
                },
                ArgvToken::Flag {
                    flag: "--unit",
                    param: "unit",
                },
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
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
                    flag: "--manufacturer",
                    param: "manufacturer",
                },
                ArgvToken::Flag {
                    flag: "--gate-name",
                    param: "gate_name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("entity", Uuid, true, None),
            p!("gate", Uuid, true, None),
            p!("unit", Uuid, true, None),
            p!("symbol", Uuid, true, None),
            p!("name", Str, true, None),
            p!("prefix", Str, true, None),
            p!("manufacturer", Str, false, Some("\"\"")),
            p!("gate_name", Str, false, Some("\"A\"")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"entity":{"type":"string"},"gate":{"type":"string"},"unit":{"type":"string"},"symbol":{"type":"string"},"name":{"type":"string"},"prefix":{"type":"string"},"manufacturer":{"type":"string"},"gate_name":{"type":"string"}},"required":["path","entity","gate","unit","symbol","name","prefix"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_padstack",
        summary: "Create one typed native pool padstack with optional circle/rect aperture and drill diameter.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_padstack",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-padstack"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--padstack",
                    param: "padstack",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--aperture",
                    param: "aperture",
                },
                ArgvToken::Flag {
                    flag: "--diameter-nm",
                    param: "diameter_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
                ArgvToken::Flag {
                    flag: "--height-nm",
                    param: "height_nm",
                },
                ArgvToken::Flag {
                    flag: "--drill-nm",
                    param: "drill_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("padstack", Uuid, true, None),
            p!("name", Str, true, None),
            p!("aperture", Str, false, None),
            p!("diameter_nm", Int, false, None),
            p!("width_nm", Int, false, None),
            p!("height_nm", Int, false, None),
            p!("drill_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"padstack":{"type":"string"},"name":{"type":"string"},"aperture":{"type":"string"},"diameter_nm":{"type":"integer"},"width_nm":{"type":"integer"},"height_nm":{"type":"integer"},"drill_nm":{"type":"integer"}},"required":["path","padstack","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_package",
        summary: "Create one typed native pool package body record; optional pad/padstack fields are legacy land-pattern compatibility input and should be replaced by first-class Footprint authoring.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_package",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-package"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
                ArgvToken::Flag {
                    flag: "--padstack",
                    param: "padstack",
                },
                ArgvToken::Flag {
                    flag: "--pad-name",
                    param: "pad_name",
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
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("name", Str, true, None),
            p!("pad", Uuid, false, Some("null")),
            p!("padstack", Uuid, false, Some("null")),
            p!("pad_name", Str, false, Some("\"1\"")),
            p!("x_nm", Int, false, Some("0")),
            p!("y_nm", Int, false, Some("0")),
            p!("layer", Int, false, Some("1")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"name":{"type":"string"},"pad":{"type":["string","null"]},"padstack":{"type":["string","null"]},"pad_name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"}},"required":["path","package","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_footprint",
        summary: "Create one first-class typed native pool footprint for an existing package.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_footprint",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-footprint"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("package", Uuid, true, None),
            p!("name", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"package":{"type":"string"},"name":{"type":"string"}},"required":["path","footprint","package","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.generate_ipc7351b_soic",
        summary: "Generate an IPC-7351B SOIC footprint and padstack through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "generate_ipc7351b_soic",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("generate-ipc7351b-soic"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--padstack",
                    param: "padstack",
                },
                ArgvToken::Repeated {
                    flag: "--pad",
                    param: "pads",
                },
                ArgvToken::Flag {
                    flag: "--package-code",
                    param: "package_code",
                },
                ArgvToken::Flag {
                    flag: "--pin-count",
                    param: "pin_count",
                },
                ArgvToken::Flag {
                    flag: "--pitch-nm",
                    param: "pitch_nm",
                },
                ArgvToken::Flag {
                    flag: "--body-length-nm",
                    param: "body_length_nm",
                },
                ArgvToken::Flag {
                    flag: "--body-width-nm",
                    param: "body_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--lead-span-nm",
                    param: "lead_span_nm",
                },
                ArgvToken::Flag {
                    flag: "--terminal-length-nm",
                    param: "terminal_length_nm",
                },
                ArgvToken::Flag {
                    flag: "--terminal-width-nm",
                    param: "terminal_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--density",
                    param: "density",
                },
                ArgvToken::Flag {
                    flag: "--mask-expansion-nm",
                    param: "mask_expansion_nm",
                },
                ArgvToken::Flag {
                    flag: "--paste-reduction-nm",
                    param: "paste_reduction_nm",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("package", Uuid, true, None),
            p!("padstack", Uuid, true, None),
            p!("pads", StrList, true, None),
            p!("package_code", Str, true, None),
            p!("pin_count", Int, true, None),
            p!("pitch_nm", Int, true, None),
            p!("body_length_nm", Int, true, None),
            p!("body_width_nm", Int, true, None),
            p!("lead_span_nm", Int, true, None),
            p!("terminal_length_nm", Int, true, None),
            p!("terminal_width_nm", Int, true, None),
            p!("density", Str, false, Some("\"nominal\"")),
            p!("mask_expansion_nm", Int, false, Some("50000")),
            p!("paste_reduction_nm", Int, false, Some("50000")),
            p!("name", Str, false, Some("null")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"package":{"type":"string"},"padstack":{"type":"string"},"pads":{"type":"array","items":{"type":"string"},"minItems":4},"package_code":{"type":"string"},"pin_count":{"type":"integer"},"pitch_nm":{"type":"integer"},"body_length_nm":{"type":"integer"},"body_width_nm":{"type":"integer"},"lead_span_nm":{"type":"integer"},"terminal_length_nm":{"type":"integer"},"terminal_width_nm":{"type":"integer"},"density":{"type":"string","enum":["most","nominal","least"]},"mask_expansion_nm":{"type":"integer"},"paste_reduction_nm":{"type":"integer"},"name":{"type":["string","null"]}},"required":["path","footprint","package","padstack","pads","package_code","pin_count","pitch_nm","body_length_nm","body_width_nm","lead_span_nm","terminal_length_nm","terminal_width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_footprint_pad",
        summary: "Set one first-class typed native pool footprint pad entry referencing an existing padstack.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_footprint_pad",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-footprint-pad"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
                ArgvToken::Flag {
                    flag: "--padstack",
                    param: "padstack",
                },
                ArgvToken::Flag {
                    flag: "--pad-name",
                    param: "pad_name",
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
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("pad", Uuid, true, None),
            p!("padstack", Uuid, true, None),
            p!("pad_name", Str, false, Some("\"1\"")),
            p!("x_nm", Int, false, Some("0")),
            p!("y_nm", Int, false, Some("0")),
            p!("layer", Int, false, Some("1")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"pad":{"type":"string"},"padstack":{"type":"string"},"pad_name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"}},"required":["path","footprint","pad","padstack"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_footprint_courtyard_rect",
        summary: "Set first-class typed native pool footprint rectangular courtyard geometry.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_footprint_courtyard_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-footprint-courtyard-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--min-x-nm",
                    param: "min_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--min-y-nm",
                    param: "min_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-x-nm",
                    param: "max_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-y-nm",
                    param: "max_y_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"}},"required":["path","footprint","min_x_nm","min_y_nm","max_x_nm","max_y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_footprint_courtyard_polygon",
        summary: "Set first-class typed native pool footprint polygon courtyard geometry.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_footprint_courtyard_polygon",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-footprint-courtyard-polygon"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--vertices",
                    param: "vertices",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("vertices", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"vertices":{"type":"string"}},"required":["path","footprint","vertices"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_footprint_silkscreen_line",
        summary: "Append one first-class typed native pool footprint silkscreen line primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_footprint_silkscreen_line",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-footprint-silkscreen-line"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--from-x-nm",
                    param: "from_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--from-y-nm",
                    param: "from_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-x-nm",
                    param: "to_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-y-nm",
                    param: "to_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","footprint","from_x_nm","from_y_nm","to_x_nm","to_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_footprint_silkscreen_rect",
        summary: "Append one first-class typed native pool footprint silkscreen rectangle primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_footprint_silkscreen_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-footprint-silkscreen-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--min-x-nm",
                    param: "min_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--min-y-nm",
                    param: "min_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-x-nm",
                    param: "max_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-y-nm",
                    param: "max_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","footprint","min_x_nm","min_y_nm","max_x_nm","max_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_footprint_silkscreen_circle",
        summary: "Append one first-class typed native pool footprint silkscreen circle primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_footprint_silkscreen_circle",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-footprint-silkscreen-circle"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--center-x-nm",
                    param: "center_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--center-y-nm",
                    param: "center_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--radius-nm",
                    param: "radius_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("center_x_nm", Int, true, None),
            p!("center_y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","footprint","center_x_nm","center_y_nm","radius_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_footprint_silkscreen_polygon",
        summary: "Append one first-class typed native pool footprint silkscreen polygon or polyline primitive.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_footprint_silkscreen_polygon",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-footprint-silkscreen-polygon"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Flag {
                    flag: "--vertices",
                    param: "vertices",
                },
                ArgvToken::Switch {
                    flag: "--closed",
                    param: "closed",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("footprint", Uuid, true, None),
            p!("vertices", Str, true, None),
            p!("closed", Bool, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"footprint":{"type":"string"},"vertices":{"type":"string","description":"Semicolon-separated x,y vertex pairs, e.g. x,y;x,y;x,y."},"closed":{"type":"boolean"},"width_nm":{"type":"integer"}},"required":["path","footprint","vertices","closed","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_package_pad",
        summary: "Legacy package-named compatibility path that writes one pad to the package-linked Footprint; prefer set_pool_footprint_pad.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_package_pad",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-package-pad"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
                ArgvToken::Flag {
                    flag: "--padstack",
                    param: "padstack",
                },
                ArgvToken::Flag {
                    flag: "--pad-name",
                    param: "pad_name",
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
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("pad", Uuid, true, None),
            p!("padstack", Uuid, true, None),
            p!("pad_name", Str, false, Some("\"1\"")),
            p!("x_nm", Int, false, Some("0")),
            p!("y_nm", Int, false, Some("0")),
            p!("layer", Int, false, Some("1")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"pad":{"type":"string"},"padstack":{"type":"string"},"pad_name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"}},"required":["path","package","pad","padstack"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_package_courtyard_rect",
        summary: "Legacy package-named compatibility path that writes rectangular courtyard geometry to the package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_package_courtyard_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-package-courtyard-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--min-x-nm",
                    param: "min_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--min-y-nm",
                    param: "min_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-x-nm",
                    param: "max_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-y-nm",
                    param: "max_y_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"}},"required":["path","package","min_x_nm","min_y_nm","max_x_nm","max_y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_package_courtyard_polygon",
        summary: "Legacy package-named compatibility path that writes polygon courtyard geometry to the package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_package_courtyard_polygon",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-package-courtyard-polygon"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--vertices",
                    param: "vertices",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("vertices", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"vertices":{"type":"string"}},"required":["path","package","vertices"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_line",
        summary: "Legacy package-named compatibility path that appends one silkscreen line primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_line",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-line"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--from-x-nm",
                    param: "from_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--from-y-nm",
                    param: "from_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-x-nm",
                    param: "to_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--to-y-nm",
                    param: "to_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","package","from_x_nm","from_y_nm","to_x_nm","to_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_rect",
        summary: "Legacy package-named compatibility path that appends one silkscreen rectangle primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--min-x-nm",
                    param: "min_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--min-y-nm",
                    param: "min_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-x-nm",
                    param: "max_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--max-y-nm",
                    param: "max_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","package","min_x_nm","min_y_nm","max_x_nm","max_y_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_polygon",
        summary: "Legacy package-named compatibility path that appends one silkscreen polygon or polyline primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_polygon",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-polygon"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--vertices",
                    param: "vertices",
                },
                ArgvToken::Switch {
                    flag: "--closed",
                    param: "closed",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("vertices", Str, true, None),
            p!("closed", Bool, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"vertices":{"type":"string","description":"Semicolon-separated x,y vertex pairs, e.g. x,y;x,y;x,y."},"closed":{"type":"boolean"},"width_nm":{"type":"integer"}},"required":["path","package","vertices","closed","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_circle",
        summary: "Legacy package-named compatibility path that appends one silkscreen circle primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_circle",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-circle"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--center-x-nm",
                    param: "center_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--center-y-nm",
                    param: "center_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--radius-nm",
                    param: "radius_nm",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("center_x_nm", Int, true, None),
            p!("center_y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","package","center_x_nm","center_y_nm","radius_nm","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_arc",
        summary: "Legacy package-named compatibility path that appends one silkscreen arc primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_arc",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-arc"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
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
                    flag: "--radius-nm",
                    param: "radius_nm",
                },
                ArgvToken::Flag {
                    flag: "--start-angle",
                    param: "start_angle",
                },
                ArgvToken::Flag {
                    flag: "--end-angle",
                    param: "end_angle",
                },
                ArgvToken::Flag {
                    flag: "--width-nm",
                    param: "width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("start_angle", Int, true, None),
            p!("end_angle", Int, true, None),
            p!("width_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"start_angle":{"type":"integer"},"end_angle":{"type":"integer"},"width_nm":{"type":"integer"}},"required":["path","package","x_nm","y_nm","radius_nm","start_angle","end_angle","width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_silkscreen_text",
        summary: "Legacy package-named compatibility path that appends one silkscreen text primitive to the unique package-linked Footprint.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_silkscreen_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-silkscreen-text"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
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
                    flag: "--rotation",
                    param: "rotation",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("text", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("rotation", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"text":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"rotation":{"type":"number"}},"required":["path","package","text","x_nm","y_nm","rotation"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.add_package_model_3d",
        summary: "Attach one 3D model to a typed native pool package.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_pool_package_model_3d",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-pool-package-model-3d"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--model-path",
                    param: "model_path",
                },
                ArgvToken::Flag {
                    flag: "--transform-json",
                    param: "transform_json",
                },
                ArgvToken::Flag {
                    flag: "--format",
                    param: "format",
                },
                ArgvToken::Flag {
                    flag: "--tx-nm",
                    param: "tx_nm",
                },
                ArgvToken::Flag {
                    flag: "--ty-nm",
                    param: "ty_nm",
                },
                ArgvToken::Flag {
                    flag: "--tz-nm",
                    param: "tz_nm",
                },
                ArgvToken::Flag {
                    flag: "--roll-tenths-deg",
                    param: "roll_tenths_deg",
                },
                ArgvToken::Flag {
                    flag: "--pitch-tenths-deg",
                    param: "pitch_tenths_deg",
                },
                ArgvToken::Flag {
                    flag: "--yaw-tenths-deg",
                    param: "yaw_tenths_deg",
                },
                ArgvToken::Flag {
                    flag: "--scale",
                    param: "scale",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("model_path", Str, true, None),
            p!("transform_json", Str, false, Some("null")),
            p!("format", Str, false, None),
            p!("tx_nm", Int, false, None),
            p!("ty_nm", Int, false, None),
            p!("tz_nm", Int, false, None),
            p!("roll_tenths_deg", Int, false, None),
            p!("pitch_tenths_deg", Int, false, None),
            p!("yaw_tenths_deg", Int, false, None),
            p!("scale", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"model_path":{"type":"string"},"transform_json":{"type":["string","null"]},"format":{"type":["string","null"]},"tx_nm":{"type":["integer","null"]},"ty_nm":{"type":["integer","null"]},"tz_nm":{"type":["integer","null"]},"roll_tenths_deg":{"type":["integer","null"]},"pitch_tenths_deg":{"type":["integer","null"]},"yaw_tenths_deg":{"type":["integer","null"]},"scale":{"type":["string","number","null"]}},"required":["path","package","model_path"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_package_body_heights",
        summary: "Set or clear typed native pool package body-height metadata.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_package_body_heights",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-package-body-heights"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--body-height-nm",
                    param: "body_height_nm",
                },
                ArgvToken::Flag {
                    flag: "--body-height-mounted-nm",
                    param: "body_height_mounted_nm",
                },
                ArgvToken::Switch {
                    flag: "--clear",
                    param: "clear",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("package", Uuid, true, None),
            p!("body_height_nm", Int, false, Some("null")),
            p!("body_height_mounted_nm", Int, false, Some("null")),
            p!("clear", Bool, false, Some("null")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"package":{"type":"string"},"body_height_nm":{"type":["integer","null"]},"body_height_mounted_nm":{"type":["integer","null"]},"clear":{"type":["boolean","null"]}},"required":["path","package"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_part",
        summary: "Create one typed native pool part binding an existing entity to an existing package.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_part",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-pool-part"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--entity",
                    param: "entity",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
                ArgvToken::Flag {
                    flag: "--mpn",
                    param: "mpn",
                },
                ArgvToken::Flag {
                    flag: "--manufacturer",
                    param: "manufacturer",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
                ArgvToken::Flag {
                    flag: "--description",
                    param: "description",
                },
                ArgvToken::Flag {
                    flag: "--datasheet",
                    param: "datasheet",
                },
                ArgvToken::Flag {
                    flag: "--lifecycle",
                    param: "lifecycle",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("entity", Uuid, true, None),
            p!("package", Uuid, true, None),
            p!("mpn", Str, true, None),
            p!("manufacturer", Str, false, Some("\"\"")),
            p!("value", Str, false, Some("\"\"")),
            p!("description", Str, false, Some("\"\"")),
            p!("datasheet", Str, false, Some("\"\"")),
            p!("lifecycle", Str, false, Some("\"Active\"")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"entity":{"type":"string"},"package":{"type":"string"},"mpn":{"type":"string"},"manufacturer":{"type":"string"},"value":{"type":"string"},"description":{"type":"string"},"datasheet":{"type":"string"},"lifecycle":{"type":"string"}},"required":["path","part","entity","package","mpn"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_metadata",
        summary: "Set typed native pool part metadata fields.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_metadata",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-metadata"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mpn",
                    param: "mpn",
                },
                ArgvToken::Flag {
                    flag: "--manufacturer",
                    param: "manufacturer",
                },
                ArgvToken::Flag {
                    flag: "--manufacturer-jep106",
                    param: "manufacturer_jep106",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
                ArgvToken::Flag {
                    flag: "--description",
                    param: "description",
                },
                ArgvToken::Flag {
                    flag: "--datasheet",
                    param: "datasheet",
                },
                ArgvToken::Flag {
                    flag: "--lifecycle",
                    param: "lifecycle",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mpn", Str, false, None),
            p!("manufacturer", Str, false, None),
            p!("manufacturer_jep106", Int, false, None),
            p!("value", Str, false, None),
            p!("description", Str, false, None),
            p!("datasheet", Str, false, None),
            p!("lifecycle", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mpn":{"type":"string"},"manufacturer":{"type":"string"},"manufacturer_jep106":{"type":"integer"},"value":{"type":"string"},"description":{"type":"string"},"datasheet":{"type":"string"},"lifecycle":{"type":"string"}},"required":["path","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_parametric",
        summary: "Set typed native pool part parametric key-value fields.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_parametric",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-parametric"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--param",
                    param: "params",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("params", Json, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string","enum":["merge","replace"]},"params":{"type":"object","additionalProperties":{"type":"string"}}},"required":["path","part","params"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_orderable_mpns",
        summary: "Set typed native pool part orderable manufacturer part numbers.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_orderable_mpns",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-orderable-mpns"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--mpn",
                    param: "mpns",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("mpns", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string","enum":["merge","replace"]},"mpns":{"type":"array","items":{"type":"string"}}},"required":["path","part","mpns"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_tags",
        summary: "Set typed native pool part tags.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_tags",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-tags"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--tag",
                    param: "tags",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("tags", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string","enum":["merge","replace"]},"tags":{"type":"array","items":{"type":"string"}}},"required":["path","part","tags"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_packaging_options",
        summary: "Set typed native pool part encoded packaging option payloads.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_packaging_options",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-packaging-options"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--option",
                    param: "options",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("options", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string","enum":["merge","replace"]},"options":{"type":"array","items":{"type":"string"}}},"required":["path","part","options"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_supply_chain",
        summary: "Set or clear typed native pool part supply-chain offer metadata.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_supply_chain",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-supply-chain"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Switch {
                    flag: "--clear",
                    param: "clear",
                },
                ArgvToken::Flag {
                    flag: "--checked-at",
                    param: "checked_at",
                },
                ArgvToken::Repeated {
                    flag: "--offer",
                    param: "offers",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("clear", Bool, false, Some("false")),
            p!("checked_at", Str, false, Some("null")),
            p!("offers", StrList, false, Some("[]")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"clear":{"type":"boolean"},"checked_at":{"type":["string","null"]},"offers":{"type":"array","items":{"type":"string"}}},"required":["path","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_behavioural_models",
        summary: "Set typed native pool part behavioural model attachment payloads.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_behavioural_models",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-behavioural-models"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--model",
                    param: "models",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("models", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string","enum":["merge","replace"]},"models":{"type":"array","items":{"type":"string"}}},"required":["path","part","models"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.attach_part_model",
        summary: "Attach one model source file to a typed native pool part without parsing the model payload.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "attach_pool_part_model",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("attach-pool-part-model"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--source",
                    param: "source",
                },
                ArgvToken::Flag {
                    flag: "--role",
                    param: "role",
                },
                ArgvToken::Flag {
                    flag: "--dialect",
                    param: "dialect",
                },
                ArgvToken::Repeated {
                    flag: "--model-name",
                    param: "model_names",
                },
                ArgvToken::Switch {
                    flag: "--encrypted",
                    param: "encrypted",
                },
                ArgvToken::Flag {
                    flag: "--encryption-scheme",
                    param: "encryption_scheme",
                },
                ArgvToken::Flag {
                    flag: "--vendor",
                    param: "vendor",
                },
                ArgvToken::Flag {
                    flag: "--fetched-at",
                    param: "fetched_at",
                },
                ArgvToken::Flag {
                    flag: "--format-metadata-json",
                    param: "format_metadata_json",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("source", Str, true, None),
            p!("role", Str, true, None),
            p!("dialect", Str, false, Some("null")),
            p!("model_names", StrList, false, Some("[]")),
            p!("encrypted", Bool, false, Some("false")),
            p!("encryption_scheme", Str, false, Some("null")),
            p!("vendor", Str, false, Some("null")),
            p!("fetched_at", Str, false, Some("null")),
            p!("format_metadata_json", Str, false, Some("null")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"source":{"type":"string"},"role":{"type":"string"},"dialect":{"type":["string","null"]},"model_names":{"type":"array","items":{"type":"string"}},"encrypted":{"type":"boolean"},"encryption_scheme":{"type":["string","null"]},"vendor":{"type":["string","null"]},"fetched_at":{"type":["string","null"]},"format_metadata_json":{"type":["string","null"]}},"required":["path","part","source","role"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.detach_part_model",
        summary: "Detach one model attachment from a typed native pool part.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "detach_pool_part_model",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("detach-pool-part-model"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--attachment",
                    param: "attachment",
                },
                ArgvToken::Flag {
                    flag: "--model",
                    param: "model",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("attachment", Uuid, false, Some("null")),
            p!("model", Uuid, false, Some("null")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"attachment":{"type":["string","null"]},"model":{"type":["string","null"]}},"required":["path","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_thermal",
        summary: "Set or clear typed native pool part thermal characteristics.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_thermal",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-thermal"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--theta-ja-c-per-w",
                    param: "theta_ja_c_per_w",
                },
                ArgvToken::Flag {
                    flag: "--theta-jc-top-c-per-w",
                    param: "theta_jc_top_c_per_w",
                },
                ArgvToken::Flag {
                    flag: "--theta-jc-bot-c-per-w",
                    param: "theta_jc_bot_c_per_w",
                },
                ArgvToken::Flag {
                    flag: "--theta-jb-c-per-w",
                    param: "theta_jb_c_per_w",
                },
                ArgvToken::Flag {
                    flag: "--max-junction-c",
                    param: "max_junction_c",
                },
                ArgvToken::Flag {
                    flag: "--thermal-reference",
                    param: "thermal_reference",
                },
                ArgvToken::Switch {
                    flag: "--clear",
                    param: "clear",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("theta_ja_c_per_w", Str, false, Some("null")),
            p!("theta_jc_top_c_per_w", Str, false, Some("null")),
            p!("theta_jc_bot_c_per_w", Str, false, Some("null")),
            p!("theta_jb_c_per_w", Str, false, Some("null")),
            p!("max_junction_c", Str, false, Some("null")),
            p!("thermal_reference", Str, false, Some("null")),
            p!("clear", Bool, false, Some("false")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"theta_ja_c_per_w":{"type":["number","string","null"]},"theta_jc_top_c_per_w":{"type":["number","string","null"]},"theta_jc_bot_c_per_w":{"type":["number","string","null"]},"theta_jb_c_per_w":{"type":["number","string","null"]},"max_junction_c":{"type":["number","string","null"]},"thermal_reference":{"type":["string","null"]},"clear":{"type":"boolean"}},"required":["path","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_pad_map_entry",
        summary: "Set one typed native pool part pad-map entry from package pad to entity gate pin.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_pad_map_entry",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-pad-map-entry"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
                ArgvToken::Flag {
                    flag: "--gate",
                    param: "gate",
                },
                ArgvToken::Flag {
                    flag: "--pin",
                    param: "pin",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("pad", Uuid, true, None),
            p!("gate", Uuid, true, None),
            p!("pin", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["path","part","pad","gate","pin"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_part_pad_map",
        summary: "Set typed native pool part pad-map entries in merge or replace mode.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_part_pad_map",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-part-pad-map"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
                ArgvToken::Repeated {
                    flag: "--entry",
                    param: "entries",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("part", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("entries", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"part":{"type":"string"},"mode":{"type":"string"},"entries":{"type":"array","items":{"type":"object","properties":{"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["pad","gate","pin"]}}},"required":["path","part","entries"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.create_pin_pad_map",
        summary: "Create one first-class native pool PinPadMap through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_pool_pin_pad_map",
            argv: &[
                ArgvToken::Lit("project"),
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
                ArgvToken::Repeated {
                    flag: "--entry",
                    param: "entries",
                },
                ArgvToken::Flag {
                    flag: "--footprint",
                    param: "footprint",
                },
                ArgvToken::Switch {
                    flag: "--set-default",
                    param: "set_default",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("map", Uuid, true, None),
            p!("part", Uuid, true, None),
            p!("entries", StrList, true, None),
            p!("footprint", Uuid, false, Some("null")),
            p!("set_default", Bool, false, Some("false")),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"map":{"type":"string"},"part":{"type":"string"},"footprint":{"type":["string","null"]},"entries":{"type":"array","items":{"type":"object","properties":{"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["pad","pin"]}},"set_default":{"type":"boolean"}},"required":["path","map","part","entries"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_pin_pad_map",
        summary: "Update first-class native pool PinPadMap mappings through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_pin_pad_map",
            argv: &[
                ArgvToken::Lit("project"),
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("map", Uuid, true, None),
            p!("mode", Str, false, Some("\"merge\"")),
            p!("entries", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"map":{"type":"string"},"mode":{"type":"string"},"entries":{"type":"array","items":{"type":"object","properties":{"pad":{"type":"string"},"gate":{"type":"string"},"pin":{"type":"string"}},"required":["pad","pin"]}}},"required":["path","map","entries"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.set_object",
        summary: "Replace one native pool-library object through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pool_library_object",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pool-library-object"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
                ArgvToken::Flag {
                    flag: "--object",
                    param: "object",
                },
                ArgvToken::Flag {
                    flag: "--from-json",
                    param: "from_json",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("kind", Str, true, None),
            p!("object", Uuid, true, None),
            p!("from_json", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]},"object":{"type":"string"},"from_json":{"type":"string"}},"required":["path","kind","object","from_json"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.library.delete_object",
        summary: "Delete one native pool-library object through the journaled project commit path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_pool_library_object",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-pool-library-object"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pool",
                    param: "pool",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
                ArgvToken::Flag {
                    flag: "--object",
                    param: "object",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pool", Str, false, Some("\"pool\"")),
            p!("kind", Str, true, None),
            p!("object", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pool":{"type":"string"},"kind":{"type":"string","enum":["units","symbols","entities","parts","packages","footprints","padstacks","pin_pad_maps"]},"object":{"type":"string"}},"required":["path","kind","object"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
