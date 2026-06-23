#!/usr/bin/env python3
"""Native pool-library MCP tool schemas."""

from __future__ import annotations

POOL_LIBRARY_KIND_SCHEMA = {
    "type": "string",
    "enum": [
        "units",
        "symbols",
        "entities",
        "parts",
        "packages",
        "footprints",
        "padstacks",
        "pin_pad_maps",
    ],
}

LIBRARY_TOOL_SCHEMAS = {
    "get_pool_library_objects": {
        "description": "List resolver-discovered native pool-library objects.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": ["string", "null"]},
                "kind": POOL_LIBRARY_KIND_SCHEMA,
                "object": {"type": ["string", "null"]},
                "include_payload": {"type": ["boolean", "null"]},
            },
            "required": ["path"],
        },
    },
    "show_pool_library_object": {
        "description": "Show one resolver-discovered native pool-library object with its materialized payload.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "object": {"type": "string"},
                "pool": {"type": ["string", "null"]},
                "kind": POOL_LIBRARY_KIND_SCHEMA,
            },
            "required": ["path", "object"],
        },
    },
    "get_pool_model_blobs": {
        "description": "List and verify native pool behavioural-model blobs and attachment references.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": ["string", "null"]},
                "role": {"type": ["string", "null"]},
                "sha256": {"type": ["string", "null"]},
            },
            "required": ["path"],
        },
    },
    "gc_pool_model_blobs": {
        "description": "Dry-run or apply conservative garbage collection for orphaned native pool behavioural-model blobs.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": ["string", "null"]},
                "role": {"type": ["string", "null"]},
                "sha256": {"type": ["string", "null"]},
                "apply": {"type": ["boolean", "null"]},
            },
            "required": ["path"],
        },
    },
    "create_pool_library_object": {
        "description": "Create one native pool-library object through the journaled project commit path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "kind": POOL_LIBRARY_KIND_SCHEMA,
                "object": {"type": "string"},
                "from_json": {"type": "string"},
            },
            "required": ["path", "kind", "object", "from_json"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "create_pool_unit": {
        "description": "Create one typed native pool unit through the journaled project commit path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "unit": {"type": "string"},
                "name": {"type": "string"},
                "manufacturer": {"type": "string"},
            },
            "required": ["path", "unit", "name"],
        },
        "x_dispatch_defaults": {"pool": "pool", "manufacturer": ""},
    },
    "set_pool_unit_pin": {
        "description": "Set one typed native pool unit pin entry.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "unit": {"type": "string"},
                "pin": {"type": "string"}, "name": {"type": "string"}, "direction": {"type": "string"},
                "swap_group": {"type": "integer"},
            },
            "required": ["path", "unit", "pin", "name"],
        },
        "x_dispatch_defaults": {"pool": "pool", "direction": "Passive", "swap_group": 0},
    },
    "create_pool_symbol": {
        "description": "Create one typed native pool symbol for an existing pool unit through the journaled project commit path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "symbol": {"type": "string"},
                "unit": {"type": "string"},
                "name": {"type": "string"},
            },
            "required": ["path", "symbol", "unit", "name"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_line": {
        "description": "Append one typed native pool symbol line primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"},
                "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"},
                "width_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_rect": {
        "description": "Append one typed native pool symbol rectangle primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "min_x_nm": {"type": "integer"}, "min_y_nm": {"type": "integer"},
                "max_x_nm": {"type": "integer"}, "max_y_nm": {"type": "integer"},
                "width_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "min_x_nm", "min_y_nm", "max_x_nm", "max_y_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_circle": {
        "description": "Append one typed native pool symbol circle primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "center_x_nm": {"type": "integer"}, "center_y_nm": {"type": "integer"},
                "radius_nm": {"type": "integer"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "center_x_nm", "center_y_nm", "radius_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_arc": {
        "description": "Append one typed native pool symbol arc primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "radius_nm": {"type": "integer"},
                "start_angle": {"type": "integer"}, "end_angle": {"type": "integer"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "x_nm", "y_nm", "radius_nm", "start_angle", "end_angle", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_polygon": {
        "description": "Append one typed native pool symbol polygon or polyline primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "vertices": {"type": "string"}, "closed": {"type": "boolean"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "vertices", "closed", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_symbol_text": {
        "description": "Append one typed native pool symbol text primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "text": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "rotation": {"type": "integer"},
            },
            "required": ["path", "symbol", "text", "x_nm", "y_nm", "rotation"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "set_pool_symbol_pin_anchor": {
        "description": "Set one typed native pool symbol pin anchor for a unit pin.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "symbol": {"type": "string"},
                "pin": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"},
            },
            "required": ["path", "symbol", "pin", "x_nm", "y_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "create_pool_entity": {
        "description": "Create one typed native pool entity with an initial gate over an existing unit and symbol.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "entity": {"type": "string"},
                "gate": {"type": "string"},
                "unit": {"type": "string"},
                "symbol": {"type": "string"},
                "name": {"type": "string"},
                "prefix": {"type": "string"},
                "manufacturer": {"type": "string"},
                "gate_name": {"type": "string"},
            },
            "required": ["path", "entity", "gate", "unit", "symbol", "name", "prefix"],
        },
        "x_dispatch_defaults": {"pool": "pool", "manufacturer": "", "gate_name": "A"},
    },
    "create_pool_padstack": {
        "description": "Create one typed native pool padstack with optional circle/rect aperture and drill diameter.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "padstack": {"type": "string"},
                "name": {"type": "string"},
                "aperture": {"type": "string"},
                "diameter_nm": {"type": "integer"},
                "width_nm": {"type": "integer"},
                "height_nm": {"type": "integer"},
                "drill_nm": {"type": "integer"},
            },
            "required": ["path", "padstack", "name"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "create_pool_package": {
        "description": "Create one typed native pool package with one initial pad referencing an existing padstack.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "name": {"type": "string"}, "pad": {"type": "string"}, "padstack": {"type": "string"},
                "pad_name": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"},
                "layer": {"type": "integer"},
            },
            "required": ["path", "package", "name", "pad", "padstack"],
        },
        "x_dispatch_defaults": {"pool": "pool", "pad_name": "1", "x_nm": 0, "y_nm": 0, "layer": 1},
    },
    "set_pool_package_pad": {
        "description": "Set one typed native pool package pad entry referencing an existing padstack.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "pad": {"type": "string"}, "padstack": {"type": "string"}, "pad_name": {"type": "string"},
                "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "layer": {"type": "integer"},
            },
            "required": ["path", "package", "pad", "padstack"],
        },
        "x_dispatch_defaults": {"pool": "pool", "pad_name": "1", "x_nm": 0, "y_nm": 0, "layer": 1},
    },
    "set_pool_package_courtyard_rect": {
        "description": "Set typed native pool package rectangular courtyard geometry.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "min_x_nm": {"type": "integer"}, "min_y_nm": {"type": "integer"},
                "max_x_nm": {"type": "integer"}, "max_y_nm": {"type": "integer"},
            },
            "required": ["path", "package", "min_x_nm", "min_y_nm", "max_x_nm", "max_y_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "set_pool_package_courtyard_polygon": {
        "description": "Set typed native pool package polygon courtyard geometry.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "vertices": {"type": "string"},
            },
            "required": ["path", "package", "vertices"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_line": {
        "description": "Append one typed native pool package silkscreen line primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"},
                "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"},
                "width_nm": {"type": "integer"},
            },
            "required": ["path", "package", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_rect": {
        "description": "Append one typed native pool package silkscreen rectangle primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "min_x_nm": {"type": "integer"}, "min_y_nm": {"type": "integer"},
                "max_x_nm": {"type": "integer"}, "max_y_nm": {"type": "integer"},
                "width_nm": {"type": "integer"},
            },
            "required": ["path", "package", "min_x_nm", "min_y_nm", "max_x_nm", "max_y_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_polygon": {
        "description": "Append one typed native pool package silkscreen polygon or polyline primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "vertices": {"type": "string", "description": "Semicolon-separated x,y vertex pairs, e.g. x,y;x,y;x,y."},
                "closed": {"type": "boolean"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "package", "vertices", "closed", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_circle": {
        "description": "Append one typed native pool package silkscreen circle primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "center_x_nm": {"type": "integer"}, "center_y_nm": {"type": "integer"},
                "radius_nm": {"type": "integer"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "package", "center_x_nm", "center_y_nm", "radius_nm", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_arc": {
        "description": "Append one typed native pool package silkscreen arc primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "radius_nm": {"type": "integer"},
                "start_angle": {"type": "integer"}, "end_angle": {"type": "integer"}, "width_nm": {"type": "integer"},
            },
            "required": ["path", "package", "x_nm", "y_nm", "radius_nm", "start_angle", "end_angle", "width_nm"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_silkscreen_text": {
        "description": "Append one typed native pool package silkscreen text primitive.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "text": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "rotation": {"type": "number"},
            },
            "required": ["path", "package", "text", "x_nm", "y_nm", "rotation"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "add_pool_package_model_3d": {
        "description": "Attach one 3D model to a typed native pool package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "model_path": {"type": "string"}, "transform_json": {"type": ["string", "null"]},
                "format": {"type": ["string", "null"]},
                "tx_nm": {"type": ["integer", "null"]}, "ty_nm": {"type": ["integer", "null"]}, "tz_nm": {"type": ["integer", "null"]},
                "roll_tenths_deg": {"type": ["integer", "null"]}, "pitch_tenths_deg": {"type": ["integer", "null"]}, "yaw_tenths_deg": {"type": ["integer", "null"]},
                "scale": {"type": ["string", "number", "null"]},
            },
            "required": ["path", "package", "model_path"],
        },
        "x_dispatch_defaults": {"pool": "pool", "transform_json": None},
    },
    "set_pool_package_body_heights": {
        "description": "Set or clear typed native pool package body-height metadata.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "package": {"type": "string"},
                "body_height_nm": {"type": ["integer", "null"]},
                "body_height_mounted_nm": {"type": ["integer", "null"]},
                "clear": {"type": ["boolean", "null"]},
            },
            "required": ["path", "package"],
        },
        "x_dispatch_defaults": {"pool": "pool", "body_height_nm": None, "body_height_mounted_nm": None, "clear": None},
    },
    "create_pool_part": {
        "description": "Create one typed native pool part binding an existing entity to an existing package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "entity": {"type": "string"}, "package": {"type": "string"}, "mpn": {"type": "string"},
                "manufacturer": {"type": "string"}, "value": {"type": "string"}, "description": {"type": "string"},
                "datasheet": {"type": "string"}, "lifecycle": {"type": "string"},
            },
            "required": ["path", "part", "entity", "package", "mpn"],
        },
        "x_dispatch_defaults": {"pool": "pool", "manufacturer": "", "value": "", "description": "", "datasheet": "", "lifecycle": "Active"},
    },
    "set_pool_part_metadata": {
        "description": "Set typed native pool part metadata fields.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mpn": {"type": "string"}, "manufacturer": {"type": "string"}, "manufacturer_jep106": {"type": "integer"}, "value": {"type": "string"},
                "description": {"type": "string"}, "datasheet": {"type": "string"}, "lifecycle": {"type": "string"},
            },
            "required": ["path", "part"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "set_pool_part_parametric": {
        "description": "Set typed native pool part parametric key-value fields.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string", "enum": ["merge", "replace"]},
                "params": {"type": "object", "additionalProperties": {"type": "string"}},
            },
            "required": ["path", "part", "params"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "set_pool_part_orderable_mpns": {
        "description": "Set typed native pool part orderable manufacturer part numbers.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string", "enum": ["merge", "replace"]},
                "mpns": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "part", "mpns"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "set_pool_part_tags": {
        "description": "Set typed native pool part tags.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string", "enum": ["merge", "replace"]},
                "tags": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "part", "tags"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "set_pool_part_packaging_options": {
        "description": "Set typed native pool part encoded packaging option payloads.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string", "enum": ["merge", "replace"]},
                "options": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "part", "options"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "set_pool_part_supply_chain": {
        "description": "Set or clear typed native pool part supply-chain offer metadata.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "clear": {"type": "boolean"}, "checked_at": {"type": ["string", "null"]},
                "offers": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "part"],
        },
        "x_dispatch_defaults": {"pool": "pool", "clear": False, "checked_at": None, "offers": []},
    },
    "set_pool_part_behavioural_models": {
        "description": "Set typed native pool part behavioural model attachment payloads.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string", "enum": ["merge", "replace"]},
                "models": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["path", "part", "models"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "attach_pool_part_model": {
        "description": "Attach one model source file to a typed native pool part without parsing the model payload.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "source": {"type": "string"}, "role": {"type": "string"}, "dialect": {"type": ["string", "null"]},
                "model_names": {"type": "array", "items": {"type": "string"}},
                "encrypted": {"type": "boolean"}, "encryption_scheme": {"type": ["string", "null"]},
                "vendor": {"type": ["string", "null"]}, "fetched_at": {"type": ["string", "null"]},
                "format_metadata_json": {"type": ["string", "null"]},
            },
            "required": ["path", "part", "source", "role"],
        },
        "x_dispatch_defaults": {"pool": "pool", "dialect": None, "model_names": [], "encrypted": False, "encryption_scheme": None, "vendor": None, "fetched_at": None, "format_metadata_json": None},
    },
    "detach_pool_part_model": {
        "description": "Detach one model attachment from a typed native pool part.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "attachment": {"type": ["string", "null"]}, "model": {"type": ["string", "null"]},
            },
            "required": ["path", "part"],
        },
        "x_dispatch_defaults": {"pool": "pool", "attachment": None, "model": None},
    },
    "set_pool_part_thermal": {
        "description": "Set or clear typed native pool part thermal characteristics.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "theta_ja_c_per_w": {"type": ["number", "string", "null"]},
                "theta_jc_top_c_per_w": {"type": ["number", "string", "null"]},
                "theta_jc_bot_c_per_w": {"type": ["number", "string", "null"]},
                "theta_jb_c_per_w": {"type": ["number", "string", "null"]},
                "max_junction_c": {"type": ["number", "string", "null"]},
                "thermal_reference": {"type": ["string", "null"]},
                "clear": {"type": "boolean"},
            },
            "required": ["path", "part"],
        },
        "x_dispatch_defaults": {
            "pool": "pool",
            "theta_ja_c_per_w": None,
            "theta_jc_top_c_per_w": None,
            "theta_jc_bot_c_per_w": None,
            "theta_jb_c_per_w": None,
            "max_junction_c": None,
            "thermal_reference": None,
            "clear": False,
        },
    },
    "set_pool_part_pad_map_entry": {
        "description": "Set one typed native pool part pad-map entry from package pad to entity gate pin.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "pad": {"type": "string"}, "gate": {"type": "string"}, "pin": {"type": "string"},
            },
            "required": ["path", "part", "pad", "gate", "pin"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "set_pool_part_pad_map": {
        "description": "Set typed native pool part pad-map entries in merge or replace mode.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}, "pool": {"type": "string"}, "part": {"type": "string"},
                "mode": {"type": "string"},
                "entries": {"type": "array", "items": {"type": "object", "properties": {"pad": {"type": "string"}, "gate": {"type": "string"}, "pin": {"type": "string"}}, "required": ["pad", "gate", "pin"]}},
            },
            "required": ["path", "part", "entries"],
        },
        "x_dispatch_defaults": {"pool": "pool", "mode": "merge"},
    },
    "set_pool_library_object": {
        "description": "Replace one native pool-library object through the journaled project commit path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "kind": POOL_LIBRARY_KIND_SCHEMA,
                "object": {"type": "string"},
                "from_json": {"type": "string"},
            },
            "required": ["path", "kind", "object", "from_json"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
    "delete_pool_library_object": {
        "description": "Delete one native pool-library object through the journaled project commit path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "pool": {"type": "string"},
                "kind": POOL_LIBRARY_KIND_SCHEMA,
                "object": {"type": "string"},
            },
            "required": ["path", "kind", "object"],
        },
        "x_dispatch_defaults": {"pool": "pool"},
    },
}
