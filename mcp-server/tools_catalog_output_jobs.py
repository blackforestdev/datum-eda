"""OutputJob MCP tool schemas."""

OUTPUT_JOB_TOOL_SCHEMAS = {
    "generate_artifacts": {
        "description": "Generate derived production artifacts from include scopes for a native project, or execute one authored OutputJob by id.",
        "x_dispatch_args": ["path", "output_dir", "include", "prefix", "output_job"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_dir": {"type": ["string", "null"]},
                "include": {"type": ["string", "null"]},
                "prefix": {"type": ["string", "null"]},
                "output_job": {"type": ["string", "null"]},
            },
            "required": ["path"],
        },
    },
    "get_artifacts": {
        "description": "Return resolver-discovered generated artifact metadata for one native project.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]},
    },
    "show_artifact": {
        "description": "Return one resolver-discovered generated artifact metadata record for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "artifact": {"type": "string"}},
            "required": ["path", "artifact"],
        },
    },
    "get_artifact_files": {
        "description": "Return generated files and production projection proofs for one artifact.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "artifact": {"type": "string"}},
            "required": ["path", "artifact"],
        },
    },
    "preview_artifact_file": {
        "description": "Preview one generated artifact file through supported semantic readers.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "artifact": {"type": "string"},
                "artifact_dir": {"type": ["string", "null"]},
                "file": {"type": "string"},
            },
            "required": ["path", "artifact", "file"],
        },
    },
    "compare_artifacts": {
        "description": "Compare two resolver-discovered generated artifact metadata records for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "before": {"type": "string"}, "after": {"type": "string"}},
            "required": ["path", "before", "after"],
        },
    },
    "validate_artifact": {
        "description": "Validate one resolver-discovered generated artifact metadata record for a native project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "artifact": {"type": "string"}},
            "required": ["path", "artifact"],
        },
    },
    "get_panel_projections": {
        "description": "Return resolver-discovered native PanelProjection entries for one project.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]},
    },
    "create_panel_projection": {
        "description": "Create or reuse one deterministic native PanelProjection.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "key": {"type": "string"}, "name": {"type": ["string", "null"]}, "board": {"type": ["string", "null"]}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "rotation_deg": {"type": "integer"}},
            "required": ["path", "key"],
        },
    },
    "create_panel_projection_proposal": {
        "description": "Create a draft proposal for a PanelProjection creation without mutating panel shards.",
        "x_dispatch_args": ["path", "key", "name", "board", "x_nm", "y_nm", "rotation_deg", "proposal", "rationale"],
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "key": {"type": "string"}, "name": {"type": ["string", "null"]}, "board": {"type": ["string", "null"]}, "x_nm": {"type": "integer"}, "y_nm": {"type": "integer"}, "rotation_deg": {"type": "integer"}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}},
            "required": ["path", "key"],
        },
    },
    "update_panel_projection": {
        "description": "Update one native PanelProjection through the journaled substrate path.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "panel_projection": {"type": "string"}, "name": {"type": ["string", "null"]}, "board": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "rotation_deg": {"type": ["integer", "null"]}},
            "required": ["path", "panel_projection"],
        },
    },
    "update_panel_projection_proposal": {
        "description": "Create a draft proposal for a PanelProjection update without mutating panel shards.",
        "x_dispatch_args": ["path", "panel_projection", "name", "board", "x_nm", "y_nm", "rotation_deg", "proposal", "rationale"],
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "panel_projection": {"type": "string"}, "name": {"type": ["string", "null"]}, "board": {"type": ["string", "null"]}, "x_nm": {"type": ["integer", "null"]}, "y_nm": {"type": ["integer", "null"]}, "rotation_deg": {"type": ["integer", "null"]}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}},
            "required": ["path", "panel_projection"],
        },
    },
    "delete_panel_projection": {
        "description": "Delete one native PanelProjection through the journaled substrate path.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "panel_projection": {"type": "string"}}, "required": ["path", "panel_projection"]},
    },
    "delete_panel_projection_proposal": {
        "description": "Create a draft proposal for deleting one PanelProjection without mutating panel shards.",
        "x_dispatch_args": ["path", "panel_projection", "proposal", "rationale"],
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "panel_projection": {"type": "string"}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}}, "required": ["path", "panel_projection"]},
    },
    "get_manufacturing_plans": {
        "description": "Return resolver-discovered native ManufacturingPlan entries for one project.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]},
    },
    "create_manufacturing_plan": {
        "description": "Create or reuse one deterministic native ManufacturingPlan.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "prefix": {"type": "string"}, "name": {"type": ["string", "null"]}, "variant": {"type": ["string", "null"]}, "panel_projection": {"type": ["string", "null"]}},
            "required": ["path", "prefix"],
        },
    },
    "create_manufacturing_plan_proposal": {
        "description": "Create a draft proposal for a ManufacturingPlan creation without mutating manufacturing plan shards.",
        "x_dispatch_args": ["path", "prefix", "name", "variant", "panel_projection", "proposal", "rationale"],
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "prefix": {"type": "string"}, "name": {"type": ["string", "null"]}, "variant": {"type": ["string", "null"]}, "panel_projection": {"type": ["string", "null"]}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}},
            "required": ["path", "prefix"],
        },
    },
    "update_manufacturing_plan": {
        "description": "Update one native ManufacturingPlan through the journaled substrate path.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "manufacturing_plan": {"type": "string"}, "name": {"type": ["string", "null"]}, "prefix": {"type": ["string", "null"]}, "variant": {"type": ["string", "null"]}, "clear_variant": {"type": ["boolean", "null"]}, "panel_projection": {"type": ["string", "null"]}, "clear_panel_projection": {"type": ["boolean", "null"]}},
            "required": ["path", "manufacturing_plan"],
        },
    },
    "update_manufacturing_plan_proposal": {
        "description": "Create a draft proposal for a ManufacturingPlan update without mutating manufacturing plan shards.",
        "x_dispatch_args": ["path", "manufacturing_plan", "name", "prefix", "variant", "clear_variant", "panel_projection", "clear_panel_projection", "proposal", "rationale"],
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}, "manufacturing_plan": {"type": "string"}, "name": {"type": ["string", "null"]}, "prefix": {"type": ["string", "null"]}, "variant": {"type": ["string", "null"]}, "clear_variant": {"type": ["boolean", "null"]}, "panel_projection": {"type": ["string", "null"]}, "clear_panel_projection": {"type": ["boolean", "null"]}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}},
            "required": ["path", "manufacturing_plan"],
        },
    },
    "delete_manufacturing_plan": {
        "description": "Delete one native ManufacturingPlan through the journaled substrate path.",
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "manufacturing_plan": {"type": "string"}}, "required": ["path", "manufacturing_plan"]},
    },
    "delete_manufacturing_plan_proposal": {
        "description": "Create a draft proposal for deleting one ManufacturingPlan without mutating manufacturing plan shards.",
        "x_dispatch_args": ["path", "manufacturing_plan", "proposal", "rationale"],
        "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "manufacturing_plan": {"type": "string"}, "proposal": {"type": ["string", "null"]}, "rationale": {"type": ["string", "null"]}}, "required": ["path", "manufacturing_plan"]},
    },
    "get_output_jobs": {
        "description": "Return resolver-discovered native OutputJob entries for one project.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    "create_gerber_output_job": {
        "description": "Create or reuse the deterministic Gerber-set OutputJob for one native project.",
        "x_dispatch_args": [
            "path",
            "prefix",
            "name",
            "manufacturing_plan",
            "output_dir",
            "variant",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "prefix": {"type": "string"},
                "name": {"type": ["string", "null"]},
                "manufacturing_plan": {"type": ["string", "null"]},
                "variant": {"type": ["string", "null"]},
                "output_dir": {"type": ["string", "null"]},
            },
            "required": ["path", "prefix"],
        },
    },
    "create_output_job": {
        "description": "Create or reuse a deterministic OutputJob for one artifact include scope: gerber-set, manufacturing-set, bom, pnp, or drill.",
        "x_dispatch_args": [
            "path",
            "prefix",
            "include",
            "name",
            "manufacturing_plan",
            "output_dir",
            "variant",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "prefix": {"type": "string"},
                "output_dir": {"type": ["string", "null"]},
                "include": {"type": "string"},
                "name": {"type": ["string", "null"]},
                "manufacturing_plan": {"type": ["string", "null"]},
                "variant": {"type": ["string", "null"]},
            },
            "required": ["path", "prefix", "include"],
        },
    },
    "create_output_job_proposal": {
        "description": "Create a draft proposal for an OutputJob creation without mutating the OutputJob shard.",
        "x_dispatch_args": [
            "path",
            "prefix",
            "include",
            "name",
            "manufacturing_plan",
            "output_dir",
            "proposal",
            "rationale",
            "variant",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "prefix": {"type": "string"},
                "output_dir": {"type": ["string", "null"]},
                "include": {"type": "string"},
                "name": {"type": ["string", "null"]},
                "manufacturing_plan": {"type": ["string", "null"]},
                "variant": {"type": ["string", "null"]},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "prefix", "include"],
        },
    },
    "update_output_job": {
        "description": "Update one native OutputJob settings through the journaled substrate path.",
        "x_dispatch_args": [
            "path",
            "output_job",
            "name",
            "output_dir",
            "manufacturing_plan",
            "clear_manufacturing_plan",
            "clear_output_dir",
            "variant",
            "clear_variant",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
                "name": {"type": ["string", "null"]},
                "output_dir": {"type": ["string", "null"]},
                "manufacturing_plan": {"type": ["string", "null"]},
                "variant": {"type": ["string", "null"]},
                "clear_manufacturing_plan": {"type": ["boolean", "null"]},
                "clear_variant": {"type": ["boolean", "null"]},
                "clear_output_dir": {"type": ["boolean", "null"]},
            },
            "required": ["path", "output_job"],
        },
    },
    "update_output_job_proposal": {
        "description": "Create a draft proposal for an OutputJob settings update without mutating the OutputJob shard.",
        "x_dispatch_args": [
            "path",
            "output_job",
            "name",
            "output_dir",
            "manufacturing_plan",
            "clear_manufacturing_plan",
            "clear_output_dir",
            "proposal",
            "rationale",
            "variant",
            "clear_variant",
        ],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
                "name": {"type": ["string", "null"]},
                "output_dir": {"type": ["string", "null"]},
                "manufacturing_plan": {"type": ["string", "null"]},
                "variant": {"type": ["string", "null"]},
                "clear_manufacturing_plan": {"type": ["boolean", "null"]},
                "clear_variant": {"type": ["boolean", "null"]},
                "clear_output_dir": {"type": ["boolean", "null"]},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "output_job"],
        },
    },
    "run_output_job": {
        "description": "Execute one authored OutputJob using its stored include, prefix, and output directory settings.",
        "x_dispatch_args": ["path", "output_job", "output_dir"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
                "output_dir": {"type": ["string", "null"]},
            },
            "required": ["path", "output_job"],
        },
    },
    "start_output_job_run": {
        "description": "Persist a running OutputJobRun evidence record for one authored OutputJob.",
        "x_dispatch_args": ["path", "output_job"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
            },
            "required": ["path", "output_job"],
        },
    },
    "cancel_output_job_run": {
        "description": "Mark one existing OutputJobRun evidence record canceled.",
        "x_dispatch_args": ["path", "run"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "run": {"type": "string"},
            },
            "required": ["path", "run"],
        },
    },
    "delete_output_job": {
        "description": "Delete one native OutputJob through the journaled substrate path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
            },
            "required": ["path", "output_job"],
        },
    },
    "delete_output_job_proposal": {
        "description": "Create a draft proposal for deleting one OutputJob without mutating the OutputJob shard.",
        "x_dispatch_args": ["path", "output_job", "proposal", "rationale"],
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_job": {"type": "string"},
                "proposal": {"type": ["string", "null"]},
                "rationale": {"type": ["string", "null"]},
            },
            "required": ["path", "output_job"],
        },
    },
    "export_manufacturing_set": {
        "description": "Export the current supported manufacturing set and persist resolver-owned artifact/run evidence.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_dir": {"type": "string"},
                "prefix": {"type": ["string", "null"]},
            },
            "required": ["path", "output_dir"],
        },
    },
    "validate_manufacturing_set": {
        "description": "Validate an exported manufacturing set against current project state and persisted artifact hashes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "output_dir": {"type": "string"},
                "prefix": {"type": ["string", "null"]},
            },
            "required": ["path", "output_dir"],
        },
    },
}
