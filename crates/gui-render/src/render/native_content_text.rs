//! Exact prepared Host text used by the native encoder, never terminal payload.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeContentTextSpan {
    pub text: String,
    pub color: [f32; 3],
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeContentText {
    pub region: &'static str,
    pub text: String,
    pub spans: Vec<NativeContentTextSpan>,
    pub position: [f32; 2],
    pub size: f32,
    pub color: [f32; 3],
    pub face: String,
    pub clip: Option<[f32; 4]>,
    pub layout_size: Option<(f32, f32)>,
}

impl PreparedScene {
    pub fn native_content_text(&self) -> Vec<NativeContentText> {
        self.text_runs
            .iter()
            .filter_map(|run| {
                if run.origin != TextOrigin::Host {
                    return None;
                }
                let region = if self.layout.right_sidebar.contains(run.x, run.y) {
                    "inspector"
                } else if self.layout.status_bar.contains(run.x, run.y) {
                    "footer"
                } else {
                    return None;
                };
                Some(NativeContentText {
                    region,
                    text: run.text.clone(),
                    spans: run
                        .rich_spans
                        .iter()
                        .map(|span| NativeContentTextSpan {
                            text: span.text.clone(),
                            color: span.color,
                            bold: span.bold,
                            italic: span.italic,
                        })
                        .collect(),
                    position: [run.x, run.y],
                    size: run.size,
                    color: run.color,
                    face: format!("{:?}", run.face),
                    clip: run.clip_bounds.map(|r| [r.x, r.y, r.width, r.height]),
                    layout_size: run.layout_size,
                })
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "native_content_text_tests.rs"]
mod tests;
