use super::*;

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum SummaryView {
    Board {
        name: String,
        layers: usize,
        components: usize,
        nets: usize,
    },
    Schematic {
        sheets: usize,
        symbols: usize,
        labels: usize,
        ports: usize,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum NetListView {
    Board { nets: Vec<BoardNetInfo> },
    Schematic { nets: Vec<SchematicNetInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum ComponentListView {
    Board { components: Vec<ComponentInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum LabelListView {
    Schematic { labels: Vec<LabelInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum PortListView {
    Schematic { ports: Vec<PortInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum HierarchyView {
    Schematic { hierarchy: HierarchyInfo },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum DiagnosticsView {
    Board {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
    Schematic {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum UnroutedView {
    Board { airwires: Vec<Airwire> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(crate) enum DesignRuleListView {
    Board { rules: Vec<Rule> },
}
