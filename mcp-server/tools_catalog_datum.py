"""Canonical datum.* MCP tool aliases."""

from tools_catalog_checks import CHECK_TOOL_SCHEMAS
from tools_catalog_import_map import IMPORT_MAP_TOOL_SCHEMAS
from tools_catalog_journal import JOURNAL_TOOL_SCHEMAS
from tools_catalog_library import LIBRARY_TOOL_SCHEMAS
from tools_catalog_output_jobs import OUTPUT_JOB_TOOL_SCHEMAS
from tools_catalog_proposals import PROPOSAL_TOOL_SCHEMAS
from tools_catalog_relationships import RELATIONSHIP_TOOL_SCHEMAS

DATUM_CONTEXT_SCHEMA = {
    "description": "Return the current Datum session/context envelope, including project identity, model revision, actor type, capabilities, visible artifacts/check runs, provenance seed, and refresh metadata.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "session": {"type": ["string", "null"]},
            "path": {"type": ["string", "null"]},
            "project_root": {"type": ["string", "null"]},
        },
    },
}

DATUM_CONTEXT_SESSION_EVENTS_SCHEMA = {
    "description": "Return recorded Datum tool-session events for a terminal/session, optionally filtered by event kind, origin, command id, or execution id.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "session": {"type": ["string", "null"]},
            "path": {"type": ["string", "null"]},
            "project_root": {"type": ["string", "null"]},
            "event_kind": {"type": ["string", "null"]},
            "origin": {"type": ["string", "null"]},
            "command_id": {"type": ["string", "null"]},
            "execution_id": {"type": ["string", "null"]},
            "limit": {"type": ["integer", "null"]},
        },
    },
}

DATUM_CONTEXT_SESSION_ACTIVITY_SCHEMA = {
    "description": "Return a compact Datum tool-session activity summary for a terminal/session. The primary agent-facing result is executions[], with start/end/duration, lifecycle/exit status, and per-execution I/O totals/previews. Results can be filtered by event kind, origin, command id, or execution id.",
    "inputSchema": DATUM_CONTEXT_SESSION_EVENTS_SCHEMA["inputSchema"],
}

DATUM_EMPTY_QUERY_SCHEMA = {
    "description": "Canonical Datum read-only query alias over the current open session.",
    "inputSchema": {"type": "object", "properties": {}},
}

DATUM_SYMBOL_UUID_QUERY_SCHEMA = {
    "description": "Canonical Datum read-only query alias for one schematic symbol object.",
    "inputSchema": {
        "type": "object",
        "properties": {"symbol_uuid": {"type": "string"}},
        "required": ["symbol_uuid"],
    },
}

DATUM_HIERARCHY_QUERY_SCHEMA = {
    "description": "Canonical Datum hierarchy query. With path, reads native project hierarchy; without path, uses legacy open-session hierarchy.",
    "inputSchema": {
        "type": "object",
        "properties": {"path": {"type": ["string", "null"]}},
    },
}

DATUM_PATH_QUERY_SCHEMA = {
    "description": "Canonical Datum read-only query alias for one native project path.",
    "inputSchema": {
        "type": "object",
        "properties": {"path": {"type": "string"}},
        "required": ["path"],
    },
}

DATUM_PLACE_COMPONENT_SCHEMA = {
    "description": "Place one native-project board component through the journaled board-package creation path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "part": {"type": "string"},
            "package": {"type": "string"},
            "reference": {"type": "string"},
            "value": {"type": "string"},
            "x_nm": {"type": "integer"},
            "y_nm": {"type": "integer"},
            "layer": {"type": "integer"},
        },
        "required": ["path", "part", "package", "reference", "value", "x_nm", "y_nm", "layer"],
    },
}

DATUM_MOVE_COMPONENT_SCHEMA = {
    "description": "Move one native-project board component through the journaled board-package position path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "component": {"type": "string"},
            "x_nm": {"type": "integer"},
            "y_nm": {"type": "integer"},
        },
        "required": ["path", "component", "x_nm", "y_nm"],
    },
}

DATUM_ROTATE_COMPONENT_SCHEMA = {
    "description": "Rotate one native-project board component through the journaled board-package rotation path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "component": {"type": "string"},
            "rotation_deg": {"type": "integer"},
        },
        "required": ["path", "component", "rotation_deg"],
    },
}

DATUM_FLIP_COMPONENT_SCHEMA = {
    "description": "Flip one native-project board component to a target copper side/layer through the journaled SetComponentSide path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "component": {"type": "string"},
            "layer": {"type": "integer"},
        },
        "required": ["path", "component", "layer"],
    },
}

DATUM_DELETE_COMPONENT_SCHEMA = {
    "description": "Delete one native-project board component through the journaled board-package removal path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "component": {"type": "string"},
        },
        "required": ["path", "component"],
    },
}

DATUM_COMPONENT_PROPERTY_SCHEMAS = {
    "datum.pcb.set_component_reference": {
        "description": "Set one native-project board component reference through the journaled board-package property path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component": {"type": "string"},
                "reference": {"type": "string"},
            },
            "required": ["path", "component", "reference"],
        },
    },
    "datum.pcb.set_component_value": {
        "description": "Set one native-project board component value through the journaled board-package property path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component": {"type": "string"},
                "value": {"type": "string"},
            },
            "required": ["path", "component", "value"],
        },
    },
    "datum.pcb.set_component_part": {
        "description": "Set one native-project board component part UUID through the journaled board-package property path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component": {"type": "string"},
                "part": {"type": "string"},
            },
            "required": ["path", "component", "part"],
        },
    },
    "datum.pcb.set_component_package": {
        "description": "Set one native-project board component package UUID through the journaled board-package property path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "component": {"type": "string"},
                "package": {"type": "string"},
            },
            "required": ["path", "component", "package"],
        },
    },
}

DATUM_LOCK_COMPONENT_SCHEMA = {
    "description": "Lock one native-project board component through the journaled board-package lock path.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "path": {"type": "string"},
            "component": {"type": "string"},
        },
        "required": ["path", "component"],
    },
}

