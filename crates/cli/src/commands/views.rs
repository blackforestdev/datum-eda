// commands/views.rs — cross-family CLI view helpers shared by command
// families that have not yet moved into commands/ (project create, rules).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRulesView {
    pub(crate) domain: &'static str,
    pub(crate) count: usize,
    pub(crate) rules: Vec<serde_json::Value>,
}

pub(crate) fn render_native_project_rules_text(report: &NativeProjectRulesView) -> String {
    let mut lines = vec![format!("rule_count: {}", report.count)];
    if !report.rules.is_empty() {
        lines.push("rules:".to_string());
        for rule in &report.rules {
            lines.push(format!(
                "  {}",
                serde_json::to_string(rule)
                    .expect("CLI text formatting rule serialization must succeed")
            ));
        }
    }
    lines.join("\n")
}
