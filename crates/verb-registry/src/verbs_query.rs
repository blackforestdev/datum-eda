//! The `datum.query` verb family.
//!
//! This prefix is fully registry-owned for MCP catalog projection. Open-session
//! imported-design aliases dispatch through daemon request methods; path-based
//! native aliases dispatch through existing CLI bridge methods. Only
//! `datum.query.source_shards` is advertised in the GUI terminal catalog.
//!
//! Entries MUST stay sorted by id (asserted by lib tests).

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, VerbStatus};

const EMPTY_QUERY_SCHEMA: &str = r#"{"type":"object","properties":{}}"#;
const PATH_QUERY_SCHEMA: &str =
    r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#;
const HIERARCHY_QUERY_SCHEMA: &str =
    r#"{"type":"object","properties":{"path":{"type":["string","null"]}}}"#;
const SYMBOL_UUID_QUERY_SCHEMA: &str = r#"{"type":"object","properties":{"symbol_uuid":{"type":"string"}},"required":["symbol_uuid"]}"#;

const EMPTY_QUERY_SUMMARY: &str =
    "Canonical Datum read-only query alias over the current open session.";
const PATH_QUERY_SUMMARY: &str =
    "Canonical Datum read-only query alias for one native project path.";

const PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: true,
    doc: "Project root directory",
    default_json: None,
};

const OPTIONAL_PATH: ParamSpec = ParamSpec {
    name: "path",
    ty: ParamType::Str,
    required: false,
    doc: "Optional native project root directory",
    default_json: None,
};

const SYMBOL_UUID: ParamSpec = ParamSpec {
    name: "symbol_uuid",
    ty: ParamType::Uuid,
    required: true,
    doc: "Schematic symbol UUID",
    default_json: None,
};

macro_rules! empty_query {
    ($id:literal, $method:literal) => {
        daemon_query!($id, EMPTY_QUERY_SUMMARY, $method, &[], EMPTY_QUERY_SCHEMA)
    };
}

macro_rules! native_query {
    ($id:literal, $method:literal, $subcommand:literal) => {
        native_query!($id, PATH_QUERY_SUMMARY, $method, $subcommand)
    };
    ($id:literal, $summary:expr, $method:literal, $subcommand:literal) => {
        cli_query!(
            $id,
            $summary,
            $method,
            &[PATH],
            PATH_QUERY_SCHEMA,
            &[
                ArgvToken::Lit("query"),
                ArgvToken::Lit($subcommand),
                ArgvToken::Param("path"),
            ],
            false,
            None
        )
    };
}

macro_rules! project_query {
    ($id:literal, $method:literal, $subcommand:literal) => {
        project_query!($id, PATH_QUERY_SUMMARY, $method, $subcommand)
    };
    ($id:literal, $summary:expr, $method:literal, $subcommand:literal) => {
        cli_query!(
            $id,
            $summary,
            $method,
            &[PATH],
            PATH_QUERY_SCHEMA,
            &[
                ArgvToken::Lit("project"),
                ArgvToken::Lit("query"),
                ArgvToken::Param("path"),
                ArgvToken::Lit($subcommand),
            ],
            false,
            None
        )
    };
}

macro_rules! daemon_query {
    ($id:literal, $summary:expr, $method:literal, $params:expr, $schema:expr) => {
        VerbSpec {
            id: $id,
            summary: $summary,
            status: VerbStatus::Public,
            replacements: &[],
            retirement: None,
            dispatch: Dispatch::DaemonRpc { method: $method },
            params: $params,
            schema_json_override: Some($schema),
            write_surface: None,
            terminal: false,
            terminal_optional_params: &[],
            terminal_argv_override: None,
        }
    };
}

macro_rules! cli_query {
    ($id:literal, $summary:expr, $method:literal, $params:expr, $schema:expr, $argv:expr, $terminal:expr, $terminal_argv:expr) => {
        VerbSpec {
            id: $id,
            summary: $summary,
            status: VerbStatus::Public,
            replacements: &[],
            retirement: None,
            dispatch: Dispatch::Cli {
                method: $method,
                argv: $argv,
            },
            params: $params,
            schema_json_override: Some($schema),
            write_surface: None,
            terminal: $terminal,
            terminal_optional_params: &[],
            terminal_argv_override: $terminal_argv,
        }
    };
}

