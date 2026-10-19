//! Bounded form geometry; body, directory list and preview retain separate scrolls.
use super::*;
use datum_gui_viewport::ScreenRectPx;

pub(super) struct FormPlan {
    pub body: RectPx,
    pub name: RectPx,
    pub folder: RectPx,
    pub checkbox: RectPx,
    pub location: RectPx,
    pub nav: RectPx,
    pub preview: RectPx,
    pub list_height: f32,
    pub naming: Option<crate::naming_bar::Plan>,
    pub hint: Vec<String>,
    pub error: Vec<String>,
    pub error_y: f32,
    pub full_refusal: Option<String>,
    pub content_height: f32,
}
impl FormPlan {
    pub fn new(
        form: FormData<'_>,
        nav: Option<&datum_gui_protocol::FolderBrowserState>,
        window: RectPx,
    ) -> anyhow::Result<Self> {
        let width = (window.width - 36.0).max(1.0);
        let message = nav
            .map(|n| n.refusal_details(form.leaf(), form.refusal()))
            .unwrap_or_else(|| form.refusal().map(str::to_owned));
        let full_message = message.clone();
        let error = message
            .map(|s| {
                navigation::wrap_text_face(
                    &format!("⊘ {s}"),
                    (width - 20.0).max(1.0),
                    TextFace::UiStrong,
                )
            })
            .transpose()?
            .unwrap_or_default();
        let band_height = if error.is_empty() {
            0.0
        } else {
            error.len().min(3) as f32 * 18.0 + 20.0 + if form.retry() { 44.0 } else { 0.0 }
        };
        let body = RectPx {
            x: 0.0,
            y: 42.0 + band_height,
            width: window.width,
            height: (window.height - 98.0 - band_height).max(1.0),
        };
        let error_y = 52.0;
        let top = body.y + if form.is_new() { 20.0 } else { 14.0 };
        let full_refusal = full_message.filter(|_| error.len() > 3);
        let name = RectPx {
            x: if form.is_new() { 180.0 } else { 168.0 },
            y: top,
            width: (window.width - if form.is_new() { 198.0 } else { 186.0 }).max(1.0),
            height: 34.0,
        };
        let checkbox_width = if form.is_new() {
            measured_text_run_width_px(
                "Use project name",
                design_tokens::typography::CAPTION_SIZE,
                TextFace::Ui,
            )? + 22.0
        } else {
            158.0
        };
        let checkbox = RectPx {
            x: window.width - 18.0 - checkbox_width,
            y: top + 40.0,
            width: checkbox_width,
            height: 34.0,
        };
        let folder = RectPx {
            y: top + 40.0,
            width: if form.is_new() {
                (checkbox.x - 12.0 - name.x).max(1.0)
            } else {
                (name.width - 170.0).max(1.0)
            },
            ..name
        };
        let expanded = nav.is_some_and(|n| n.expanded);
        let location = RectPx {
            x: 180.0,
            y: top + if form.is_new() { 80.0 } else { 40.0 },
            width: (window.width - 198.0).max(1.0),
            height: 34.0,
        };
        let minimum = window.width <= 620.0 || window.height <= 620.0;
        let has_naming = nav.is_some_and(|n| n.naming.is_some());
        let list_height: f32 = if has_naming {
            if minimum {
                168.0
            } else if form.is_new() {
                170.0
            } else {
                240.0
            }
        } else if form.is_new() {
            if minimum { 220.0 } else { 290.0 }
        } else if minimum {
            240.0
        } else {
            360.0
        };
        let query_header = navigation::suggestion_header_lines(nav, location.width)?;
        let query_height = if query_header.is_empty() {
            0.0
        } else {
            query_header.len() as f32 * 16.0 + 8.0
        };
        let list_height = if query_header.is_empty() {
            list_height
        } else {
            (list_height - 24.0).max(1.0)
        };
        let destination = nav
            .and_then(|n| n.preview(form.leaf()))
            .unwrap_or_else(|| "Type a valid folder name".into());
        let mut nav_bounds = RectPx {
            x: location.x,
            y: location.y + location.height + 6.0,
            width: location.width,
            height: if expanded {
                list_height + 38.0 + query_height
            } else {
                0.0
            },
        };
        let naming = nav
            .and_then(|n| n.naming.as_ref())
            .map(|draft| {
                crate::naming_bar::Plan::new(
                    draft,
                    RectPx {
                        x: nav_bounds.x,
                        y: nav_bounds.y + nav_bounds.height + 8.0,
                        width: nav_bounds.width,
                        height: 0.0,
                    },
                )
            })
            .transpose()?;
        if let Some(bar) = &naming {
            nav_bounds.height = bar.bounds.y + bar.bounds.height - nav_bounds.y;
        }
        let hint = if expanded && !has_naming {
            navigation::wrap_text(
                if !query_header.is_empty() {
                    "Suggestions help you type; they do not change Location until you accept one with Tab, Enter or double-click."
                } else if form.is_new() {
                    "Location is the parent. Enter or double-click enters a folder; highlighting alone does not change the parent."
                } else {
                    "Open uses the highlighted folder, otherwise Location. Enter or double-click enters folders; on project.json, it opens this Project."
                },
                nav_bounds.width,
            )?
        } else {
            vec![]
        };
        let preview_y = if expanded {
            nav_bounds.y + nav_bounds.height + hint.len() as f32 * 17.0 + 36.0
        } else {
            location.y + location.height + 32.0
        };
        let preview_width = width;
        let preview_height = if form.is_new() && full_refusal.is_none() {
            // CSS uses max-height:64, so a one-line destination and hint
            // use their intrinsic height; ordinary New fits without body scroll.
            (navigation::wrap_text(&destination, (preview_width - 24.0).max(1.0))?.len() as f32
                * 18.0
                + 30.0)
                .min(64.0)
        } else {
            64.0
        };
        let preview = RectPx {
            x: 18.0,
            y: preview_y,
            width: preview_width,
            height: preview_height,
        };
        let left_bottom = nav_bounds.y + nav_bounds.height + hint.len() as f32 * 17.0 + 12.0;
        let right_bottom = preview.y + preview.height;
        let content_height = left_bottom.max(right_bottom) - body.y + 18.0;
        Ok(Self {
            body,
            name,
            folder,
            checkbox,
            location,
            nav: nav_bounds,
            naming,
            preview,
            list_height,
            hint,
            error,
            error_y,
            full_refusal,
            content_height,
        })
    }
    pub fn reveal(
        &self,
        form: FormData<'_>,
        scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
        reveal: bool,
    ) {
        let reflow = scroll.viewport
            != ScreenRectPx {
                x: self.body.x,
                y: self.body.y,
                width: self.body.width,
                height: self.body.height,
            }
            || scroll.content_height != self.content_height;
        scroll.layout(
            ScreenRectPx {
                x: self.body.x,
                y: self.body.y,
                width: self.body.width,
                height: self.body.height,
            },
            self.content_height,
        );
        if !reveal && !reflow {
            return;
        }
        let rect = match form.focus_target() {
            Some(HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::NameEditor)) => {
                self.naming.as_ref().map(|n| n.editor).unwrap_or(self.nav)
            }
            Some(HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::NameCancel)) => {
                self.naming.as_ref().map(|n| n.cancel).unwrap_or(self.nav)
            }
            Some(HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::NameSubmit)) => {
                self.naming.as_ref().map(|n| n.submit).unwrap_or(self.nav)
            }
            Some(HitTarget::NewProjectName) => self.name,
            Some(HitTarget::NewProjectFolderName) => self.folder,
            Some(HitTarget::NewProjectUseName) => self.checkbox,
            Some(HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::Preview)) => {
                self.preview
            }
            Some(HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::Folders)) => {
                RectPx {
                    y: self.nav.y + 38.0,
                    height: self.list_height,
                    ..self.nav
                }
            }
            Some(HitTarget::FolderBrowser(
                datum_gui_protocol::FolderBrowserFocus::Path
                | datum_gui_protocol::FolderBrowserFocus::Browse,
            )) => self.location,
            Some(HitTarget::FolderBrowser(_)) => RectPx {
                height: 30.0,
                ..self.nav
            },
            _ => return,
        };
        scroll.reveal(rect.y - self.body.y, rect.y + rect.height - self.body.y);
    }
}
pub(super) fn shifted(rect: RectPx, dy: f32) -> RectPx {
    RectPx {
        y: rect.y + dy,
        ..rect
    }
}
