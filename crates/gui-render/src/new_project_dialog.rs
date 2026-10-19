//! Complete New/Open forms over shared inline navigation.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter, button};
use datum_gui_protocol::{
    FolderBrowserFocus, FolderBrowserMode, FolderBrowserState, NewProjectFocus, ProjectOpenFocus,
};
use datum_gui_viewport::scroll::ScrollViewport;
#[path = "project_dialog_layout.rs"]
mod form_layout;
#[path = "folder_browser_dialog.rs"]
mod navigation;
#[path = "project_open_dialog.rs"]
mod open;
#[path = "render/new_project_scene.rs"]
mod scene;
pub(super) use scene::render_new_project_dialog;

#[derive(Clone, Copy)]
pub(crate) enum FormData<'a> {
    New(&'a datum_gui_protocol::NewProjectDialogState),
    Open(&'a datum_gui_protocol::ProjectOpenDialogState),
}
impl<'a> FormData<'a> {
    fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }
    fn mode(self) -> FolderBrowserMode {
        if self.is_new() {
            FolderBrowserMode::New
        } else {
            FolderBrowserMode::Open
        }
    }
    fn open(self) -> bool {
        match self {
            Self::New(s) => s.open,
            Self::Open(s) => s.open,
        }
    }
    fn enabled(self) -> bool {
        match self {
            Self::New(s) => s.create_enabled(),
            Self::Open(s) => s.open_enabled(),
        }
    }
    fn busy(self) -> bool {
        matches!(self,Self::Open(s) if s.busy)
    }
    fn host_action_pending(self) -> bool {
        match self {
            Self::New(s) => s.host_action_pending,
            Self::Open(s) => s.host_action_pending,
        }
    }
    fn refusal(self) -> Option<&'a str> {
        match self {
            Self::New(s) => s.refusal.as_deref(),
            Self::Open(s) => s.refusal.as_deref(),
        }
    }
    fn retry(self) -> bool {
        matches!(self,Self::New(s) if s.retry_global_available())
    }
    fn leaf(self) -> Option<&'a str> {
        match self {
            Self::New(s) => Some(s.effective_folder_name()),
            Self::Open(_) => None,
        }
    }
    fn nav_focus(self) -> Option<FolderBrowserFocus> {
        match self {
            Self::New(s) => {
                if let NewProjectFocus::Navigation(f) = s.focus {
                    Some(f)
                } else {
                    None
                }
            }
            Self::Open(s) => {
                if let ProjectOpenFocus::Navigation(f) = s.focus {
                    Some(f)
                } else {
                    None
                }
            }
        }
    }
    fn focus_target(self) -> Option<HitTarget> {
        if let Some(focus) = self.nav_focus() {
            return Some(HitTarget::FolderBrowser(focus));
        }
        Some(match self {
            Self::New(s) => match s.focus {
                NewProjectFocus::ProjectName => HitTarget::NewProjectName,
                NewProjectFocus::FolderName => HitTarget::NewProjectFolderName,
                NewProjectFocus::UseProjectName => HitTarget::NewProjectUseName,
                NewProjectFocus::RetryGlobal => HitTarget::NewProjectRetryGlobal,
                NewProjectFocus::Cancel => HitTarget::NewProjectCancel,
                NewProjectFocus::Create => HitTarget::NewProjectCreate,
                NewProjectFocus::Navigation(_) => return None,
            },
            Self::Open(s) => match s.focus {
                ProjectOpenFocus::Cancel => HitTarget::ProjectOpenCancel,
                ProjectOpenFocus::Open => HitTarget::ProjectOpenConfirm,
                ProjectOpenFocus::Navigation(_) => return None,
            },
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_form(
    form: FormData<'_>,
    nav: Option<&FolderBrowserState>,
    layout: &ShellLayout,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    body_scroll: &mut ScrollViewport,
    list_scroll: &mut ScrollViewport,
    preview_scroll: &mut ScrollViewport,
    reveal: bool,
) -> anyhow::Result<()> {
    if !form.open() {
        return Ok(());
    }
    let nav = nav.filter(|n| n.mode == form.mode());
    let logical = layout.clone().scale_by(1.0 / scale);
    let window = RectPx {
        x: 0.0,
        y: 0.0,
        width: logical.top_menu_bar.width,
        height: logical.status_bar.y + logical.status_bar.height,
    };
    let plan = form_layout::FormPlan::new(form, nav, window)?;
    plan.reveal(form, body_scroll, reveal);
    let dy = -body_scroll.offset();
    let shift = |rect| form_layout::shifted(rect, dy);
    let title = if form.is_new() {
        "New Project"
    } else {
        "Open Project"
    };
    let mut painter = ControlPainter::new(quads, controls, scale);
    painter.push(Quad::from_rect(window, design_tokens::chrome::BG_BASE));
    hits.push(HitRegion {
        target: if form.is_new() {
            HitTarget::NewProjectModal
        } else {
            HitTarget::ProjectOpenModal
        },
        rect: window,
    });
    painter.push(Quad::from_rect(
        RectPx {
            height: 42.0,
            ..window
        },
        design_tokens::chrome::SURFACE_01,
    ));
    draw_text(
        title,
        18.0,
        14.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    let route = format!("File > {title}");
    let route_w = measured_text_run_width_px(
        &route,
        design_tokens::typography::MICRO_SIZE,
        TextFace::Mono,
    )?;
    draw_text(
        &route,
        window.width - route_w - 18.0,
        16.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );
    if !plan.error.is_empty() {
        let band = RectPx {
            x: 18.0,
            y: 46.0,
            width: window.width - 36.0,
            height: plan.body.y - 50.0,
        };
        painter.push(Quad::from_rect(band, design_tokens::chrome::SURFACE_02));
        for (i, line) in plan
            .error
            .iter()
            .take(if plan.full_refusal.is_some() { 2 } else { 3 })
            .enumerate()
        {
            draw_text(
                line,
                24.0,
                plan.error_y + i as f32 * 18.0,
                design_tokens::typography::CAPTION_SIZE,
                design_tokens::chrome::STATUS_ERROR,
                TextFace::UiStrong,
                text,
            );
        }
        if plan.full_refusal.is_some() {
            draw_text(
                if form.is_new() {
                    "Full refusal details continue in the New Project folder preview."
                } else {
                    "Full reason in Project directory preview below."
                },
                24.0,
                plan.error_y + 36.0,
                design_tokens::typography::CAPTION_SIZE,
                design_tokens::chrome::STATUS_ERROR,
                TextFace::UiStrong,
                text,
            );
        }
    }
    if form.retry() {
        let retry = RectPx {
            x: 24.0,
            y: plan.error_y + plan.error.len().min(3) as f32 * 18.0 + 8.0,
            width: 174.0,
            height: 32.0,
        };
        button(
            "Re-read Global defaults",
            retry,
            form.focus_target() == Some(HitTarget::NewProjectRetryGlobal),
            true,
            &mut painter,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::NewProjectRetryGlobal,
            rect: retry,
        });
    }
    let body_start = (painter.len(), text.len(), hits.len());
    if let FormData::New(dialog) = form {
        for (label, rect, value, editor, editable, focus, target) in [
            (
                "Project name",
                shift(plan.name),
                dialog.project_name.as_str(),
                dialog.project_name_editor,
                true,
                NewProjectFocus::ProjectName,
                HitTarget::NewProjectName,
            ),
            (
                "Folder name",
                shift(plan.folder),
                dialog.effective_folder_name(),
                dialog.folder_name_editor,
                !dialog.use_project_name,
                NewProjectFocus::FolderName,
                HitTarget::NewProjectFolderName,
            ),
        ] {
            draw_text(
                label,
                18.0,
                rect.y + 10.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_SECONDARY,
                TextFace::UiStrong,
                text,
            );
            crate::single_line_control::paint(
                value,
                "Enter a name",
                editor,
                dialog.focus == focus,
                editable,
                rect,
                &mut painter,
                text,
                hits,
                target,
            )?;
        }
        let check = shift(plan.checkbox);
        let box_rect = RectPx {
            x: check.x,
            y: check.y + 9.0,
            width: 14.0,
            height: 14.0,
        };
        push_rect_border(&mut painter, box_rect, TEXT_SECONDARY, 1.0);
        if dialog.use_project_name {
            painter.push(Quad::from_rect(
                RectPx {
                    x: box_rect.x + 3.0,
                    y: box_rect.y + 3.0,
                    width: 8.0,
                    height: 8.0,
                },
                TEXT_PRIMARY,
            ));
        }
        draw_text(
            "Use project name",
            check.x + 22.0,
            check.y + 10.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
        if dialog.focus == NewProjectFocus::UseProjectName {
            push_rect_border(&mut painter, check, design_tokens::chrome::STATUS_INFO, 1.0);
        }
        hits.push(HitRegion {
            target: HitTarget::NewProjectUseName,
            rect: check,
        });
    } else {
        draw_text(
            "Choose a native Project directory",
            18.0,
            plan.name.y + dy,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
    }
    drop(painter);
    navigation::paint_location(
        nav,
        form.nav_focus(),
        shift(plan.location),
        !form.busy(),
        controls,
        scale,
        quads,
        text,
        hits,
    )?;
    navigation::paint_navigation(
        nav,
        form.mode(),
        form.nav_focus(),
        shift(plan.nav),
        plan.list_height,
        list_scroll,
        reveal,
        !form.busy(),
        controls,
        scale,
        quads,
        text,
        hits,
    )?;
    if let (Some(draft), Some(bar)) = (nav.and_then(|n| n.naming.as_ref()), plan.naming.as_ref()) {
        crate::naming_bar::paint(
            draft,
            bar,
            dy,
            form.nav_focus(),
            controls,
            scale,
            quads,
            text,
            hits,
        )?;
    }
    let painter = ControlPainter::new(quads, controls, scale);
    let hint_y = plan.nav.y + dy + plan.nav.height + 10.0;
    for (i, line) in plan.hint.iter().enumerate() {
        draw_text(
            line,
            plan.nav.x,
            hint_y + i as f32 * 17.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
    draw_text(
        if form.is_new() {
            "New Project folder"
        } else {
            "Project directory"
        },
        plan.preview.x,
        plan.preview.y + dy - 20.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    drop(painter);
    navigation::paint_preview(
        nav,
        form.mode(),
        form.leaf(),
        plan.full_refusal.as_deref(),
        form.nav_focus(),
        shift(plan.preview),
        preview_scroll,
        controls,
        scale,
        quads,
        text,
        hits,
    )?;
    crate::hit_clipping::clip_content(
        quads,
        text,
        hits,
        body_start.0,
        body_start.1,
        body_start.2,
        RectPx {
            width: (plan.body.width - 11.0).max(1.0),
            ..plan.body
        },
    );
    crate::global_preferences_primitives::paint_scrollbar(body_scroll, quads);
    let mut painter = ControlPainter::new(quads, controls, scale);
    let footer = RectPx {
        x: 0.0,
        y: window.height - 56.0,
        width: window.width,
        height: 56.0,
    };
    painter.push(Quad::from_rect(footer, design_tokens::chrome::BG_BASE));
    let cancel = RectPx {
        x: if form.is_new() {
            window.width - 204.0
        } else {
            18.0
        },
        y: footer.y + 12.0,
        width: 88.0,
        height: 32.0,
    };
    let action = RectPx {
        x: cancel.x + 98.0,
        ..cancel
    };
    if form.host_action_pending() {
        let (x, width) = if form.is_new() {
            (18.0, (cancel.x - 30.0).max(1.0))
        } else {
            (
                action.x + action.width + 12.0,
                (window.width - action.x - action.width - 30.0).max(1.0),
            )
        };
        let lines = crate::control_text::wrap_text(form.mode().host_action_wait_reason(), width)?;
        for (i, line) in lines.iter().enumerate() {
            draw_text(
                line,
                x,
                footer.y + 14.0 + i as f32 * 14.0,
                11.0,
                TEXT_SECONDARY,
                TextFace::Ui,
                text,
            );
        }
    }
    let (cancel_target, action_target) = if form.is_new() {
        (HitTarget::NewProjectCancel, HitTarget::NewProjectCreate)
    } else {
        (HitTarget::ProjectOpenCancel, HitTarget::ProjectOpenConfirm)
    };
    button(
        "Cancel",
        cancel,
        form.focus_target() == Some(cancel_target.clone()),
        true,
        &mut painter,
        text,
    );
    button(
        if form.is_new() {
            "Create"
        } else if form.busy() {
            "Opening…"
        } else {
            "Open"
        },
        action,
        form.focus_target() == Some(action_target.clone()),
        form.enabled(),
        &mut painter,
        text,
    );
    hits.push(HitRegion {
        target: cancel_target,
        rect: cancel,
    });
    hits.push(HitRegion {
        target: action_target,
        rect: action,
    });
    drop(painter);
    Ok(())
}

#[cfg(test)]
#[path = "project_dialog_tests.rs"]
mod tests;