pub(crate) static VERBS: &[VerbSpec] = &[
    project_query!(
        "datum.query.board_dimensions",
        "get_board_dimensions",
        "board-dimensions"
    ),
    project_query!(
        "datum.query.board_keepouts",
        "get_board_keepouts",
        "board-keepouts"
    ),
    project_query!(
        "datum.query.board_net_classes",
        "get_board_net_classes",
        "board-net-classes"
    ),
    project_query!("datum.query.board_nets", "get_board_nets", "board-nets"),
    project_query!(
        "datum.query.board_outline",
        "get_board_outline",
        "board-outline"
    ),
    project_query!("datum.query.board_pads", "get_board_pads", "board-pads"),
    project_query!(
        "datum.query.board_stackup",
        "get_board_stackup",
        "board-stackup"
    ),
    empty_query!("datum.query.board_summary", "get_board_summary"),
    project_query!("datum.query.board_texts", "get_board_texts", "board-texts"),
    project_query!(
        "datum.query.board_tracks",
        "get_board_tracks",
        "board-tracks"
    ),
    project_query!("datum.query.board_vias", "get_board_vias", "board-vias"),
    project_query!("datum.query.board_zones", "get_board_zones", "board-zones"),
    empty_query!("datum.query.bus_entries", "get_bus_entries"),
    empty_query!("datum.query.buses", "get_buses"),
    native_query!(
        "datum.query.component_instances",
        "Read authored ComponentInstance records plus resolver-bound symbol/package refs for a native project.",
        "get_component_instances",
        "component-instances"
    ),
    empty_query!("datum.query.components", "get_components"),
    empty_query!(
        "datum.query.connectivity_diagnostics",
        "get_connectivity_diagnostics"
    ),
    empty_query!("datum.query.design_rules", "get_design_rules"),
    daemon_query!(
        "datum.query.hierarchy",
        "Canonical Datum hierarchy query. With path, reads native project hierarchy; without path, uses legacy open-session hierarchy.",
        "get_project_hierarchy",
        &[OPTIONAL_PATH],
        HIERARCHY_QUERY_SCHEMA
    ),
    native_query!(
        "datum.query.import_map",
        "Read resolver-validated import-key identity mappings for a native project.",
        "get_import_map",
        "import-map"
    ),
    empty_query!("datum.query.labels", "get_labels"),
    native_query!(
        "datum.query.manufacturing_plans",
        "Return resolver-discovered native ManufacturingPlan entries for one project.",
        "get_manufacturing_plans",
        "manufacturing-plans"
    ),
    empty_query!("datum.query.netlist", "get_netlist"),
    empty_query!("datum.query.noconnects", "get_noconnects"),
    native_query!(
        "datum.query.output_jobs",
        "Return resolver-discovered native OutputJob entries for one project.",
        "get_output_jobs",
        "output-jobs"
    ),
    native_query!(
        "datum.query.panel_projections",
        "Return resolver-discovered native PanelProjection entries for one project.",
        "get_panel_projections",
        "panel-projections"
    ),
    empty_query!("datum.query.ports", "get_ports"),
    native_query!(
        "datum.query.relationships",
        "Read authored relationship records plus derived resolver status for a native project.",
        "get_relationships",
        "relationships"
    ),
    project_query!(
        "datum.query.schematic_bus_entries",
        "get_schematic_bus_entries",
        "bus-entries"
    ),
    project_query!(
        "datum.query.schematic_buses",
        "get_schematic_buses",
        "buses"
    ),
    project_query!(
        "datum.query.schematic_drawings",
        "get_schematic_drawings",
        "drawings"
    ),
    project_query!(
        "datum.query.schematic_junctions",
        "get_schematic_junctions",
        "junctions"
    ),
    project_query!(
        "datum.query.schematic_labels",
        "get_schematic_labels",
        "labels"
    ),
    empty_query!("datum.query.schematic_nets", "get_schematic_net_info"),
    project_query!(
        "datum.query.schematic_noconnects",
        "get_schematic_noconnects",
        "noconnects"
    ),
    project_query!(
        "datum.query.schematic_ports",
        "get_schematic_ports",
        "ports"
    ),
    empty_query!("datum.query.schematic_summary", "get_schematic_summary"),
    project_query!(
        "datum.query.schematic_texts",
        "get_schematic_texts",
        "texts"
    ),
    project_query!(
        "datum.query.schematic_wires",
        "get_schematic_wires",
        "wires"
    ),
    empty_query!("datum.query.sheets", "get_sheets"),
    cli_query!(
        "datum.query.source_shards",
        PATH_QUERY_SUMMARY,
        "get_source_shards",
        &[PATH],
        PATH_QUERY_SCHEMA,
        &[
            ArgvToken::Lit("project"),
            ArgvToken::Lit("query"),
            ArgvToken::Param("path"),
            ArgvToken::Lit("resolve-debug"),
        ],
        true,
        Some(&[
            ArgvToken::Lit("project"),
            ArgvToken::Lit("query"),
            ArgvToken::Param("path"),
            ArgvToken::Lit("resolve-debug"),
        ])
    ),
    daemon_query!(
        "datum.query.symbol_fields",
        "Canonical Datum read-only query alias for one schematic symbol object.",
        "get_symbol_fields",
        &[SYMBOL_UUID],
        SYMBOL_UUID_QUERY_SCHEMA
    ),
    empty_query!("datum.query.symbols", "get_symbols"),
    native_query!(
        "datum.query.variants",
        "Read authored variant overlays plus derived population/applicability for a native project.",
        "get_variants",
        "variants"
    ),
    native_query!(
        "datum.query.zone_fills",
        "Return resolver-derived native board zone-fill state without pretending unfilled zones are copper.",
        "get_zone_fills",
        "zone-fills"
    ),
];
