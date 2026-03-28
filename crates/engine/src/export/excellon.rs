use crate::board::Via;

use super::ExportError;
use super::formatting::format_mm_6;

pub fn render_excellon_drill(vias: &[Via]) -> Result<String, ExportError> {
    if vias.iter().any(|via| via.drill <= 0) {
        return Err(ExportError::InvalidViaDrill);
    }

    let mut drills = vias.iter().map(|via| via.drill).collect::<Vec<_>>();
    drills.sort_unstable();
    drills.dedup();

    let mut lines = vec![String::from("M48"), String::from("METRIC,TZ")];
    for (idx, drill) in drills.iter().enumerate() {
        let tool_code = idx + 1;
        lines.push(format!("T{tool_code:02}C{}", format_mm_6(*drill)));
    }
    lines.push(String::from("%"));

    let mut ordered_vias = vias.to_vec();
    ordered_vias.sort_by(|a, b| {
        a.drill
            .cmp(&b.drill)
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    let mut current_tool = None;
    for via in ordered_vias {
        let tool_code = drills.binary_search(&via.drill).expect("known drill tool") + 1;
        if current_tool != Some(tool_code) {
            lines.push(format!("T{tool_code:02}"));
            current_tool = Some(tool_code);
        }
        lines.push(format!(
            "X{}Y{}",
            format_mm_6(via.position.x),
            format_mm_6(via.position.y)
        ));
    }

    lines.push(String::from("M30"));
    Ok(lines.join("\n") + "\n")
}
