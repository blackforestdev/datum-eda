//! The `datum.schematic` verb family (62 verbs), generated from the legacy
//! hand-written MCP catalog (`tools_catalog_datum.py`) and the Python
//! bridge argv builders (`server_runtime.py` / schematic runtime modules).
//!
//! Entries preserve the historical public MCP catalog order; `lib.rs`
//! assembles all families sorted by id for deterministic projection.

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
        id: "datum.schematic.create_sheet",
        summary: "Create one native-project schematic sheet.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_sheet",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-sheet"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("name", Str, true, None),
            p!("sheet", Uuid, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"sheet":{"type":["string","null"]}},"required":["path","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_sheet",
        summary: "Delete one native-project schematic sheet and its payload as one journaled operation.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_sheet",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-sheet"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("sheet", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"}},"required":["path","sheet"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.rename_sheet",
        summary: "Rename one native-project schematic sheet through the journaled substrate path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "rename_sheet",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("rename-sheet"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("name", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"name":{"type":"string"}},"required":["path","sheet","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.create_sheet_definition",
        summary: "Create one native-project schematic sheet definition through the journaled substrate path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_sheet_definition",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-sheet-definition"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--root-sheet",
                    param: "root_sheet",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--definition",
                    param: "definition",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("root_sheet", Uuid, true, None),
            p!("name", Str, true, None),
            p!("definition", Uuid, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"root_sheet":{"type":"string"},"name":{"type":"string"},"definition":{"type":["string","null"]}},"required":["path","root_sheet","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.create_sheet_instance",
        summary: "Create one native-project schematic sheet instance through the journaled substrate path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_sheet_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-sheet-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--definition",
                    param: "definition",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
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
                    flag: "--parent-sheet",
                    param: "parent_sheet",
                },
                ArgvToken::Flag {
                    flag: "--instance",
                    param: "instance",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("definition", Uuid, true, None),
            p!("name", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("parent_sheet", Uuid, false, None),
            p!("instance", Uuid, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"definition":{"type":"string"},"name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"parent_sheet":{"type":["string","null"]},"instance":{"type":["string","null"]}},"required":["path","definition","name","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_sheet_instance",
        summary: "Delete one native-project schematic sheet instance through the journaled substrate path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_sheet_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-sheet-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--instance",
                    param: "instance",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("instance", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"instance":{"type":"string"}},"required":["path","instance"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.move_sheet_instance",
        summary: "Move one native-project schematic sheet instance through the journaled substrate path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "move_sheet_instance",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("move-sheet-instance"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--instance",
                    param: "instance",
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
            p!("instance", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"instance":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","instance","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.bind_sheet_instance_port",
        summary: "Bind a parent-sheet hierarchical port to a sheet instance.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "bind_sheet_instance_port",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("bind-sheet-instance-port"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--instance",
                    param: "instance",
                },
                ArgvToken::Flag {
                    flag: "--port",
                    param: "port",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("instance", Uuid, true, None),
            p!("port", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"instance":{"type":"string"},"port":{"type":"string"}},"required":["path","instance","port"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.unbind_sheet_instance_port",
        summary: "Remove a parent-sheet hierarchical port binding from a sheet instance.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "unbind_sheet_instance_port",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("unbind-sheet-instance-port"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--instance",
                    param: "instance",
                },
                ArgvToken::Flag {
                    flag: "--port",
                    param: "port",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("instance", Uuid, true, None),
            p!("port", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"instance":{"type":"string"},"port":{"type":"string"}},"required":["path","instance","port"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.draw_wire",
        summary: "Draw one native-project schematic wire.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "draw_wire",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("draw-wire"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"}},"required":["path","sheet","from_x_nm","from_y_nm","to_x_nm","to_y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_wire",
        summary: "Delete one native-project schematic wire.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_wire",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-wire"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--wire",
                    param: "wire",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("wire", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"wire":{"type":"string"}},"required":["path","wire"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_junction",
        summary: "Place one native-project schematic junction.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_junction",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-junction"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            p!("sheet", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","sheet","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_junction",
        summary: "Delete one native-project schematic junction.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_junction",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-junction"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--junction",
                    param: "junction",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("junction", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"junction":{"type":"string"}},"required":["path","junction"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_noconnect",
        summary: "Place one native-project schematic no-connect marker.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_noconnect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-noconnect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            p!("sheet", Uuid, true, None),
            p!("symbol", Uuid, true, None),
            p!("pin", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"symbol":{"type":"string"},"pin":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","sheet","symbol","pin","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_noconnect",
        summary: "Delete one native-project schematic no-connect marker.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_noconnect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-noconnect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--noconnect",
                    param: "noconnect",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("noconnect", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"noconnect":{"type":"string"}},"required":["path","noconnect"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_label",
        summary: "Place one native-project schematic label.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_label",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-label"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
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
                    flag: "--kind",
                    param: "kind",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("name", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("kind", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"kind":{"type":["string","null"]}},"required":["path","sheet","name","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.rename_label",
        summary: "Rename one native-project schematic label.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "rename_label",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("rename-label"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--label",
                    param: "label",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("label", Uuid, true, None),
            p!("name", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"label":{"type":"string"},"name":{"type":"string"}},"required":["path","label","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_label",
        summary: "Delete one native-project schematic label.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_label",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-label"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--label",
                    param: "label",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("label", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"label":{"type":"string"}},"required":["path","label"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_port",
        summary: "Place one native-project schematic port.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_port",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-port"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            p!("sheet", Uuid, true, None),
            p!("name", Str, true, None),
            p!("direction", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"name":{"type":"string"},"direction":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","sheet","name","direction","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_port",
        summary: "Edit one native-project schematic port.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_port",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-port"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--port",
                    param: "port",
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
            p!("port", Uuid, true, None),
            p!("name", Str, false, None),
            p!("direction", Str, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"port":{"type":"string"},"name":{"type":["string","null"]},"direction":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]}},"required":["path","port"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_port",
        summary: "Delete one native-project schematic port.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_port",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-port"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--port",
                    param: "port",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("port", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"port":{"type":"string"}},"required":["path","port"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.create_bus",
        summary: "Create one native-project schematic bus.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "create_bus",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("create-bus"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Repeated {
                    flag: "--member",
                    param: "members",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("name", Str, true, None),
            p!("members", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"name":{"type":"string"},"members":{"type":"array","items":{"type":"string"}}},"required":["path","sheet","name","members"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_bus_members",
        summary: "Replace native-project schematic bus members.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_bus_members",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-bus-members"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--bus",
                    param: "bus",
                },
                ArgvToken::Repeated {
                    flag: "--member",
                    param: "members",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("bus", Uuid, true, None),
            p!("members", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"bus":{"type":"string"},"members":{"type":"array","items":{"type":"string"}}},"required":["path","bus","members"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_bus",
        summary: "Delete one native-project schematic bus.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_bus",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-bus"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--bus",
                    param: "bus",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("bus", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"bus":{"type":"string"}},"required":["path","bus"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_bus_entry",
        summary: "Place one native-project schematic bus entry.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_bus_entry",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-bus-entry"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
                ArgvToken::Flag {
                    flag: "--bus",
                    param: "bus",
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
                    flag: "--wire",
                    param: "wire",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("bus", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("wire", Uuid, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"bus":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"wire":{"type":["string","null"]}},"required":["path","sheet","bus","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_bus_entry",
        summary: "Delete one native-project schematic bus entry.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_bus_entry",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-bus-entry"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--bus-entry",
                    param: "bus_entry",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("bus_entry", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"bus_entry":{"type":"string"}},"required":["path","bus_entry"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_text",
        summary: "Place one native-project schematic text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_schematic_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-text"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("text", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("rotation_deg", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"text":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"rotation_deg":{"type":["integer","null"]}},"required":["path","sheet","text","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_text",
        summary: "Edit one native-project schematic text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_schematic_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-text"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("text", Uuid, true, None),
            p!("value", Str, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
            p!("rotation_deg", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"text":{"type":"string"},"value":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"rotation_deg":{"type":["integer","null"]}},"required":["path","text"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_text",
        summary: "Delete one native-project schematic text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_schematic_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-text"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("text", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"text":{"type":"string"}},"required":["path","text"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_drawing_line",
        summary: "Place one schematic drawing line.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_drawing_line",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-drawing-line"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"}},"required":["path","sheet","from_x_nm","from_y_nm","to_x_nm","to_y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_drawing_rect",
        summary: "Place one schematic drawing rectangle.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_drawing_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-drawing-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            p!("sheet", Uuid, true, None),
            p!("min_x_nm", Int, true, None),
            p!("min_y_nm", Int, true, None),
            p!("max_x_nm", Int, true, None),
            p!("max_y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"min_x_nm":{"type":"integer"},"min_y_nm":{"type":"integer"},"max_x_nm":{"type":"integer"},"max_y_nm":{"type":"integer"}},"required":["path","sheet","min_x_nm","min_y_nm","max_x_nm","max_y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_drawing_circle",
        summary: "Place one schematic drawing circle.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_drawing_circle",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-drawing-circle"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("center_x_nm", Int, true, None),
            p!("center_y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"}},"required":["path","sheet","center_x_nm","center_y_nm","radius_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_drawing_arc",
        summary: "Place one schematic drawing arc.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_drawing_arc",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-drawing-arc"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
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
                    flag: "--start-angle-mdeg",
                    param: "start_angle_mdeg",
                },
                ArgvToken::Flag {
                    flag: "--end-angle-mdeg",
                    param: "end_angle_mdeg",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("center_x_nm", Int, true, None),
            p!("center_y_nm", Int, true, None),
            p!("radius_nm", Int, true, None),
            p!("start_angle_mdeg", Int, true, None),
            p!("end_angle_mdeg", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"center_x_nm":{"type":"integer"},"center_y_nm":{"type":"integer"},"radius_nm":{"type":"integer"},"start_angle_mdeg":{"type":"integer"},"end_angle_mdeg":{"type":"integer"}},"required":["path","sheet","center_x_nm","center_y_nm","radius_nm","start_angle_mdeg","end_angle_mdeg"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_drawing_line",
        summary: "Edit one schematic drawing line.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_drawing_line",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-drawing-line"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--drawing",
                    param: "drawing",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("drawing", Uuid, true, None),
            p!("from_x_nm", Int, false, None),
            p!("from_y_nm", Int, false, None),
            p!("to_x_nm", Int, false, None),
            p!("to_y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"drawing":{"type":"string"},"from_x_nm":{"type":["integer","null"]},"from_y_nm":{"type":["integer","null"]},"to_x_nm":{"type":["integer","null"]},"to_y_nm":{"type":["integer","null"]}},"required":["path","drawing"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_drawing_rect",
        summary: "Edit one schematic drawing rectangle.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_drawing_rect",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-drawing-rect"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--drawing",
                    param: "drawing",
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
            p!("drawing", Uuid, true, None),
            p!("min_x_nm", Int, false, None),
            p!("min_y_nm", Int, false, None),
            p!("max_x_nm", Int, false, None),
            p!("max_y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"drawing":{"type":"string"},"min_x_nm":{"type":["integer","null"]},"min_y_nm":{"type":["integer","null"]},"max_x_nm":{"type":["integer","null"]},"max_y_nm":{"type":["integer","null"]}},"required":["path","drawing"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_drawing_circle",
        summary: "Edit one schematic drawing circle.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_drawing_circle",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-drawing-circle"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--drawing",
                    param: "drawing",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("drawing", Uuid, true, None),
            p!("center_x_nm", Int, false, None),
            p!("center_y_nm", Int, false, None),
            p!("radius_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"drawing":{"type":"string"},"center_x_nm":{"type":["integer","null"]},"center_y_nm":{"type":["integer","null"]},"radius_nm":{"type":["integer","null"]}},"required":["path","drawing"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_drawing_arc",
        summary: "Edit one schematic drawing arc.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_drawing_arc",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-drawing-arc"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--drawing",
                    param: "drawing",
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
                    flag: "--start-angle-mdeg",
                    param: "start_angle_mdeg",
                },
                ArgvToken::Flag {
                    flag: "--end-angle-mdeg",
                    param: "end_angle_mdeg",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("drawing", Uuid, true, None),
            p!("center_x_nm", Int, false, None),
            p!("center_y_nm", Int, false, None),
            p!("radius_nm", Int, false, None),
            p!("start_angle_mdeg", Int, false, None),
            p!("end_angle_mdeg", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"drawing":{"type":"string"},"center_x_nm":{"type":["integer","null"]},"center_y_nm":{"type":["integer","null"]},"radius_nm":{"type":["integer","null"]},"start_angle_mdeg":{"type":["integer","null"]},"end_angle_mdeg":{"type":["integer","null"]}},"required":["path","drawing"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_drawing",
        summary: "Delete one schematic drawing object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_drawing",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-drawing"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--drawing",
                    param: "drawing",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("drawing", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"drawing":{"type":"string"}},"required":["path","drawing"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.place_symbol",
        summary: "Place one native-project schematic symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-symbol"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--sheet",
                    param: "sheet",
                },
                ArgvToken::Flag {
                    flag: "--reference",
                    param: "reference",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
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
                    flag: "--lib-id",
                    param: "lib_id",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
                ArgvToken::Switch {
                    flag: "--mirrored",
                    param: "mirrored",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("sheet", Uuid, true, None),
            p!("reference", Str, true, None),
            p!("value", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("lib_id", Str, false, None),
            p!("rotation_deg", Int, false, None),
            p!("mirrored", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"sheet":{"type":"string"},"reference":{"type":"string"},"value":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"lib_id":{"type":["string","null"]},"rotation_deg":{"type":["integer","null"]},"mirrored":{"type":["boolean","null"]}},"required":["path","sheet","reference","value","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.move_symbol",
        summary: "Move one native-project schematic symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "move_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("move-symbol"),
                ArgvToken::Param("path"),
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","symbol","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.rotate_symbol",
        summary: "Rotate one native-project schematic symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "rotate_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("rotate-symbol"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("rotation_deg", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"rotation_deg":{"type":"integer"}},"required":["path","symbol","rotation_deg"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.mirror_symbol",
        summary: "Mirror one native-project schematic symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "mirror_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("mirror-symbol"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_symbol",
        summary: "Delete one native-project schematic symbol.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_symbol",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-symbol"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_reference",
        summary: "Set one native-project schematic symbol reference.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_reference",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-reference"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--reference",
                    param: "reference",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("reference", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"reference":{"type":"string"}},"required":["path","symbol","reference"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_value",
        summary: "Set one native-project schematic symbol value.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_value",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-value"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("value", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"value":{"type":"string"}},"required":["path","symbol","value"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_display_mode",
        summary: "Set one native-project schematic symbol display mode.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_display_mode",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-display-mode"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--mode",
                    param: "mode",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("mode", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"mode":{"type":"string"}},"required":["path","symbol","mode"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_hidden_power_behavior",
        summary: "Set one native-project schematic symbol hidden-power behavior.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_hidden_power_behavior",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-hidden-power-behavior"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--behavior",
                    param: "behavior",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("behavior", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"behavior":{"type":"string"}},"required":["path","symbol","behavior"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_unit",
        summary: "Set one native-project schematic symbol unit selection.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_unit",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-unit"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--unit",
                    param: "unit",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("unit", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"unit":{"type":"string"}},"required":["path","symbol","unit"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_symbol_unit",
        summary: "Clear one native-project schematic symbol unit selection.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_symbol_unit",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-symbol-unit"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_gate",
        summary: "Set one native-project schematic symbol gate UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_gate",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-gate"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--gate",
                    param: "gate",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("gate", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"gate":{"type":"string"}},"required":["path","symbol","gate"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_symbol_gate",
        summary: "Clear one native-project schematic symbol gate UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_symbol_gate",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-symbol-gate"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_entity",
        summary: "Set one native-project schematic symbol entity UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_entity",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-entity"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--entity",
                    param: "entity",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("entity", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"entity":{"type":"string"}},"required":["path","symbol","entity"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_symbol_entity",
        summary: "Clear one native-project schematic symbol entity UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_symbol_entity",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-symbol-entity"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_part",
        summary: "Set one native-project schematic symbol part UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_part",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-part"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("part", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"part":{"type":"string"}},"required":["path","symbol","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_symbol_part",
        summary: "Clear one native-project schematic symbol part UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_symbol_part",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-symbol-part"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_symbol_lib_id",
        summary: "Set one native-project schematic symbol library identifier.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_symbol_lib_id",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-symbol-lib-id"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--lib-id",
                    param: "lib_id",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("lib_id", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"lib_id":{"type":"string"}},"required":["path","symbol","lib_id"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_symbol_lib_id",
        summary: "Clear one native-project schematic symbol library identifier.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_symbol_lib_id",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-symbol-lib-id"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("symbol", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"}},"required":["path","symbol"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.set_pin_override",
        summary: "Set one native-project schematic symbol pin display override.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_pin_override",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-pin-override"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--pin",
                    param: "pin",
                },
                ArgvToken::Flag {
                    flag: "--visible",
                    param: "visible",
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
            p!("symbol", Uuid, true, None),
            p!("pin", Uuid, true, None),
            p!("visible", Bool, true, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"pin":{"type":"string"},"visible":{"type":"boolean"},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]}},"required":["path","symbol","pin","visible"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.clear_pin_override",
        summary: "Clear one native-project schematic symbol pin display override.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_pin_override",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-pin-override"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--pin",
                    param: "pin",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("symbol", Uuid, true, None),
            p!("pin", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"pin":{"type":"string"}},"required":["path","symbol","pin"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.add_symbol_field",
        summary: "Add one native-project schematic symbol field.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_symbol_field",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-symbol-field"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--symbol",
                    param: "symbol",
                },
                ArgvToken::Flag {
                    flag: "--key",
                    param: "key",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
                ArgvToken::Switch {
                    flag: "--hidden",
                    param: "hidden",
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
            p!("symbol", Uuid, true, None),
            p!("key", Str, true, None),
            p!("value", Str, true, None),
            p!("hidden", Bool, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"symbol":{"type":"string"},"key":{"type":"string"},"value":{"type":"string"},"hidden":{"type":["boolean","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]}},"required":["path","symbol","key","value"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.edit_symbol_field",
        summary: "Edit one native-project schematic symbol field.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_symbol_field",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-symbol-field"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--field",
                    param: "field",
                },
                ArgvToken::Flag {
                    flag: "--key",
                    param: "key",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
                ArgvToken::Flag {
                    flag: "--visible",
                    param: "visible",
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
            p!("field", Uuid, true, None),
            p!("key", Str, false, None),
            p!("value", Str, false, None),
            p!("visible", Bool, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"field":{"type":"string"},"key":{"type":["string","null"]},"value":{"type":["string","null"]},"visible":{"type":["boolean","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]}},"required":["path","field"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.schematic.delete_symbol_field",
        summary: "Delete one native-project schematic symbol field.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_symbol_field",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-symbol-field"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--field",
                    param: "field",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("field", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"field":{"type":"string"}},"required":["path","field"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