DATUM_PCB_PRIMITIVE_SCHEMAS = {
    "datum.pcb.draw_track": {"description": "Draw one native-project board track.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net": {"type": "string"}, "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"}, "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"}, "width_nm": {"type": "integer"}, "layer": {"type": "integer"}}, "required": ["path", "net", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "width_nm", "layer"]}},
    "datum.pcb.edit_track": {"description": "Edit one native-project board track.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "track": {"type": "string"}, "net": {"type": ["string", "null"]}, "from_x_nm": {"type": ["integer", "null"]}, "from_y_nm": {"type": ["integer", "null"]}, "to_x_nm": {"type": ["integer", "null"]}, "to_y_nm": {"type": ["integer", "null"]}, "width_nm": {"type": ["integer", "null"]}, "layer": {"type": ["integer", "null"]}}, "required": ["path", "track"]}},
    "datum.pcb.delete_track": {"description": "Delete one native-project board track.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "track": {"type": "string"}}, "required": ["path", "track"]}},
    "datum.pcb.place_via": {"description": "Place one native-project board via.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "drill_nm": {"type": "integer"}, "diameter_nm": {"type": "integer"}, "from_layer": {"type": "integer"}, "to_layer": {"type": "integer"}}, "required": ["path", "net", "x_nm", "y_nm", "drill_nm", "diameter_nm", "from_layer", "to_layer"]}},
    "datum.pcb.edit_via": {"description": "Edit one native-project board via.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "via": {"type": "string"}, "net": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "drill_nm": {"type": ["integer", "null"]}, "diameter_nm": {"type": ["integer", "null"]}, "from_layer": {"type": ["integer", "null"]}, "to_layer": {"type": ["integer", "null"]}}, "required": ["path", "via"]}},
    "datum.pcb.delete_via": {"description": "Delete one native-project board via.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "via": {"type": "string"}}, "required": ["path", "via"]}},
    "datum.pcb.place_zone": {"description": "Place one native-project board copper zone boundary.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net": {"type": "string"}, "vertices": {"type": "array", "items": {"type": "string"}}, "layer": {"type": "integer"}, "priority": {"type": ["integer", "null"]}, "thermal_relief": {"type": ["boolean", "null"]}, "thermal_gap_nm": {"type": "integer"}, "thermal_spoke_width_nm": {"type": "integer"}}, "required": ["path", "net", "vertices", "layer", "thermal_gap_nm", "thermal_spoke_width_nm"]}},
    "datum.pcb.edit_zone": {"description": "Edit one native-project board copper zone boundary.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "zone": {"type": "string"}, "net": {"type": ["string", "null"]}, "vertices": {"type": ["array", "null"], "items": {"type": "string"}}, "layer": {"type": ["integer", "null"]}, "priority": {"type": ["integer", "null"]}, "thermal_relief": {"type": ["boolean", "null"]}, "thermal_gap_nm": {"type": ["integer", "null"]}, "thermal_spoke_width_nm": {"type": ["integer", "null"]}}, "required": ["path", "zone"]}},
    "datum.pcb.delete_zone": {"description": "Delete one native-project board copper zone boundary.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "zone": {"type": "string"}}, "required": ["path", "zone"]}},
    "datum.pcb.place_pad": {"description": "Place one native-project board pad.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "package": {"type": "string"}, "name": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "layer": {"type": "integer"}, "shape": {"type": ["string", "null"]}, "diameter_nm": {"type": ["integer", "null"]}, "width_nm": {"type": ["integer", "null"]}, "height_nm": {"type": ["integer", "null"]}, "net": {"type": ["string", "null"]}}, "required": ["path", "package", "name", "x_nm", "y_nm", "layer"]}},
    "datum.pcb.edit_pad": {"description": "Edit one native-project board pad.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "pad": {"type": "string"}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "layer": {"type": ["integer", "null"]}, "shape": {"type": ["string", "null"]}, "diameter_nm": {"type": ["integer", "null"]}, "width_nm": {"type": ["integer", "null"]}, "height_nm": {"type": ["integer", "null"]}}, "required": ["path", "pad"]}},
    "datum.pcb.delete_pad": {"description": "Delete one native-project board pad.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "pad": {"type": "string"}}, "required": ["path", "pad"]}},
    "datum.pcb.set_pad_net": {"description": "Set one native-project board pad net assignment.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "pad": {"type": "string"}, "net": {"type": "string"}}, "required": ["path", "pad", "net"]}},
    "datum.pcb.clear_pad_net": {"description": "Clear one native-project board pad net assignment.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "pad": {"type": "string"}}, "required": ["path", "pad"]}},
    "datum.pcb.place_net": {"description": "Place one native-project board net, optionally with controlled-impedance metadata.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "name": {"type": "string"}, "class": {"type": "string"}, "impedance_target_ohms": {"type": ["string", "null"]}, "impedance_tolerance_pct": {"type": ["string", "null"]}, "controlled_dielectric_layer": {"type": ["integer", "null"]}}, "required": ["path", "name", "class"]}},
    "datum.pcb.edit_net": {"description": "Edit one native-project board net, including controlled-impedance metadata.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net": {"type": "string"}, "name": {"type": ["string", "null"]}, "class": {"type": ["string", "null"]}, "impedance_target_ohms": {"type": ["string", "null"]}, "impedance_tolerance_pct": {"type": ["string", "null"]}, "controlled_dielectric_layer": {"type": ["integer", "null"]}, "clear_controlled_impedance": {"type": ["boolean", "null"]}}, "required": ["path", "net"]}},
    "datum.pcb.delete_net": {"description": "Delete one native-project board net.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net": {"type": "string"}}, "required": ["path", "net"]}},
    "datum.pcb.set_board_name": {"description": "Set the native-project board name.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "name": {"type": "string"}}, "required": ["path", "name"]}},
    "datum.pcb.set_outline": {"description": "Replace the native-project board outline polygon.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "vertices": {"type": "array", "items": {"type": "string"}}}, "required": ["path", "vertices"]}},
    "datum.pcb.set_stackup": {"description": "Replace the native-project board stackup. Each layer is id:name:type:thickness_nm with optional material fields :dielectric_constant:loss_tangent:copper_weight_oz:roughness_um:material_name.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "layers": {"type": "array", "items": {"type": "string"}}}, "required": ["path", "layers"]}},
    "datum.pcb.add_default_top_stackup": {"description": "Add the default top-side board stackup support layers (top copper, mask, silk, paste, and mechanical) without replacing compatible existing layers.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}},
    "datum.pcb.place_keepout": {"description": "Place one native-project board keepout polygon.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "vertices": {"type": "array", "items": {"type": "string"}}, "layers": {"type": "array", "items": {"type": "integer"}}, "kind": {"type": "string"}}, "required": ["path", "vertices", "layers", "kind"]}},
    "datum.pcb.edit_keepout": {"description": "Edit one native-project board keepout polygon.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "keepout": {"type": "string"}, "vertices": {"type": ["array", "null"], "items": {"type": "string"}}, "layers": {"type": ["array", "null"], "items": {"type": "integer"}}, "kind": {"type": ["string", "null"]}}, "required": ["path", "keepout"]}},
    "datum.pcb.delete_keepout": {"description": "Delete one native-project board keepout polygon.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "keepout": {"type": "string"}}, "required": ["path", "keepout"]}},
    "datum.pcb.place_dimension": {"description": "Place one native-project board dimension.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"}, "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"}, "layer": {"type": "integer"}, "text": {"type": ["string", "null"]}}, "required": ["path", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "layer"]}},
    "datum.pcb.edit_dimension": {"description": "Edit one native-project board dimension.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "dimension": {"type": "string"}, "from_x_nm": {"type": ["integer", "null"]}, "from_y_nm": {"type": ["integer", "null"]}, "to_x_nm": {"type": ["integer", "null"]}, "to_y_nm": {"type": ["integer", "null"]}, "layer": {"type": ["integer", "null"]}, "text": {"type": ["string", "null"]}, "clear_text": {"type": ["boolean", "null"]}}, "required": ["path", "dimension"]}},
    "datum.pcb.delete_dimension": {"description": "Delete one native-project board dimension.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "dimension": {"type": "string"}}, "required": ["path", "dimension"]}},
    "datum.pcb.place_text": {"description": "Place one native-project board text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "text": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "layer": {"type": "integer"}, "rotation_deg": {"type": ["integer", "null"]}, "height_nm": {"type": ["integer", "null"]}, "stroke_width_nm": {"type": ["integer", "null"]}, "render_intent": {"type": ["string", "null"]}, "family": {"type": ["string", "null"]}, "style": {"type": ["string", "null"]}, "style_class": {"type": ["string", "null"]}, "h_align": {"type": ["string", "null"]}, "v_align": {"type": ["string", "null"]}, "mirrored": {"type": ["boolean", "null"]}, "keep_upright": {"type": ["boolean", "null"]}, "line_spacing_ratio_ppm": {"type": ["integer", "null"]}, "bold": {"type": ["boolean", "null"]}, "italic": {"type": ["boolean", "null"]}}, "required": ["path", "text", "x_nm", "y_nm", "layer"]}},
    "datum.pcb.edit_text": {"description": "Edit one native-project board text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "text": {"type": "string"}, "value": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "layer": {"type": ["integer", "null"]}, "rotation_deg": {"type": ["integer", "null"]}, "height_nm": {"type": ["integer", "null"]}, "stroke_width_nm": {"type": ["integer", "null"]}, "render_intent": {"type": ["string", "null"]}, "family": {"type": ["string", "null"]}, "style": {"type": ["string", "null"]}, "style_class": {"type": ["string", "null"]}, "h_align": {"type": ["string", "null"]}, "v_align": {"type": ["string", "null"]}, "mirrored": {"type": ["boolean", "null"]}, "keep_upright": {"type": ["boolean", "null"]}, "line_spacing_ratio_ppm": {"type": ["integer", "null"]}, "bold": {"type": ["boolean", "null"]}, "italic": {"type": ["boolean", "null"]}}, "required": ["path", "text"]}},
    "datum.pcb.delete_text": {"description": "Delete one native-project board text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "text": {"type": "string"}}, "required": ["path", "text"]}},
    "datum.pcb.place_net_class": {"description": "Place one native-project board net class.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "name": {"type": "string"}, "clearance_nm": {"type": "integer"}, "track_width_nm": {"type": "integer"}, "via_drill_nm": {"type": "integer"}, "via_diameter_nm": {"type": "integer"}, "diffpair_width_nm": {"type": ["integer", "null"]}, "diffpair_gap_nm": {"type": ["integer", "null"]}}, "required": ["path", "name", "clearance_nm", "track_width_nm", "via_drill_nm", "via_diameter_nm"]}},
    "datum.pcb.edit_net_class": {"description": "Edit one native-project board net class.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net_class": {"type": "string"}, "name": {"type": ["string", "null"]}, "clearance_nm": {"type": ["integer", "null"]}, "track_width_nm": {"type": ["integer", "null"]}, "via_drill_nm": {"type": ["integer", "null"]}, "via_diameter_nm": {"type": ["integer", "null"]}, "diffpair_width_nm": {"type": ["integer", "null"]}, "diffpair_gap_nm": {"type": ["integer", "null"]}}, "required": ["path", "net_class"]}},
    "datum.pcb.delete_net_class": {"description": "Delete one native-project board net class.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "net_class": {"type": "string"}}, "required": ["path", "net_class"]}},
}


def datum_alias(method: str, schema: dict[str, object]) -> dict[str, object]:
    aliased = dict(schema)
    aliased["x_dispatch_method"] = method
    return aliased


def proposal_write_alias(
    method: str,
    schema: dict[str, object],
    write_surface_class: str,
    evidence: str,
) -> dict[str, object]:
    aliased = datum_alias(method, schema)
    aliased["x_public_write_surface_class"] = write_surface_class
    aliased["x_write_surface_evidence"] = evidence
    return aliased


def gerber_set_proposal_alias() -> dict[str, object]:
    aliased = proposal_write_alias(
        "create_output_job_proposal",
        OUTPUT_JOB_TOOL_SCHEMAS["create_gerber_output_job"],
        "proposal_metadata_write",
        PROPOSAL_METADATA_EVIDENCE,
    )
    aliased["description"] = (
        "Create a draft proposal for the deterministic Gerber-set OutputJob without "
        "mutating the OutputJob shard."
    )
    aliased["x_dispatch_args"] = [
        "path",
        "prefix",
        "include",
        "name",
        "manufacturing_plan",
        "output_dir",
        "proposal",
        "rationale",
        "variant",
    ]
    aliased["x_dispatch_defaults"] = {"include": "gerber-set"}
    return aliased


PROPOSAL_METADATA_EVIDENCE = (
    "writes only persisted proposal metadata for later review; does not mutate design shards"
)
PROPOSAL_REVIEW_EVIDENCE = (
    "updates persisted proposal review state without applying design mutations"
)
PROPOSAL_APPLY_EVIDENCE = (
    "applies an accepted proposal through the generic proposal journal gateway"
)


DATUM_TOOL_SCHEMAS = {
    "datum.context.get": DATUM_CONTEXT_SCHEMA,
    "datum.context.refresh": DATUM_CONTEXT_SCHEMA,
    "datum.context.session_events": DATUM_CONTEXT_SESSION_EVENTS_SCHEMA,
    "datum.context.session_activity": DATUM_CONTEXT_SESSION_ACTIVITY_SCHEMA,
}

DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS = {
    "datum.schematic.create_sheet": {"description": "Create one native-project schematic sheet.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "name": {"type": "string"}, "sheet": {"type": ["string", "null"]}}, "required": ["path", "name"]}},
    "datum.schematic.delete_sheet": {"description": "Delete one native-project schematic sheet and its payload as one journaled operation.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}}, "required": ["path", "sheet"]}},
    "datum.schematic.rename_sheet": {"description": "Rename one native-project schematic sheet through the journaled substrate path.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "name": {"type": "string"}}, "required": ["path", "sheet", "name"]}},
    "datum.schematic.create_sheet_definition": {"description": "Create one native-project schematic sheet definition through the journaled substrate path.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "root_sheet": {"type": "string"}, "name": {"type": "string"}, "definition": {"type": ["string", "null"]}}, "required": ["path", "root_sheet", "name"]}},
    "datum.schematic.create_sheet_instance": {"description": "Create one native-project schematic sheet instance through the journaled substrate path.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "definition": {"type": "string"}, "name": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "parent_sheet": {"type": ["string", "null"]}, "instance": {"type": ["string", "null"]}}, "required": ["path", "definition", "name", "x_nm", "y_nm"]}},
    "datum.schematic.delete_sheet_instance": {"description": "Delete one native-project schematic sheet instance through the journaled substrate path.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "instance": {"type": "string"}}, "required": ["path", "instance"]}},
    "datum.schematic.move_sheet_instance": {"description": "Move one native-project schematic sheet instance through the journaled substrate path.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "instance": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}}, "required": ["path", "instance", "x_nm", "y_nm"]}},
    "datum.schematic.bind_sheet_instance_port": {"description": "Bind a parent-sheet hierarchical port to a sheet instance.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "instance": {"type": "string"}, "port": {"type": "string"}}, "required": ["path", "instance", "port"]}},
    "datum.schematic.unbind_sheet_instance_port": {"description": "Remove a parent-sheet hierarchical port binding from a sheet instance.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "instance": {"type": "string"}, "port": {"type": "string"}}, "required": ["path", "instance", "port"]}},
    "datum.schematic.draw_wire": {"description": "Draw one native-project schematic wire.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"}, "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"}}, "required": ["path", "sheet", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm"]}},
    "datum.schematic.delete_wire": {"description": "Delete one native-project schematic wire.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "wire": {"type": "string"}}, "required": ["path", "wire"]}},
    "datum.schematic.place_junction": {"description": "Place one native-project schematic junction.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}}, "required": ["path", "sheet", "x_nm", "y_nm"]}},
    "datum.schematic.delete_junction": {"description": "Delete one native-project schematic junction.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "junction": {"type": "string"}}, "required": ["path", "junction"]}},
    "datum.schematic.place_noconnect": {"description": "Place one native-project schematic no-connect marker.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "symbol": {"type": "string"}, "pin": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}}, "required": ["path", "sheet", "symbol", "pin", "x_nm", "y_nm"]}},
    "datum.schematic.delete_noconnect": {"description": "Delete one native-project schematic no-connect marker.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "noconnect": {"type": "string"}}, "required": ["path", "noconnect"]}},
    "datum.schematic.place_label": {"description": "Place one native-project schematic label.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "name": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "kind": {"type": ["string", "null"]}}, "required": ["path", "sheet", "name", "x_nm", "y_nm"]}},
    "datum.schematic.rename_label": {"description": "Rename one native-project schematic label.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "label": {"type": "string"}, "name": {"type": "string"}}, "required": ["path", "label", "name"]}},
    "datum.schematic.delete_label": {"description": "Delete one native-project schematic label.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "label": {"type": "string"}}, "required": ["path", "label"]}},
    "datum.schematic.place_port": {"description": "Place one native-project schematic port.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "name": {"type": "string"}, "direction": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}}, "required": ["path", "sheet", "name", "direction", "x_nm", "y_nm"]}},
    "datum.schematic.edit_port": {"description": "Edit one native-project schematic port.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "port": {"type": "string"}, "name": {"type": ["string", "null"]}, "direction": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}}, "required": ["path", "port"]}},
    "datum.schematic.delete_port": {"description": "Delete one native-project schematic port.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "port": {"type": "string"}}, "required": ["path", "port"]}},
    "datum.schematic.create_bus": {"description": "Create one native-project schematic bus.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "name": {"type": "string"}, "members": {"type": "array", "items": {"type": "string"}}}, "required": ["path", "sheet", "name", "members"]}},
    "datum.schematic.edit_bus_members": {"description": "Replace native-project schematic bus members.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "bus": {"type": "string"}, "members": {"type": "array", "items": {"type": "string"}}}, "required": ["path", "bus", "members"]}},
    "datum.schematic.delete_bus": {"description": "Delete one native-project schematic bus.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "bus": {"type": "string"}}, "required": ["path", "bus"]}},
    "datum.schematic.place_bus_entry": {"description": "Place one native-project schematic bus entry.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "bus": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "wire": {"type": ["string", "null"]}}, "required": ["path", "sheet", "bus", "x_nm", "y_nm"]}},
    "datum.schematic.delete_bus_entry": {"description": "Delete one native-project schematic bus entry.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "bus_entry": {"type": "string"}}, "required": ["path", "bus_entry"]}},
    "datum.schematic.place_text": {"description": "Place one native-project schematic text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "text": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "rotation_deg": {"type": ["integer", "null"]}}, "required": ["path", "sheet", "text", "x_nm", "y_nm"]}},
    "datum.schematic.edit_text": {"description": "Edit one native-project schematic text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "text": {"type": "string"}, "value": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "rotation_deg": {"type": ["integer", "null"]}}, "required": ["path", "text"]}},
    "datum.schematic.delete_text": {"description": "Delete one native-project schematic text object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "text": {"type": "string"}}, "required": ["path", "text"]}},
    "datum.schematic.place_drawing_line": {"description": "Place one schematic drawing line.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "from_x_nm": {"type": "integer"}, "from_y_nm": {"type": "integer"}, "to_x_nm": {"type": "integer"}, "to_y_nm": {"type": "integer"}}, "required": ["path", "sheet", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm"]}},
    "datum.schematic.place_drawing_rect": {"description": "Place one schematic drawing rectangle.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "min_x_nm": {"type": "integer"}, "min_y_nm": {"type": "integer"}, "max_x_nm": {"type": "integer"}, "max_y_nm": {"type": "integer"}}, "required": ["path", "sheet", "min_x_nm", "min_y_nm", "max_x_nm", "max_y_nm"]}},
    "datum.schematic.place_drawing_circle": {"description": "Place one schematic drawing circle.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "center_x_nm": {"type": "integer"}, "center_y_nm": {"type": "integer"}, "radius_nm": {"type": "integer"}}, "required": ["path", "sheet", "center_x_nm", "center_y_nm", "radius_nm"]}},
    "datum.schematic.place_drawing_arc": {"description": "Place one schematic drawing arc.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "center_x_nm": {"type": "integer"}, "center_y_nm": {"type": "integer"}, "radius_nm": {"type": "integer"}, "start_angle_mdeg": {"type": "integer"}, "end_angle_mdeg": {"type": "integer"}}, "required": ["path", "sheet", "center_x_nm", "center_y_nm", "radius_nm", "start_angle_mdeg", "end_angle_mdeg"]}},
    "datum.schematic.edit_drawing_line": {"description": "Edit one schematic drawing line.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "drawing": {"type": "string"}, "from_x_nm": {"type": ["integer", "null"]}, "from_y_nm": {"type": ["integer", "null"]}, "to_x_nm": {"type": ["integer", "null"]}, "to_y_nm": {"type": ["integer", "null"]}}, "required": ["path", "drawing"]}},
    "datum.schematic.edit_drawing_rect": {"description": "Edit one schematic drawing rectangle.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "drawing": {"type": "string"}, "min_x_nm": {"type": ["integer", "null"]}, "min_y_nm": {"type": ["integer", "null"]}, "max_x_nm": {"type": ["integer", "null"]}, "max_y_nm": {"type": ["integer", "null"]}}, "required": ["path", "drawing"]}},
    "datum.schematic.edit_drawing_circle": {"description": "Edit one schematic drawing circle.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "drawing": {"type": "string"}, "center_x_nm": {"type": ["integer", "null"]}, "center_y_nm": {"type": ["integer", "null"]}, "radius_nm": {"type": ["integer", "null"]}}, "required": ["path", "drawing"]}},
    "datum.schematic.edit_drawing_arc": {"description": "Edit one schematic drawing arc.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "drawing": {"type": "string"}, "center_x_nm": {"type": ["integer", "null"]}, "center_y_nm": {"type": ["integer", "null"]}, "radius_nm": {"type": ["integer", "null"]}, "start_angle_mdeg": {"type": ["integer", "null"]}, "end_angle_mdeg": {"type": ["integer", "null"]}}, "required": ["path", "drawing"]}},
    "datum.schematic.delete_drawing": {"description": "Delete one schematic drawing object.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "drawing": {"type": "string"}}, "required": ["path", "drawing"]}},
    "datum.schematic.place_symbol": {"description": "Place one native-project schematic symbol.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "sheet": {"type": "string"}, "reference": {"type": "string"}, "value": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "lib_id": {"type": ["string", "null"]}, "rotation_deg": {"type": ["integer", "null"]}, "mirrored": {"type": ["boolean", "null"]}}, "required": ["path", "sheet", "reference", "value", "x_nm", "y_nm"]}},
    "datum.schematic.move_symbol": {"description": "Move one native-project schematic symbol.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}}, "required": ["path", "symbol", "x_nm", "y_nm"]}},
    "datum.schematic.rotate_symbol": {"description": "Rotate one native-project schematic symbol.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}, "rotation_deg": {"type": "integer"}}, "required": ["path", "symbol", "rotation_deg"]}},
    "datum.schematic.mirror_symbol": {"description": "Mirror one native-project schematic symbol.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}}, "required": ["path", "symbol"]}},
    "datum.schematic.delete_symbol": {"description": "Delete one native-project schematic symbol.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}}, "required": ["path", "symbol"]}},
    "datum.schematic.set_symbol_reference": {"description": "Set one native-project schematic symbol reference.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}, "reference": {"type": "string"}}, "required": ["path", "symbol", "reference"]}},
    "datum.schematic.set_symbol_value": {"description": "Set one native-project schematic symbol value.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "symbol": {"type": "string"}, "value": {"type": "string"}}, "required": ["path", "symbol", "value"]}},
}

DATUM_SCHEMATIC_SYMBOL_METADATA_METHODS = "set_symbol_display_mode set_symbol_hidden_power_behavior set_symbol_unit clear_symbol_unit set_symbol_gate clear_symbol_gate set_symbol_entity clear_symbol_entity set_symbol_part clear_symbol_part set_symbol_lib_id clear_symbol_lib_id set_pin_override clear_pin_override add_symbol_field edit_symbol_field delete_symbol_field".split()

def _symbol_schema(description: str, properties: dict, required: list[str]) -> dict:
    return {"description": description, "inputSchema": {"type": "object", "properties": properties, "required": required}}

_STR = {"type": "string"}; _BOOL_NULL = {"type": ["boolean", "null"]}; _INT_NULL = {"type": ["integer", "null"]}; _STR_NULL = {"type": ["string", "null"]}
DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS.update({
    "datum.schematic.set_symbol_display_mode": _symbol_schema("Set one native-project schematic symbol display mode.", {"path": _STR, "symbol": _STR, "mode": _STR}, ["path", "symbol", "mode"]),
    "datum.schematic.set_symbol_hidden_power_behavior": _symbol_schema("Set one native-project schematic symbol hidden-power behavior.", {"path": _STR, "symbol": _STR, "behavior": _STR}, ["path", "symbol", "behavior"]),
    "datum.schematic.set_symbol_unit": _symbol_schema("Set one native-project schematic symbol unit selection.", {"path": _STR, "symbol": _STR, "unit": _STR}, ["path", "symbol", "unit"]),
    "datum.schematic.clear_symbol_unit": _symbol_schema("Clear one native-project schematic symbol unit selection.", {"path": _STR, "symbol": _STR}, ["path", "symbol"]),
    "datum.schematic.set_symbol_gate": _symbol_schema("Set one native-project schematic symbol gate UUID.", {"path": _STR, "symbol": _STR, "gate": _STR}, ["path", "symbol", "gate"]),
    "datum.schematic.clear_symbol_gate": _symbol_schema("Clear one native-project schematic symbol gate UUID.", {"path": _STR, "symbol": _STR}, ["path", "symbol"]),
    "datum.schematic.set_symbol_entity": _symbol_schema("Set one native-project schematic symbol entity UUID.", {"path": _STR, "symbol": _STR, "entity": _STR}, ["path", "symbol", "entity"]),
    "datum.schematic.clear_symbol_entity": _symbol_schema("Clear one native-project schematic symbol entity UUID.", {"path": _STR, "symbol": _STR}, ["path", "symbol"]),
    "datum.schematic.set_symbol_part": _symbol_schema("Set one native-project schematic symbol part UUID.", {"path": _STR, "symbol": _STR, "part": _STR}, ["path", "symbol", "part"]),
    "datum.schematic.clear_symbol_part": _symbol_schema("Clear one native-project schematic symbol part UUID.", {"path": _STR, "symbol": _STR}, ["path", "symbol"]),
    "datum.schematic.set_symbol_lib_id": _symbol_schema("Set one native-project schematic symbol library identifier.", {"path": _STR, "symbol": _STR, "lib_id": _STR}, ["path", "symbol", "lib_id"]),
    "datum.schematic.clear_symbol_lib_id": _symbol_schema("Clear one native-project schematic symbol library identifier.", {"path": _STR, "symbol": _STR}, ["path", "symbol"]),
    "datum.schematic.set_pin_override": _symbol_schema("Set one native-project schematic symbol pin display override.", {"path": _STR, "symbol": _STR, "pin": _STR, "visible": {"type": "boolean"}, "x_nm": _INT_NULL, "y_nm": _INT_NULL}, ["path", "symbol", "pin", "visible"]),
    "datum.schematic.clear_pin_override": _symbol_schema("Clear one native-project schematic symbol pin display override.", {"path": _STR, "symbol": _STR, "pin": _STR}, ["path", "symbol", "pin"]),
    "datum.schematic.add_symbol_field": _symbol_schema("Add one native-project schematic symbol field.", {"path": _STR, "symbol": _STR, "key": _STR, "value": _STR, "hidden": _BOOL_NULL, "x_nm": _INT_NULL, "y_nm": _INT_NULL}, ["path", "symbol", "key", "value"]),
    "datum.schematic.edit_symbol_field": _symbol_schema("Edit one native-project schematic symbol field.", {"path": _STR, "field": _STR, "key": _STR_NULL, "value": _STR_NULL, "visible": _BOOL_NULL, "x_nm": _INT_NULL, "y_nm": _INT_NULL}, ["path", "field"]),
    "datum.schematic.delete_symbol_field": _symbol_schema("Delete one native-project schematic symbol field.", {"path": _STR, "field": _STR}, ["path", "field"]),
})

DATUM_TOOL_SPECS = [
    {"name": "datum.context.get", **datum_alias("datum_context_get", DATUM_TOOL_SCHEMAS["datum.context.get"])},
    {"name": "datum.context.refresh", **datum_alias("datum_context_refresh", DATUM_TOOL_SCHEMAS["datum.context.refresh"])},
    {"name": "datum.context.session_events", **datum_alias("datum_context_session_events", DATUM_TOOL_SCHEMAS["datum.context.session_events"])},
    {"name": "datum.context.session_activity", **datum_alias("datum_context_session_activity", DATUM_TOOL_SCHEMAS["datum.context.session_activity"])},
    {"name": "datum.check.run", **datum_alias("get_check_run", CHECK_TOOL_SCHEMAS["get_check_run"])},
    {"name": "datum.check.run_profile", **datum_alias("get_check_run", CHECK_TOOL_SCHEMAS["get_check_run"])},
    {"name": "datum.check.list", **datum_alias("get_check_runs", CHECK_TOOL_SCHEMAS["get_check_runs"])},
    {"name": "datum.check.show", **datum_alias("show_check_run", CHECK_TOOL_SCHEMAS["show_check_run"])},
    {"name": "datum.check.profiles", **datum_alias("get_check_profiles", CHECK_TOOL_SCHEMAS["get_check_profiles"])},
    {"name": "datum.check.fill_zones", **datum_alias("fill_zones", CHECK_TOOL_SCHEMAS["fill_zones"])},
    {"name": "datum.check.repair_standards", **datum_alias("generate_standards_repair_proposals", CHECK_TOOL_SCHEMAS["generate_standards_repair_proposals"])},
    {"name": "datum.check.waive", **datum_alias("waive_finding", CHECK_TOOL_SCHEMAS["waive_finding"])},
    {"name": "datum.check.accept_deviation", **datum_alias("accept_deviation", CHECK_TOOL_SCHEMAS["accept_deviation"])},
    {"name": "datum.check.explain_violation", **datum_alias("explain_violation", CHECK_TOOL_SCHEMAS["explain_violation"])},
    {"name": "datum.query.board_summary", **datum_alias("get_board_summary", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.components", **datum_alias("get_components", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.netlist", **datum_alias("get_netlist", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_summary", **datum_alias("get_schematic_summary", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_wires", **datum_alias("get_schematic_wires", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_junctions", **datum_alias("get_schematic_junctions", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_tracks", **datum_alias("get_board_tracks", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_vias", **datum_alias("get_board_vias", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_pads", **datum_alias("get_board_pads", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_zones", **datum_alias("get_board_zones", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_texts", **datum_alias("get_board_texts", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_keepouts", **datum_alias("get_board_keepouts", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_outline", **datum_alias("get_board_outline", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_stackup", **datum_alias("get_board_stackup", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_dimensions", **datum_alias("get_board_dimensions", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_nets", **datum_alias("get_board_nets", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.board_net_classes", **datum_alias("get_board_net_classes", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.sheets", **datum_alias("get_sheets", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.symbols", **datum_alias("get_symbols", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.symbol_fields", **datum_alias("get_symbol_fields", DATUM_SYMBOL_UUID_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_labels", **datum_alias("get_schematic_labels", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_ports", **datum_alias("get_schematic_ports", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_noconnects", **datum_alias("get_schematic_noconnects", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_buses", **datum_alias("get_schematic_buses", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_bus_entries", **datum_alias("get_schematic_bus_entries", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_texts", **datum_alias("get_schematic_texts", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_drawings", **datum_alias("get_schematic_drawings", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.labels", **datum_alias("get_labels", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.ports", **datum_alias("get_ports", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.buses", **datum_alias("get_buses", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.bus_entries", **datum_alias("get_bus_entries", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.noconnects", **datum_alias("get_noconnects", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.hierarchy", **datum_alias("get_project_hierarchy", DATUM_HIERARCHY_QUERY_SCHEMA)},
    {"name": "datum.query.schematic_nets", **datum_alias("get_schematic_net_info", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.connectivity_diagnostics", **datum_alias("get_connectivity_diagnostics", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.design_rules", **datum_alias("get_design_rules", DATUM_EMPTY_QUERY_SCHEMA)},
    {"name": "datum.query.source_shards", **datum_alias("get_source_shards", DATUM_PATH_QUERY_SCHEMA)},
    {"name": "datum.query.zone_fills", **datum_alias("get_zone_fills", CHECK_TOOL_SCHEMAS["get_zone_fills"])},
    {"name": "datum.query.component_instances", **datum_alias("get_component_instances", RELATIONSHIP_TOOL_SCHEMAS["get_component_instances"])},
    {"name": "datum.query.relationships", **datum_alias("get_relationships", RELATIONSHIP_TOOL_SCHEMAS["get_relationships"])},
    {"name": "datum.query.variants", **datum_alias("get_variants", RELATIONSHIP_TOOL_SCHEMAS["get_variants"])},
    {"name": "datum.query.import_map", **datum_alias("get_import_map", IMPORT_MAP_TOOL_SCHEMAS["get_import_map"])},
    {"name": "datum.query.panel_projections", **datum_alias("get_panel_projections", OUTPUT_JOB_TOOL_SCHEMAS["get_panel_projections"])},
    {"name": "datum.query.manufacturing_plans", **datum_alias("get_manufacturing_plans", OUTPUT_JOB_TOOL_SCHEMAS["get_manufacturing_plans"])},
    {"name": "datum.query.output_jobs", **datum_alias("get_output_jobs", OUTPUT_JOB_TOOL_SCHEMAS["get_output_jobs"])},
    {"name": "datum.pcb.place_component", **datum_alias("place_board_component", DATUM_PLACE_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.move_component", **datum_alias("move_board_component", DATUM_MOVE_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.rotate_component", **datum_alias("rotate_board_component", DATUM_ROTATE_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.flip_component", **datum_alias("flip_board_component", DATUM_FLIP_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.delete_component", **datum_alias("delete_board_component", DATUM_DELETE_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.set_component_reference", **datum_alias("set_board_component_reference", DATUM_COMPONENT_PROPERTY_SCHEMAS["datum.pcb.set_component_reference"])},
    {"name": "datum.pcb.set_component_value", **datum_alias("set_board_component_value", DATUM_COMPONENT_PROPERTY_SCHEMAS["datum.pcb.set_component_value"])},
    {"name": "datum.pcb.set_component_part", **datum_alias("set_board_component_part", DATUM_COMPONENT_PROPERTY_SCHEMAS["datum.pcb.set_component_part"])},
    {"name": "datum.pcb.set_component_package", **datum_alias("set_board_component_package", DATUM_COMPONENT_PROPERTY_SCHEMAS["datum.pcb.set_component_package"])},
    {"name": "datum.pcb.lock_component", **datum_alias("lock_board_component", DATUM_LOCK_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.unlock_component", **datum_alias("unlock_board_component", DATUM_LOCK_COMPONENT_SCHEMA)},
    {"name": "datum.pcb.draw_track", **datum_alias("draw_board_track", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.draw_track"])},
    {"name": "datum.pcb.edit_track", **datum_alias("edit_board_track", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_track"])},
    {"name": "datum.pcb.delete_track", **datum_alias("delete_board_track", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_track"])},
    {"name": "datum.pcb.place_via", **datum_alias("place_board_via", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_via"])},
    {"name": "datum.pcb.edit_via", **datum_alias("edit_board_via", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_via"])},
    {"name": "datum.pcb.delete_via", **datum_alias("delete_board_via", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_via"])},
    {"name": "datum.pcb.place_zone", **datum_alias("place_board_zone", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_zone"]), "x_dispatch_args": ["path", "net", "vertices", "layer", "thermal_gap_nm", "thermal_spoke_width_nm", "priority", "thermal_relief"]},
    {"name": "datum.pcb.edit_zone", **datum_alias("edit_board_zone", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_zone"]), "x_dispatch_args": ["path", "zone", "net", "vertices", "layer", "priority", "thermal_relief", "thermal_gap_nm", "thermal_spoke_width_nm"]},
    {"name": "datum.pcb.delete_zone", **datum_alias("delete_board_zone", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_zone"])},
    {"name": "datum.pcb.place_pad", **datum_alias("place_board_pad", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_pad"])},
    {"name": "datum.pcb.edit_pad", **datum_alias("edit_board_pad", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_pad"])},
    {"name": "datum.pcb.delete_pad", **datum_alias("delete_board_pad", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_pad"])},
    {"name": "datum.pcb.set_pad_net", **datum_alias("set_board_pad_net", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.set_pad_net"])},
    {"name": "datum.pcb.clear_pad_net", **datum_alias("clear_board_pad_net", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.clear_pad_net"])},
    {"name": "datum.pcb.place_net", **datum_alias("place_board_net", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_net"])},
    {"name": "datum.pcb.edit_net", **datum_alias("edit_board_net", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_net"]), "x_dispatch_args": ["path", "net", "name", "class", "impedance_target_ohms", "impedance_tolerance_pct", "controlled_dielectric_layer", "clear_controlled_impedance"]},
    {"name": "datum.pcb.delete_net", **datum_alias("delete_board_net", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_net"])},
    {"name": "datum.pcb.set_board_name", **datum_alias("set_board_name", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.set_board_name"])},
    {"name": "datum.pcb.set_outline", **datum_alias("set_board_outline", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.set_outline"])},
    {"name": "datum.pcb.set_stackup", **datum_alias("set_board_stackup", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.set_stackup"]), "x_dispatch_args": ["path", "layers"]},
    {"name": "datum.pcb.add_default_top_stackup", **datum_alias("add_default_top_stackup", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.add_default_top_stackup"])},
    {"name": "datum.pcb.place_keepout", **datum_alias("place_board_keepout", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_keepout"]), "x_dispatch_args": ["path", "vertices", "layers", "kind"]},
    {"name": "datum.pcb.edit_keepout", **datum_alias("edit_board_keepout", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_keepout"]), "x_dispatch_args": ["path", "keepout", "vertices", "layers", "kind"]},
    {"name": "datum.pcb.delete_keepout", **datum_alias("delete_board_keepout", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_keepout"])},
    {"name": "datum.pcb.place_dimension", **datum_alias("place_board_dimension", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_dimension"])},
    {"name": "datum.pcb.edit_dimension", **datum_alias("edit_board_dimension", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_dimension"]), "x_dispatch_args": ["path", "dimension", "from_x_nm", "from_y_nm", "to_x_nm", "to_y_nm", "layer", "text", "clear_text"]},
    {"name": "datum.pcb.delete_dimension", **datum_alias("delete_board_dimension", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_dimension"])},
    {"name": "datum.pcb.place_text", **datum_alias("place_board_text", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_text"]), "x_dispatch_args": ["path", "text", "x_nm", "y_nm", "layer", "rotation_deg", "height_nm", "stroke_width_nm", "render_intent", "family", "style", "style_class", "h_align", "v_align", "mirrored", "keep_upright", "line_spacing_ratio_ppm", "bold", "italic"]},
    {"name": "datum.pcb.edit_text", **datum_alias("edit_board_text", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_text"]), "x_dispatch_args": ["path", "text", "value", "x_nm", "y_nm", "layer", "rotation_deg", "height_nm", "stroke_width_nm", "render_intent", "family", "style", "style_class", "h_align", "v_align", "mirrored", "keep_upright", "line_spacing_ratio_ppm", "bold", "italic"]},
    {"name": "datum.pcb.delete_text", **datum_alias("delete_board_text", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_text"])},
    {"name": "datum.pcb.place_net_class", **datum_alias("place_board_net_class", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.place_net_class"])},
    {"name": "datum.pcb.edit_net_class", **datum_alias("edit_board_net_class", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.edit_net_class"]), "x_dispatch_args": ["path", "net_class", "name", "clearance_nm", "track_width_nm", "via_drill_nm", "via_diameter_nm", "diffpair_width_nm", "diffpair_gap_nm"]},
    {"name": "datum.pcb.delete_net_class", **datum_alias("delete_board_net_class", DATUM_PCB_PRIMITIVE_SCHEMAS["datum.pcb.delete_net_class"])},
    {"name": "datum.component_instance.bind", **datum_alias("bind_component_instance", RELATIONSHIP_TOOL_SCHEMAS["bind_component_instance"])},
    {"name": "datum.component_instance.set", **datum_alias("set_component_instance", RELATIONSHIP_TOOL_SCHEMAS["set_component_instance"])},
    {"name": "datum.component_instance.delete", **datum_alias("delete_component_instance", RELATIONSHIP_TOOL_SCHEMAS["delete_component_instance"])},
    {"name": "datum.library.list_objects", **datum_alias("get_pool_library_objects", LIBRARY_TOOL_SCHEMAS["get_pool_library_objects"])},
    {"name": "datum.library.show_object", **datum_alias("show_pool_library_object", LIBRARY_TOOL_SCHEMAS["show_pool_library_object"])},
    {"name": "datum.library.pool_models", **datum_alias("get_pool_model_blobs", LIBRARY_TOOL_SCHEMAS["get_pool_model_blobs"])},
    {"name": "datum.library.gc_pool_models", **datum_alias("gc_pool_model_blobs", LIBRARY_TOOL_SCHEMAS["gc_pool_model_blobs"])},
    {"name": "datum.library.create_object", **datum_alias("create_pool_library_object", LIBRARY_TOOL_SCHEMAS["create_pool_library_object"])},
    {"name": "datum.library.create_unit", **datum_alias("create_pool_unit", LIBRARY_TOOL_SCHEMAS["create_pool_unit"])},
    {"name": "datum.library.set_unit_pin", **datum_alias("set_pool_unit_pin", LIBRARY_TOOL_SCHEMAS["set_pool_unit_pin"])},
    {"name": "datum.library.create_symbol", **datum_alias("create_pool_symbol", LIBRARY_TOOL_SCHEMAS["create_pool_symbol"])},
    {"name": "datum.library.add_symbol_line", **datum_alias("add_pool_symbol_line", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_line"])},
    {"name": "datum.library.add_symbol_rect", **datum_alias("add_pool_symbol_rect", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_rect"])},
    {"name": "datum.library.add_symbol_circle", **datum_alias("add_pool_symbol_circle", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_circle"])},
    {"name": "datum.library.add_symbol_arc", **datum_alias("add_pool_symbol_arc", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_arc"])},
    {"name": "datum.library.add_symbol_polygon", **datum_alias("add_pool_symbol_polygon", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_polygon"])},
    {"name": "datum.library.add_symbol_text", **datum_alias("add_pool_symbol_text", LIBRARY_TOOL_SCHEMAS["add_pool_symbol_text"])},
    {"name": "datum.library.set_symbol_pin_anchor", **datum_alias("set_pool_symbol_pin_anchor", LIBRARY_TOOL_SCHEMAS["set_pool_symbol_pin_anchor"])},
    {"name": "datum.library.create_entity", **datum_alias("create_pool_entity", LIBRARY_TOOL_SCHEMAS["create_pool_entity"])},
    {"name": "datum.library.create_padstack", **datum_alias("create_pool_padstack", LIBRARY_TOOL_SCHEMAS["create_pool_padstack"])},
    {"name": "datum.library.create_package", **datum_alias("create_pool_package", LIBRARY_TOOL_SCHEMAS["create_pool_package"])},
    {"name": "datum.library.set_package_pad", **datum_alias("set_pool_package_pad", LIBRARY_TOOL_SCHEMAS["set_pool_package_pad"])},
    {"name": "datum.library.set_package_courtyard_rect", **datum_alias("set_pool_package_courtyard_rect", LIBRARY_TOOL_SCHEMAS["set_pool_package_courtyard_rect"])},
    {"name": "datum.library.set_package_courtyard_polygon", **datum_alias("set_pool_package_courtyard_polygon", LIBRARY_TOOL_SCHEMAS["set_pool_package_courtyard_polygon"])},
    {"name": "datum.library.add_package_silkscreen_line", **datum_alias("add_pool_package_silkscreen_line", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_line"])},
    {"name": "datum.library.add_package_silkscreen_rect", **datum_alias("add_pool_package_silkscreen_rect", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_rect"])},
    {"name": "datum.library.add_package_silkscreen_polygon", **datum_alias("add_pool_package_silkscreen_polygon", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_polygon"])},
    {"name": "datum.library.add_package_silkscreen_circle", **datum_alias("add_pool_package_silkscreen_circle", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_circle"])},
    {"name": "datum.library.add_package_silkscreen_arc", **datum_alias("add_pool_package_silkscreen_arc", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_arc"])},
    {"name": "datum.library.add_package_silkscreen_text", **datum_alias("add_pool_package_silkscreen_text", LIBRARY_TOOL_SCHEMAS["add_pool_package_silkscreen_text"])},
    {"name": "datum.library.add_package_model_3d", **datum_alias("add_pool_package_model_3d", LIBRARY_TOOL_SCHEMAS["add_pool_package_model_3d"])},
    {"name": "datum.library.set_package_body_heights", **datum_alias("set_pool_package_body_heights", LIBRARY_TOOL_SCHEMAS["set_pool_package_body_heights"])},
    {"name": "datum.library.create_part", **datum_alias("create_pool_part", LIBRARY_TOOL_SCHEMAS["create_pool_part"])},
    {"name": "datum.library.set_part_metadata", **datum_alias("set_pool_part_metadata", LIBRARY_TOOL_SCHEMAS["set_pool_part_metadata"])},
    {"name": "datum.library.set_part_parametric", **datum_alias("set_pool_part_parametric", LIBRARY_TOOL_SCHEMAS["set_pool_part_parametric"])},
    {"name": "datum.library.set_part_orderable_mpns", **datum_alias("set_pool_part_orderable_mpns", LIBRARY_TOOL_SCHEMAS["set_pool_part_orderable_mpns"])},
    {"name": "datum.library.set_part_tags", **datum_alias("set_pool_part_tags", LIBRARY_TOOL_SCHEMAS["set_pool_part_tags"])},
    {"name": "datum.library.set_part_packaging_options", **datum_alias("set_pool_part_packaging_options", LIBRARY_TOOL_SCHEMAS["set_pool_part_packaging_options"])},
    {"name": "datum.library.set_part_supply_chain", **datum_alias("set_pool_part_supply_chain", LIBRARY_TOOL_SCHEMAS["set_pool_part_supply_chain"])},
    {"name": "datum.library.set_part_behavioural_models", **datum_alias("set_pool_part_behavioural_models", LIBRARY_TOOL_SCHEMAS["set_pool_part_behavioural_models"])},
    {"name": "datum.library.attach_part_model", **datum_alias("attach_pool_part_model", LIBRARY_TOOL_SCHEMAS["attach_pool_part_model"])},
    {"name": "datum.library.detach_part_model", **datum_alias("detach_pool_part_model", LIBRARY_TOOL_SCHEMAS["detach_pool_part_model"])},
    {"name": "datum.library.set_part_thermal", **datum_alias("set_pool_part_thermal", LIBRARY_TOOL_SCHEMAS["set_pool_part_thermal"])},
    {"name": "datum.library.set_part_pad_map_entry", **datum_alias("set_pool_part_pad_map_entry", LIBRARY_TOOL_SCHEMAS["set_pool_part_pad_map_entry"])},
    {"name": "datum.library.set_part_pad_map", **datum_alias("set_pool_part_pad_map", LIBRARY_TOOL_SCHEMAS["set_pool_part_pad_map"])},
    {"name": "datum.library.set_object", **datum_alias("set_pool_library_object", LIBRARY_TOOL_SCHEMAS["set_pool_library_object"])},
    {"name": "datum.library.delete_object", **datum_alias("delete_pool_library_object", LIBRARY_TOOL_SCHEMAS["delete_pool_library_object"])},
    {"name": "datum.schematic.create_sheet", **datum_alias("create_sheet", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.create_sheet"])},
    {"name": "datum.schematic.delete_sheet", **datum_alias("delete_sheet", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_sheet"])},
    {"name": "datum.schematic.rename_sheet", **datum_alias("rename_sheet", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.rename_sheet"])},
    {"name": "datum.schematic.create_sheet_definition", **datum_alias("create_sheet_definition", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.create_sheet_definition"])},
    {"name": "datum.schematic.create_sheet_instance", **datum_alias("create_sheet_instance", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.create_sheet_instance"])},
    {"name": "datum.schematic.delete_sheet_instance", **datum_alias("delete_sheet_instance", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_sheet_instance"])},
    {"name": "datum.schematic.move_sheet_instance", **datum_alias("move_sheet_instance", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.move_sheet_instance"])},
    {"name": "datum.schematic.bind_sheet_instance_port", **datum_alias("bind_sheet_instance_port", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.bind_sheet_instance_port"])},
    {"name": "datum.schematic.unbind_sheet_instance_port", **datum_alias("unbind_sheet_instance_port", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.unbind_sheet_instance_port"])},
    {"name": "datum.schematic.draw_wire", **datum_alias("draw_wire", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.draw_wire"])},
    {"name": "datum.schematic.delete_wire", **datum_alias("delete_wire", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_wire"])},
    {"name": "datum.schematic.place_junction", **datum_alias("place_junction", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_junction"])},
    {"name": "datum.schematic.delete_junction", **datum_alias("delete_junction", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_junction"])},
    {"name": "datum.schematic.place_noconnect", **datum_alias("place_noconnect", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_noconnect"])},
    {"name": "datum.schematic.delete_noconnect", **datum_alias("delete_noconnect", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_noconnect"])},
    {"name": "datum.schematic.place_label", **datum_alias("place_label", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_label"])},
    {"name": "datum.schematic.rename_label", **datum_alias("rename_label", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.rename_label"])},
    {"name": "datum.schematic.delete_label", **datum_alias("delete_label", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_label"])},
    {"name": "datum.schematic.place_port", **datum_alias("place_port", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_port"])},
    {"name": "datum.schematic.edit_port", **datum_alias("edit_port", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_port"])},
    {"name": "datum.schematic.delete_port", **datum_alias("delete_port", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_port"])},
    {"name": "datum.schematic.create_bus", **datum_alias("create_bus", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.create_bus"])},
    {"name": "datum.schematic.edit_bus_members", **datum_alias("edit_bus_members", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_bus_members"])},
    {"name": "datum.schematic.delete_bus", **datum_alias("delete_bus", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_bus"])},
    {"name": "datum.schematic.place_bus_entry", **datum_alias("place_bus_entry", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_bus_entry"])},
    {"name": "datum.schematic.delete_bus_entry", **datum_alias("delete_bus_entry", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_bus_entry"])},
    {"name": "datum.schematic.place_text", **datum_alias("place_schematic_text", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_text"])},
    {"name": "datum.schematic.edit_text", **datum_alias("edit_schematic_text", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_text"])},
    {"name": "datum.schematic.delete_text", **datum_alias("delete_schematic_text", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_text"])},
    {"name": "datum.schematic.place_drawing_line", **datum_alias("place_drawing_line", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_drawing_line"])},
    {"name": "datum.schematic.place_drawing_rect", **datum_alias("place_drawing_rect", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_drawing_rect"])},
    {"name": "datum.schematic.place_drawing_circle", **datum_alias("place_drawing_circle", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_drawing_circle"])},
    {"name": "datum.schematic.place_drawing_arc", **datum_alias("place_drawing_arc", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_drawing_arc"])},
    {"name": "datum.schematic.edit_drawing_line", **datum_alias("edit_drawing_line", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_drawing_line"])},
    {"name": "datum.schematic.edit_drawing_rect", **datum_alias("edit_drawing_rect", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_drawing_rect"])},
    {"name": "datum.schematic.edit_drawing_circle", **datum_alias("edit_drawing_circle", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_drawing_circle"])},
    {"name": "datum.schematic.edit_drawing_arc", **datum_alias("edit_drawing_arc", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.edit_drawing_arc"])},
    {"name": "datum.schematic.delete_drawing", **datum_alias("delete_drawing", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_drawing"])},
    {"name": "datum.schematic.place_symbol", **datum_alias("place_symbol", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.place_symbol"])},
    {"name": "datum.schematic.move_symbol", **datum_alias("move_symbol", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.move_symbol"])},
    {"name": "datum.schematic.rotate_symbol", **datum_alias("rotate_symbol", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.rotate_symbol"])},
    {"name": "datum.schematic.mirror_symbol", **datum_alias("mirror_symbol", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.mirror_symbol"])},
    {"name": "datum.schematic.delete_symbol", **datum_alias("delete_symbol", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.delete_symbol"])},
    {"name": "datum.schematic.set_symbol_reference", **datum_alias("set_symbol_reference", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.set_symbol_reference"])},
    {"name": "datum.schematic.set_symbol_value", **datum_alias("set_symbol_value", DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS["datum.schematic.set_symbol_value"])},
    *[{"name": f"datum.schematic.{name}", **datum_alias(name, DATUM_SCHEMATIC_PRIMITIVE_SCHEMAS[f"datum.schematic.{name}"])} for name in DATUM_SCHEMATIC_SYMBOL_METADATA_METHODS],
    {"name": "datum.proposal.create", **proposal_write_alias("create_proposal", PROPOSAL_TOOL_SCHEMAS["create_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_draw_wire", **proposal_write_alias("create_draw_wire_proposal", PROPOSAL_TOOL_SCHEMAS["create_draw_wire_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_place_label", **proposal_write_alias("create_place_label_proposal", PROPOSAL_TOOL_SCHEMAS["create_place_label_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_place_symbol", **proposal_write_alias("create_place_symbol_proposal", PROPOSAL_TOOL_SCHEMAS["create_place_symbol_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_board_component_replacement", **proposal_write_alias("create_board_component_replacement_proposal", PROPOSAL_TOOL_SCHEMAS["create_board_component_replacement_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_board_component_replacements", **proposal_write_alias("create_board_component_replacements_proposal", PROPOSAL_TOOL_SCHEMAS["create_board_component_replacements_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_board_component_replacement_plan", **proposal_write_alias("create_board_component_replacement_plan_proposal", PROPOSAL_TOOL_SCHEMAS["create_board_component_replacement_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_library_object", **proposal_write_alias("create_pool_library_object_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_library_object_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_unit", **proposal_write_alias("create_pool_unit_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_unit_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_symbol", **proposal_write_alias("create_pool_symbol_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_symbol_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_entity", **proposal_write_alias("create_pool_entity_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_entity_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_padstack", **proposal_write_alias("create_pool_padstack_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_padstack_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_pool_package", **proposal_write_alias("create_pool_package_proposal", PROPOSAL_TOOL_SCHEMAS["create_pool_package_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.set_pool_package_pad", **proposal_write_alias("set_pool_package_pad_proposal", PROPOSAL_TOOL_SCHEMAS["set_pool_package_pad_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.set_pool_package_courtyard_rect", **proposal_write_alias("set_pool_package_courtyard_rect_proposal", PROPOSAL_TOOL_SCHEMAS["set_pool_package_courtyard_rect_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.set_pool_package_courtyard_polygon", **proposal_write_alias("set_pool_package_courtyard_polygon_proposal", PROPOSAL_TOOL_SCHEMAS["set_pool_package_courtyard_polygon_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_panel_projection", **proposal_write_alias("create_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.update_panel_projection", **proposal_write_alias("update_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.delete_panel_projection", **proposal_write_alias("delete_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_manufacturing_plan", **proposal_write_alias("create_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.update_manufacturing_plan", **proposal_write_alias("update_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.delete_manufacturing_plan", **proposal_write_alias("delete_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.create_output_job", **proposal_write_alias("create_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.update_output_job", **proposal_write_alias("update_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.delete_output_job", **proposal_write_alias("delete_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.proposal.list", **datum_alias("get_proposals", PROPOSAL_TOOL_SCHEMAS["get_proposals"])},
    {"name": "datum.proposal.show", **datum_alias("show_proposal", PROPOSAL_TOOL_SCHEMAS["show_proposal"])},
    {"name": "datum.proposal.preview", **datum_alias("preview_proposal", PROPOSAL_TOOL_SCHEMAS["preview_proposal"])},
    {"name": "datum.proposal.validate", **datum_alias("validate_proposal", PROPOSAL_TOOL_SCHEMAS["validate_proposal"])},
    {"name": "datum.proposal.review", **proposal_write_alias("review_proposal", PROPOSAL_TOOL_SCHEMAS["review_proposal"], "proposal_review_state_write", PROPOSAL_REVIEW_EVIDENCE)},
    {"name": "datum.proposal.defer", **proposal_write_alias("defer_proposal", PROPOSAL_TOOL_SCHEMAS["defer_proposal"], "proposal_review_state_write", PROPOSAL_REVIEW_EVIDENCE)},
    {"name": "datum.proposal.reject", **proposal_write_alias("reject_proposal", PROPOSAL_TOOL_SCHEMAS["reject_proposal"], "proposal_review_state_write", PROPOSAL_REVIEW_EVIDENCE)},
    {"name": "datum.proposal.accept_apply", **proposal_write_alias("accept_apply_proposal", PROPOSAL_TOOL_SCHEMAS["accept_apply_proposal"], "proposal_gateway_apply", PROPOSAL_APPLY_EVIDENCE)},
    {"name": "datum.proposal.apply", **proposal_write_alias("apply_proposal", PROPOSAL_TOOL_SCHEMAS["apply_proposal"], "proposal_gateway_apply", PROPOSAL_APPLY_EVIDENCE)},
    {"name": "datum.journal.list", **datum_alias("get_journal_list", JOURNAL_TOOL_SCHEMAS["get_journal_list"])},
    {"name": "datum.journal.show", **datum_alias("get_journal_transaction", JOURNAL_TOOL_SCHEMAS["get_journal_transaction"])},
    {"name": "datum.journal.undo", **datum_alias("journal_undo", JOURNAL_TOOL_SCHEMAS["journal_undo"])},
    {"name": "datum.journal.redo", **datum_alias("journal_redo", JOURNAL_TOOL_SCHEMAS["journal_redo"])},
    {"name": "datum.artifact.generate", **datum_alias("generate_artifacts", OUTPUT_JOB_TOOL_SCHEMAS["generate_artifacts"])},
    {"name": "datum.artifact.list", **datum_alias("get_artifacts", OUTPUT_JOB_TOOL_SCHEMAS["get_artifacts"])},
    {"name": "datum.artifact.show", **datum_alias("show_artifact", OUTPUT_JOB_TOOL_SCHEMAS["show_artifact"])},
    {"name": "datum.artifact.files", **datum_alias("get_artifact_files", OUTPUT_JOB_TOOL_SCHEMAS["get_artifact_files"])},
    {"name": "datum.artifact.preview", **datum_alias("preview_artifact_file", OUTPUT_JOB_TOOL_SCHEMAS["preview_artifact_file"])},
    {"name": "datum.artifact.compare", **datum_alias("compare_artifacts", OUTPUT_JOB_TOOL_SCHEMAS["compare_artifacts"])},
    {"name": "datum.artifact.validate", **datum_alias("validate_artifact", OUTPUT_JOB_TOOL_SCHEMAS["validate_artifact"])},
    {"name": "datum.artifact.start_output_job_run", **datum_alias("start_output_job_run", OUTPUT_JOB_TOOL_SCHEMAS["start_output_job_run"])},
    {"name": "datum.artifact.cancel_output_job_run", **datum_alias("cancel_output_job_run", OUTPUT_JOB_TOOL_SCHEMAS["cancel_output_job_run"])},
    {"name": "datum.artifact.export_manufacturing_set", **datum_alias("export_manufacturing_set", OUTPUT_JOB_TOOL_SCHEMAS["export_manufacturing_set"])},
    {"name": "datum.artifact.validate_manufacturing_set", **datum_alias("validate_manufacturing_set", OUTPUT_JOB_TOOL_SCHEMAS["validate_manufacturing_set"])},
    {"name": "datum.manufacturing.create_panel_projection", **proposal_write_alias("create_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.manufacturing.update_panel_projection", **proposal_write_alias("update_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.manufacturing.delete_panel_projection", **proposal_write_alias("delete_panel_projection_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_panel_projection_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.manufacturing.create_plan", **proposal_write_alias("create_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.manufacturing.update_plan", **proposal_write_alias("update_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.manufacturing.delete_plan", **proposal_write_alias("delete_manufacturing_plan_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_manufacturing_plan_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.output_job.create_gerber_set", **gerber_set_proposal_alias()},
    {"name": "datum.output_job.create", **proposal_write_alias("create_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["create_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.output_job.update", **proposal_write_alias("update_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["update_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
    {"name": "datum.output_job.run", **datum_alias("run_output_job", OUTPUT_JOB_TOOL_SCHEMAS["run_output_job"])},
    {"name": "datum.output_job.delete", **proposal_write_alias("delete_output_job_proposal", OUTPUT_JOB_TOOL_SCHEMAS["delete_output_job_proposal"], "proposal_metadata_write", PROPOSAL_METADATA_EVIDENCE)},
]
