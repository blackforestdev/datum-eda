//! The `datum.pcb` verb family (45 verbs), generated from the legacy
//! hand-written MCP catalog (`tools_catalog_datum.py`) and the Python
//! bridge argv builders (`server_runtime.py`).
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
        id: "datum.pcb.place_component",
        summary: "Place one native-project board component through the journaled board-package creation path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-component"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
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
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("part", Uuid, true, None),
            p!("package", Uuid, true, None),
            p!("reference", Str, true, None),
            p!("value", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("layer", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"part":{"type":"string"},"package":{"type":"string"},"reference":{"type":"string"},"value":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"}},"required":["path","part","package","reference","value","x_nm","y_nm","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.generate_board_components",
        summary: "Generate initial native-project board components from schematic symbols with bound parts/packages; previews by default and journals only when apply is true.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "generate_board_components",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("generate-board-components"),
                ArgvToken::Param("path"),
                ArgvToken::Switch {
                    flag: "--apply",
                    param: "apply",
                },
                ArgvToken::Switch {
                    flag: "--as-proposal",
                    param: "as_proposal",
                },
                ArgvToken::Flag {
                    flag: "--proposal",
                    param: "proposal",
                },
                ArgvToken::Flag {
                    flag: "--rationale",
                    param: "rationale",
                },
                ArgvToken::Flag {
                    flag: "--origin-x-nm",
                    param: "origin_x_nm",
                },
                ArgvToken::Flag {
                    flag: "--origin-y-nm",
                    param: "origin_y_nm",
                },
                ArgvToken::Flag {
                    flag: "--pitch-nm",
                    param: "pitch_nm",
                },
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("apply", Bool, false, None),
            p!("as_proposal", Bool, false, None),
            p!("proposal", Uuid, false, None),
            p!("rationale", Str, false, None),
            p!("origin_x_nm", Int, false, None),
            p!("origin_y_nm", Int, false, None),
            p!("pitch_nm", Int, false, None),
            p!("layer", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"apply":{"type":["boolean","null"]},"as_proposal":{"type":["boolean","null"]},"proposal":{"type":["string","null"]},"rationale":{"type":["string","null"]},"origin_x_nm":{"type":["integer","null"]},"origin_y_nm":{"type":["integer","null"]},"pitch_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]}},"required":["path"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.move_component",
        summary: "Move one native-project board component through the journaled board-package position path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "move_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("move-board-component"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
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
            p!("component", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"}},"required":["path","component","x_nm","y_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.rotate_component",
        summary: "Rotate one native-project board component through the journaled board-package rotation path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "rotate_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("rotate-board-component"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("rotation_deg", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"rotation_deg":{"type":"integer"}},"required":["path","component","rotation_deg"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.flip_component",
        summary: "Flip one native-project board component to a target copper side/layer through the journaled SetComponentSide path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "flip_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("flip-board-component"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("layer", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"layer":{"type":"integer"}},"required":["path","component","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_component",
        summary: "Delete one native-project board component through the journaled board-package removal path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-component"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"}},"required":["path","component"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_component_reference",
        summary: "Set one native-project board component reference through the journaled board-package property path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_component_reference",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-component-reference"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--reference",
                    param: "reference",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("reference", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"reference":{"type":"string"}},"required":["path","component","reference"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_component_value",
        summary: "Set one native-project board component value through the journaled board-package property path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_component_value",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-component-value"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--value",
                    param: "value",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("value", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"value":{"type":"string"}},"required":["path","component","value"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_component_part",
        summary: "Set one native-project board component part UUID through the journaled board-package property path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_component_part",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-component-part"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--part",
                    param: "part",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("part", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"part":{"type":"string"}},"required":["path","component","part"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_component_package",
        summary: "Set one native-project board component package UUID through the journaled board-package property path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_component_package",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-component-package"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
            p!("package", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"},"package":{"type":"string"}},"required":["path","component","package"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.lock_component",
        summary: "Lock one native-project board component through the journaled board-package lock path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "lock_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-component-locked"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"}},"required":["path","component"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.unlock_component",
        summary: "Lock one native-project board component through the journaled board-package lock path.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "unlock_board_component",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-board-component-locked"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--component",
                    param: "component",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("component", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"component":{"type":"string"}},"required":["path","component"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.draw_track",
        summary: "Draw one native-project board track.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "draw_board_track",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("draw-board-track"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
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
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net", Uuid, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
            p!("width_nm", Int, true, None),
            p!("layer", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"width_nm":{"type":"integer"},"layer":{"type":"integer"}},"required":["path","net","from_x_nm","from_y_nm","to_x_nm","to_y_nm","width_nm","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_track",
        summary: "Edit one native-project board track.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_track",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-track"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--track",
                    param: "track",
                },
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
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
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("track", Uuid, true, None),
            p!("net", Uuid, false, None),
            p!("from_x_nm", Int, false, None),
            p!("from_y_nm", Int, false, None),
            p!("to_x_nm", Int, false, None),
            p!("to_y_nm", Int, false, None),
            p!("width_nm", Int, false, None),
            p!("layer", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"track":{"type":"string"},"net":{"type":["string","null"]},"from_x_nm":{"type":["integer","null"]},"from_y_nm":{"type":["integer","null"]},"to_x_nm":{"type":["integer","null"]},"to_y_nm":{"type":["integer","null"]},"width_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]}},"required":["path","track"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_track",
        summary: "Delete one native-project board track.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_track",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-track"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--track",
                    param: "track",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("track", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"track":{"type":"string"}},"required":["path","track"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_via",
        summary: "Place one native-project board via.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_via",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-via"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
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
                    flag: "--drill-nm",
                    param: "drill_nm",
                },
                ArgvToken::Flag {
                    flag: "--diameter-nm",
                    param: "diameter_nm",
                },
                ArgvToken::Flag {
                    flag: "--from-layer",
                    param: "from_layer",
                },
                ArgvToken::Flag {
                    flag: "--to-layer",
                    param: "to_layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net", Uuid, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("drill_nm", Int, true, None),
            p!("diameter_nm", Int, true, None),
            p!("from_layer", Int, true, None),
            p!("to_layer", Int, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"drill_nm":{"type":"integer"},"diameter_nm":{"type":"integer"},"from_layer":{"type":"integer"},"to_layer":{"type":"integer"}},"required":["path","net","x_nm","y_nm","drill_nm","diameter_nm","from_layer","to_layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_via",
        summary: "Edit one native-project board via.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_via",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-via"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--via",
                    param: "via",
                },
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
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
                    flag: "--drill-nm",
                    param: "drill_nm",
                },
                ArgvToken::Flag {
                    flag: "--diameter-nm",
                    param: "diameter_nm",
                },
                ArgvToken::Flag {
                    flag: "--from-layer",
                    param: "from_layer",
                },
                ArgvToken::Flag {
                    flag: "--to-layer",
                    param: "to_layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("via", Uuid, true, None),
            p!("net", Uuid, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
            p!("drill_nm", Int, false, None),
            p!("diameter_nm", Int, false, None),
            p!("from_layer", Int, false, None),
            p!("to_layer", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"via":{"type":"string"},"net":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"drill_nm":{"type":["integer","null"]},"diameter_nm":{"type":["integer","null"]},"from_layer":{"type":["integer","null"]},"to_layer":{"type":["integer","null"]}},"required":["path","via"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_via",
        summary: "Delete one native-project board via.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_via",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-via"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--via",
                    param: "via",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("via", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"via":{"type":"string"}},"required":["path","via"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_zone",
        summary: "Place one native-project board copper zone boundary.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_zone",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-zone"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
                },
                ArgvToken::Repeated {
                    flag: "--vertex",
                    param: "vertices",
                },
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--priority",
                    param: "priority",
                },
                ArgvToken::Flag {
                    flag: "--thermal-relief",
                    param: "thermal_relief",
                },
                ArgvToken::Flag {
                    flag: "--thermal-gap-nm",
                    param: "thermal_gap_nm",
                },
                ArgvToken::Flag {
                    flag: "--thermal-spoke-width-nm",
                    param: "thermal_spoke_width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net", Uuid, true, None),
            p!("vertices", StrList, true, None),
            p!("layer", Int, true, None),
            p!("thermal_gap_nm", Int, true, None),
            p!("thermal_spoke_width_nm", Int, true, None),
            p!("priority", Int, false, None),
            p!("thermal_relief", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net":{"type":"string"},"vertices":{"type":"array","items":{"type":"string"}},"layer":{"type":"integer"},"priority":{"type":["integer","null"]},"thermal_relief":{"type":["boolean","null"]},"thermal_gap_nm":{"type":"integer"},"thermal_spoke_width_nm":{"type":"integer"}},"required":["path","net","vertices","layer","thermal_gap_nm","thermal_spoke_width_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_zone",
        summary: "Edit one native-project board copper zone boundary.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_zone",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-zone"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--zone",
                    param: "zone",
                },
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
                },
                ArgvToken::Repeated {
                    flag: "--vertex",
                    param: "vertices",
                },
                ArgvToken::Flag {
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--priority",
                    param: "priority",
                },
                ArgvToken::Flag {
                    flag: "--thermal-relief",
                    param: "thermal_relief",
                },
                ArgvToken::Flag {
                    flag: "--thermal-gap-nm",
                    param: "thermal_gap_nm",
                },
                ArgvToken::Flag {
                    flag: "--thermal-spoke-width-nm",
                    param: "thermal_spoke_width_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("zone", Uuid, true, None),
            p!("net", Uuid, false, None),
            p!("vertices", StrList, false, None),
            p!("layer", Int, false, None),
            p!("priority", Int, false, None),
            p!("thermal_relief", Bool, false, None),
            p!("thermal_gap_nm", Int, false, None),
            p!("thermal_spoke_width_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"zone":{"type":"string"},"net":{"type":["string","null"]},"vertices":{"type":["array","null"],"items":{"type":"string"}},"layer":{"type":["integer","null"]},"priority":{"type":["integer","null"]},"thermal_relief":{"type":["boolean","null"]},"thermal_gap_nm":{"type":["integer","null"]},"thermal_spoke_width_nm":{"type":["integer","null"]}},"required":["path","zone"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_zone",
        summary: "Delete one native-project board copper zone boundary.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_zone",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-zone"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--zone",
                    param: "zone",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("zone", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"zone":{"type":"string"}},"required":["path","zone"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_pad",
        summary: "Place one native-project board pad.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_pad",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-pad"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--package",
                    param: "package",
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
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--shape",
                    param: "shape",
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
                    flag: "--net",
                    param: "net",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("package", Uuid, true, None),
            p!("name", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("layer", Int, true, None),
            p!("shape", Str, false, None),
            p!("diameter_nm", Int, false, None),
            p!("width_nm", Int, false, None),
            p!("height_nm", Int, false, None),
            p!("net", Uuid, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"package":{"type":"string"},"name":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"},"shape":{"type":["string","null"]},"diameter_nm":{"type":["integer","null"]},"width_nm":{"type":["integer","null"]},"height_nm":{"type":["integer","null"]},"net":{"type":["string","null"]}},"required":["path","package","name","x_nm","y_nm","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_pad",
        summary: "Edit one native-project board pad.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_pad",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-pad"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
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
                ArgvToken::Flag {
                    flag: "--shape",
                    param: "shape",
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
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pad", Uuid, true, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
            p!("layer", Int, false, None),
            p!("shape", Str, false, None),
            p!("diameter_nm", Int, false, None),
            p!("width_nm", Int, false, None),
            p!("height_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pad":{"type":"string"},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"shape":{"type":["string","null"]},"diameter_nm":{"type":["integer","null"]},"width_nm":{"type":["integer","null"]},"height_nm":{"type":["integer","null"]}},"required":["path","pad"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_pad",
        summary: "Delete one native-project board pad.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_pad",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-pad"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("pad", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pad":{"type":"string"}},"required":["path","pad"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_pad_net",
        summary: "Set one native-project board pad net assignment.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_pad_net",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-pad-net"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("pad", Uuid, true, None),
            p!("net", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pad":{"type":"string"},"net":{"type":"string"}},"required":["path","pad","net"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.clear_pad_net",
        summary: "Clear one native-project board pad net assignment.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "clear_board_pad_net",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("clear-board-pad-net"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--pad",
                    param: "pad",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("pad", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"pad":{"type":"string"}},"required":["path","pad"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_net",
        summary: "Place one native-project board net, optionally with controlled-impedance metadata.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_net",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-net"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--class",
                    param: "class",
                },
                ArgvToken::Flag {
                    flag: "--impedance-target-ohms",
                    param: "impedance_target_ohms",
                },
                ArgvToken::Flag {
                    flag: "--impedance-tolerance-pct",
                    param: "impedance_tolerance_pct",
                },
                ArgvToken::Flag {
                    flag: "--controlled-dielectric-layer",
                    param: "controlled_dielectric_layer",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("name", Str, true, None),
            p!("class", Uuid, true, None),
            p!("impedance_target_ohms", Str, false, None),
            p!("impedance_tolerance_pct", Str, false, None),
            p!("controlled_dielectric_layer", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"class":{"type":"string"},"impedance_target_ohms":{"type":["string","null"]},"impedance_tolerance_pct":{"type":["string","null"]},"controlled_dielectric_layer":{"type":["integer","null"]}},"required":["path","name","class"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_net",
        summary: "Edit one native-project board net, including controlled-impedance metadata.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_net",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-net"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--class",
                    param: "class",
                },
                ArgvToken::Flag {
                    flag: "--impedance-target-ohms",
                    param: "impedance_target_ohms",
                },
                ArgvToken::Flag {
                    flag: "--impedance-tolerance-pct",
                    param: "impedance_tolerance_pct",
                },
                ArgvToken::Flag {
                    flag: "--controlled-dielectric-layer",
                    param: "controlled_dielectric_layer",
                },
                ArgvToken::Switch {
                    flag: "--clear-controlled-impedance",
                    param: "clear_controlled_impedance",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net", Uuid, true, None),
            p!("name", Str, false, None),
            p!("class", Uuid, false, None),
            p!("impedance_target_ohms", Str, false, None),
            p!("impedance_tolerance_pct", Str, false, None),
            p!("controlled_dielectric_layer", Int, false, None),
            p!("clear_controlled_impedance", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net":{"type":"string"},"name":{"type":["string","null"]},"class":{"type":["string","null"]},"impedance_target_ohms":{"type":["string","null"]},"impedance_tolerance_pct":{"type":["string","null"]},"controlled_dielectric_layer":{"type":["integer","null"]},"clear_controlled_impedance":{"type":["boolean","null"]}},"required":["path","net"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_net",
        summary: "Delete one native-project board net.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_net",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-net"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net",
                    param: "net",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("net", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net":{"type":"string"}},"required":["path","net"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_board_name",
        summary: "Set the native-project board name.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_name",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-name"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("name", Str, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"}},"required":["path","name"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_outline",
        summary: "Replace the native-project board outline polygon.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_outline",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-outline"),
                ArgvToken::Param("path"),
                ArgvToken::Repeated {
                    flag: "--vertex",
                    param: "vertices",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("vertices", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"vertices":{"type":"array","items":{"type":"string"}}},"required":["path","vertices"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.set_stackup",
        summary: "Replace the native-project board stackup. Each layer is id:name:type:thickness_nm with optional material fields :dielectric_constant:loss_tangent:copper_weight_oz:roughness_um:material_name.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "set_board_stackup",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("set-board-stackup"),
                ArgvToken::Param("path"),
                ArgvToken::Repeated {
                    flag: "--layer",
                    param: "layers",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("layers", StrList, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"layers":{"type":"array","items":{"type":"string"}}},"required":["path","layers"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.add_default_top_stackup",
        summary: "Add the default top-side board stackup support layers (top copper, mask, silk, paste, and mechanical) without replacing compatible existing layers.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "add_default_top_stackup",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("add-default-top-stackup"),
                ArgvToken::Param("path"),
            ],
        },
        params: &[p!("path", Str, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_keepout",
        summary: "Place one native-project board keepout polygon.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_keepout",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-keepout"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
                ArgvToken::Repeated {
                    flag: "--vertex",
                    param: "vertices",
                },
                ArgvToken::Repeated {
                    flag: "--layer",
                    param: "layers",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("vertices", StrList, true, None),
            p!("layers", Int, true, None),
            p!("kind", Str, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"vertices":{"type":"array","items":{"type":"string"}},"layers":{"type":"array","items":{"type":"integer"}},"kind":{"type":"string"}},"required":["path","vertices","layers","kind"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_keepout",
        summary: "Edit one native-project board keepout polygon.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_keepout",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-keepout"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--keepout",
                    param: "keepout",
                },
                ArgvToken::Repeated {
                    flag: "--vertex",
                    param: "vertices",
                },
                ArgvToken::Repeated {
                    flag: "--layer",
                    param: "layers",
                },
                ArgvToken::Flag {
                    flag: "--kind",
                    param: "kind",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("keepout", Uuid, true, None),
            p!("vertices", StrList, false, None),
            p!("layers", Int, false, None),
            p!("kind", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"keepout":{"type":"string"},"vertices":{"type":["array","null"],"items":{"type":"string"}},"layers":{"type":["array","null"],"items":{"type":"integer"}},"kind":{"type":["string","null"]}},"required":["path","keepout"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_keepout",
        summary: "Delete one native-project board keepout polygon.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_keepout",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-keepout"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--keepout",
                    param: "keepout",
                },
            ],
        },
        params: &[p!("path", Str, true, None), p!("keepout", Uuid, true, None)],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"keepout":{"type":"string"}},"required":["path","keepout"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_dimension",
        summary: "Place one native-project board dimension.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_dimension",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-dimension"),
                ArgvToken::Param("path"),
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
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("from_x_nm", Int, true, None),
            p!("from_y_nm", Int, true, None),
            p!("to_x_nm", Int, true, None),
            p!("to_y_nm", Int, true, None),
            p!("layer", Int, true, None),
            p!("text", Str, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"from_x_nm":{"type":"integer"},"from_y_nm":{"type":"integer"},"to_x_nm":{"type":"integer"},"to_y_nm":{"type":"integer"},"layer":{"type":"integer"},"text":{"type":["string","null"]}},"required":["path","from_x_nm","from_y_nm","to_x_nm","to_y_nm","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_dimension",
        summary: "Edit one native-project board dimension.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_dimension",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-dimension"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--dimension",
                    param: "dimension",
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
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--text",
                    param: "text",
                },
                ArgvToken::Switch {
                    flag: "--clear-text",
                    param: "clear_text",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("dimension", Uuid, true, None),
            p!("from_x_nm", Int, false, None),
            p!("from_y_nm", Int, false, None),
            p!("to_x_nm", Int, false, None),
            p!("to_y_nm", Int, false, None),
            p!("layer", Int, false, None),
            p!("text", Str, false, None),
            p!("clear_text", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"dimension":{"type":"string"},"from_x_nm":{"type":["integer","null"]},"from_y_nm":{"type":["integer","null"]},"to_x_nm":{"type":["integer","null"]},"to_y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"text":{"type":["string","null"]},"clear_text":{"type":["boolean","null"]}},"required":["path","dimension"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_dimension",
        summary: "Delete one native-project board dimension.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_dimension",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-dimension"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--dimension",
                    param: "dimension",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("dimension", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"dimension":{"type":"string"}},"required":["path","dimension"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.place_text",
        summary: "Place one native-project board text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-text"),
                ArgvToken::Param("path"),
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
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
                ArgvToken::Flag {
                    flag: "--height-nm",
                    param: "height_nm",
                },
                ArgvToken::Flag {
                    flag: "--stroke-width-nm",
                    param: "stroke_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--render-intent",
                    param: "render_intent",
                },
                ArgvToken::Flag {
                    flag: "--family",
                    param: "family",
                },
                ArgvToken::Flag {
                    flag: "--style",
                    param: "style",
                },
                ArgvToken::Flag {
                    flag: "--style-class",
                    param: "style_class",
                },
                ArgvToken::Flag {
                    flag: "--h-align",
                    param: "h_align",
                },
                ArgvToken::Flag {
                    flag: "--v-align",
                    param: "v_align",
                },
                ArgvToken::Switch {
                    flag: "--mirrored",
                    param: "mirrored",
                },
                ArgvToken::Switch {
                    flag: "--keep-upright",
                    param: "keep_upright",
                },
                ArgvToken::Flag {
                    flag: "--line-spacing-ratio-ppm",
                    param: "line_spacing_ratio_ppm",
                },
                ArgvToken::Switch {
                    flag: "--bold",
                    param: "bold",
                },
                ArgvToken::Switch {
                    flag: "--italic",
                    param: "italic",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("text", Str, true, None),
            p!("x_nm", Int, true, None),
            p!("y_nm", Int, true, None),
            p!("layer", Int, true, None),
            p!("rotation_deg", Int, false, None),
            p!("height_nm", Int, false, None),
            p!("stroke_width_nm", Int, false, None),
            p!("render_intent", Str, false, None),
            p!("family", Str, false, None),
            p!("style", Str, false, None),
            p!("style_class", Str, false, None),
            p!("h_align", Str, false, None),
            p!("v_align", Str, false, None),
            p!("mirrored", Bool, false, None),
            p!("keep_upright", Bool, false, None),
            p!("line_spacing_ratio_ppm", Int, false, None),
            p!("bold", Bool, false, None),
            p!("italic", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"text":{"type":"string"},"x_nm":{"type":"integer"},"y_nm":{"type":"integer"},"layer":{"type":"integer"},"rotation_deg":{"type":["integer","null"]},"height_nm":{"type":["integer","null"]},"stroke_width_nm":{"type":["integer","null"]},"render_intent":{"type":["string","null"]},"family":{"type":["string","null"]},"style":{"type":["string","null"]},"style_class":{"type":["string","null"]},"h_align":{"type":["string","null"]},"v_align":{"type":["string","null"]},"mirrored":{"type":["boolean","null"]},"keep_upright":{"type":["boolean","null"]},"line_spacing_ratio_ppm":{"type":["integer","null"]},"bold":{"type":["boolean","null"]},"italic":{"type":["boolean","null"]}},"required":["path","text","x_nm","y_nm","layer"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_text",
        summary: "Edit one native-project board text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-text"),
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
                    flag: "--layer",
                    param: "layer",
                },
                ArgvToken::Flag {
                    flag: "--rotation-deg",
                    param: "rotation_deg",
                },
                ArgvToken::Flag {
                    flag: "--height-nm",
                    param: "height_nm",
                },
                ArgvToken::Flag {
                    flag: "--stroke-width-nm",
                    param: "stroke_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--render-intent",
                    param: "render_intent",
                },
                ArgvToken::Flag {
                    flag: "--family",
                    param: "family",
                },
                ArgvToken::Flag {
                    flag: "--style",
                    param: "style",
                },
                ArgvToken::Flag {
                    flag: "--style-class",
                    param: "style_class",
                },
                ArgvToken::Flag {
                    flag: "--h-align",
                    param: "h_align",
                },
                ArgvToken::Flag {
                    flag: "--v-align",
                    param: "v_align",
                },
                ArgvToken::Flag {
                    flag: "--mirrored",
                    param: "mirrored",
                },
                ArgvToken::Flag {
                    flag: "--keep-upright",
                    param: "keep_upright",
                },
                ArgvToken::Flag {
                    flag: "--line-spacing-ratio-ppm",
                    param: "line_spacing_ratio_ppm",
                },
                ArgvToken::Flag {
                    flag: "--bold",
                    param: "bold",
                },
                ArgvToken::Flag {
                    flag: "--italic",
                    param: "italic",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("text", Uuid, true, None),
            p!("value", Str, false, None),
            p!("x_nm", Int, false, None),
            p!("y_nm", Int, false, None),
            p!("layer", Int, false, None),
            p!("rotation_deg", Int, false, None),
            p!("height_nm", Int, false, None),
            p!("stroke_width_nm", Int, false, None),
            p!("render_intent", Str, false, None),
            p!("family", Str, false, None),
            p!("style", Str, false, None),
            p!("style_class", Str, false, None),
            p!("h_align", Str, false, None),
            p!("v_align", Str, false, None),
            p!("mirrored", Bool, false, None),
            p!("keep_upright", Bool, false, None),
            p!("line_spacing_ratio_ppm", Int, false, None),
            p!("bold", Bool, false, None),
            p!("italic", Bool, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"text":{"type":"string"},"value":{"type":["string","null"]},"x_nm":{"type":["integer","null"]},"y_nm":{"type":["integer","null"]},"layer":{"type":["integer","null"]},"rotation_deg":{"type":["integer","null"]},"height_nm":{"type":["integer","null"]},"stroke_width_nm":{"type":["integer","null"]},"render_intent":{"type":["string","null"]},"family":{"type":["string","null"]},"style":{"type":["string","null"]},"style_class":{"type":["string","null"]},"h_align":{"type":["string","null"]},"v_align":{"type":["string","null"]},"mirrored":{"type":["boolean","null"]},"keep_upright":{"type":["boolean","null"]},"line_spacing_ratio_ppm":{"type":["integer","null"]},"bold":{"type":["boolean","null"]},"italic":{"type":["boolean","null"]}},"required":["path","text"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_text",
        summary: "Delete one native-project board text object.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_text",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-text"),
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
        id: "datum.pcb.place_net_class",
        summary: "Place one native-project board net class.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "place_board_net_class",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("place-board-net-class"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--clearance-nm",
                    param: "clearance_nm",
                },
                ArgvToken::Flag {
                    flag: "--track-width-nm",
                    param: "track_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--via-drill-nm",
                    param: "via_drill_nm",
                },
                ArgvToken::Flag {
                    flag: "--via-diameter-nm",
                    param: "via_diameter_nm",
                },
                ArgvToken::Flag {
                    flag: "--diffpair-width-nm",
                    param: "diffpair_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--diffpair-gap-nm",
                    param: "diffpair_gap_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("name", Str, true, None),
            p!("clearance_nm", Int, true, None),
            p!("track_width_nm", Int, true, None),
            p!("via_drill_nm", Int, true, None),
            p!("via_diameter_nm", Int, true, None),
            p!("diffpair_width_nm", Int, false, None),
            p!("diffpair_gap_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"name":{"type":"string"},"clearance_nm":{"type":"integer"},"track_width_nm":{"type":"integer"},"via_drill_nm":{"type":"integer"},"via_diameter_nm":{"type":"integer"},"diffpair_width_nm":{"type":["integer","null"]},"diffpair_gap_nm":{"type":["integer","null"]}},"required":["path","name","clearance_nm","track_width_nm","via_drill_nm","via_diameter_nm"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.edit_net_class",
        summary: "Edit one native-project board net class.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "edit_board_net_class",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("edit-board-net-class"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net-class",
                    param: "net_class",
                },
                ArgvToken::Flag {
                    flag: "--name",
                    param: "name",
                },
                ArgvToken::Flag {
                    flag: "--clearance-nm",
                    param: "clearance_nm",
                },
                ArgvToken::Flag {
                    flag: "--track-width-nm",
                    param: "track_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--via-drill-nm",
                    param: "via_drill_nm",
                },
                ArgvToken::Flag {
                    flag: "--via-diameter-nm",
                    param: "via_diameter_nm",
                },
                ArgvToken::Flag {
                    flag: "--diffpair-width-nm",
                    param: "diffpair_width_nm",
                },
                ArgvToken::Flag {
                    flag: "--diffpair-gap-nm",
                    param: "diffpair_gap_nm",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net_class", Uuid, true, None),
            p!("name", Str, false, None),
            p!("clearance_nm", Int, false, None),
            p!("track_width_nm", Int, false, None),
            p!("via_drill_nm", Int, false, None),
            p!("via_diameter_nm", Int, false, None),
            p!("diffpair_width_nm", Int, false, None),
            p!("diffpair_gap_nm", Int, false, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net_class":{"type":"string"},"name":{"type":["string","null"]},"clearance_nm":{"type":["integer","null"]},"track_width_nm":{"type":["integer","null"]},"via_drill_nm":{"type":["integer","null"]},"via_diameter_nm":{"type":["integer","null"]},"diffpair_width_nm":{"type":["integer","null"]},"diffpair_gap_nm":{"type":["integer","null"]}},"required":["path","net_class"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.pcb.delete_net_class",
        summary: "Delete one native-project board net class.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::Cli {
            method: "delete_board_net_class",
            argv: &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("delete-board-net-class"),
                ArgvToken::Param("path"),
                ArgvToken::Flag {
                    flag: "--net-class",
                    param: "net_class",
                },
            ],
        },
        params: &[
            p!("path", Str, true, None),
            p!("net_class", Uuid, true, None),
        ],
        schema_json_override: Some(
            r###"{"type":"object","properties":{"path":{"type":"string"},"net_class":{"type":"string"}},"required":["path","net_class"]}"###,
        ),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
