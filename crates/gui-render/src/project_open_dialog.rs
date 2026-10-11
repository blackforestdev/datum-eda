//! Open Project uses the existing native dialog preparation and controls.
use super::*;
use datum_gui_protocol::{ProjectOpenDialogState, ProjectOpenFocus};

impl Renderer {
    pub fn prepare_native_open_project(
        &mut self,
        dialog: &ProjectOpenDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> anyhow::Result<PreparedScene> {
        let _host = self.resource_host.enter();
        crate::text_metrics::measurement_owner::with_owner(&mut self.measurement_fonts, || {
            let scale = scale_factor.max(0.01);
            let layout = PreparedScene::native_dialog_layout(width, height, scale);
            let logical = layout.clone().scale_by(1.0 / scale);
            let window = RectPx {
                x: 0.0,
                y: 0.0,
                width: logical.top_menu_bar.width,
                height: logical.status_bar.y + logical.status_bar.height,
            };
            let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
            let mut painter = ControlPainter::new(&mut quads, &mut self.control_meshes, scale);
            painter.push(Quad::from_rect(window, design_tokens::chrome::BG_BASE));
            hits.push(HitRegion {
                target: HitTarget::ProjectOpenModal,
                rect: window,
            });
            draw_text(
                "Open Project",
                18.0,
                16.0,
                design_tokens::typography::BODY_SIZE,
                TEXT_PRIMARY,
                TextFace::UiStrong,
                &mut text,
            );
            draw_text(
                "Choose a native Project directory",
                18.0,
                58.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_SECONDARY,
                TextFace::Ui,
                &mut text,
            );
            draw_text_field(
                "Directory",
                &dialog.directory,
                "Project directory",
                dialog.focus == ProjectOpenFocus::Directory,
                92.0,
                (window.width - 36.0).max(1.0),
                &mut painter,
                &mut text,
                &mut hits,
                HitTarget::ProjectOpenDirectory,
            )?;
            let cancel = RectPx {
                x: 18.0,
                y: 150.0,
                width: 88.0,
                height: 32.0,
            };
            let open = RectPx { x: 118.0, ..cancel };
            button(
                "Cancel",
                cancel,
                dialog.focus == ProjectOpenFocus::Cancel,
                true,
                &mut painter,
                &mut text,
            );
            button(
                if dialog.busy { "Opening…" } else { "Open" },
                open,
                dialog.focus == ProjectOpenFocus::Open,
                dialog.open_enabled(),
                &mut painter,
                &mut text,
            );
            hits.push(HitRegion {
                target: HitTarget::ProjectOpenCancel,
                rect: cancel,
            });
            hits.push(HitRegion {
                target: HitTarget::ProjectOpenConfirm,
                rect: open,
            });
            if let Some(reason) = &dialog.refusal {
                draw_text(
                    &truncate_text(reason, ((window.width - 36.0) / 7.0).max(1.0) as usize),
                    18.0,
                    206.0,
                    design_tokens::typography::CAPTION_SIZE,
                    TEXT_PRIMARY,
                    TextFace::Ui,
                    &mut text,
                );
            }
            drop(painter);
            crate::global_preferences_dialog::dialog_coordinates::scale_output(
                &mut quads,
                &mut text,
                &mut hits,
                (0, 0, 0),
                scale,
            );
            Ok(PreparedScene::from_dialog_parts(
                layout,
                quads,
                text,
                hits,
                scale,
                (width, height),
            ))
        })
    }
}
