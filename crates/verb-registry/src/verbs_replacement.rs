//! The public `datum.replacement` planning read family (5 verbs), transcribed
//! from the legacy canonical alias catalog in `tools_catalog_data.py`.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const UUID: ParamSpec = ParamSpec {
    name: "uuid",
    ty: ParamType::Str,
    required: true,
    doc: "Board component UUID",
    default_json: None,
};

const SCOPE: ParamSpec = ParamSpec {
    name: "scope",
    ty: ParamType::Json,
    required: true,
    doc: "Scoped replacement selector",
    default_json: None,
};

const POLICY: ParamSpec = ParamSpec {
    name: "policy",
    ty: ParamType::Str,
    required: true,
    doc: "Replacement selection policy",
    default_json: None,
};

const PLAN: ParamSpec = ParamSpec {
    name: "plan",
    ty: ParamType::Json,
    required: true,
    doc: "Scoped replacement preview plan",
    default_json: None,
};

const GET_SCOPED_PLAN_SCHEMA: &str = r#"{"type":"object","properties":{"scope":{"type":"object","properties":{"reference_prefix":{"type":["string","null"]},"value_equals":{"type":["string","null"]},"current_package_uuid":{"type":["string","null"]},"current_part_uuid":{"type":["string","null"]}}},"policy":{"type":"string","enum":["best_compatible_package","best_compatible_part"]}},"required":["scope","policy"]}"#;

const EDIT_SCOPED_PLAN_SCHEMA: &str = r#"{"type":"object","properties":{"plan":{"type":"object"},"exclude_component_uuids":{"type":"array","items":{"type":"string"}},"overrides":{"type":"array","items":{"type":"object","properties":{"component_uuid":{"type":"string"},"target_package_uuid":{"type":"string"},"target_part_uuid":{"type":"string"}},"required":["component_uuid","target_package_uuid","target_part_uuid"]}}},"required":["plan"]}"#;

pub(crate) static VERBS: &[VerbSpec] = &[
    VerbSpec {
        id: "datum.replacement.edit_scoped_plan",
        summary: "Exclude or override items in a scoped replacement preview without hand-editing raw JSON.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "edit_scoped_component_replacement_plan",
        },
        params: &[
            PLAN,
            ParamSpec {
                name: "exclude_component_uuids",
                ty: ParamType::StrList,
                required: false,
                doc: "Component UUIDs to exclude from the scoped preview",
                default_json: Some("[]"),
            },
            ParamSpec {
                name: "overrides",
                ty: ParamType::Json,
                required: false,
                doc: "Replacement target overrides",
                default_json: Some("[]"),
            },
        ],
        schema_json_override: Some(EDIT_SCOPED_PLAN_SCHEMA),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.replacement.get_plan",
        summary: "Return a unified replacement-planning report for a board component UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "get_component_replacement_plan",
        },
        params: &[UUID],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.replacement.get_scoped_plan",
        summary: "Preview the exact replacements a scoped compatibility policy would choose before mutation.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "get_scoped_component_replacement_plan",
        },
        params: &[SCOPE, POLICY],
        schema_json_override: Some(GET_SCOPED_PLAN_SCHEMA),
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.replacement.package_candidates",
        summary: "Return compatible target-package candidates for a board component UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "get_package_change_candidates",
        },
        params: &[UUID],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
    VerbSpec {
        id: "datum.replacement.part_candidates",
        summary: "Return compatible target-part candidates for a board component UUID.",
        status: VerbStatus::Public,
        replacements: &[],
        retirement: None,
        dispatch: Dispatch::DaemonRpc {
            method: "get_part_change_candidates",
        },
        params: &[UUID],
        schema_json_override: None,
        write_surface: None,
        terminal: false,
        terminal_optional_params: &[],
        terminal_argv_override: None,
    },
];
