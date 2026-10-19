//! One transactional preparation owner for both complete native Project forms.
use super::*;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_native_project_form(
        &mut self,
        form: FormData<'_>,
        navigation: Option<&FolderBrowserState>,
        view: crate::DialogView,
        body: &mut ScrollViewport,
        list: &mut ScrollViewport,
        preview: &mut ScrollViewport,
        reveal: bool,
    ) -> anyhow::Result<PreparedScene> {
        let _host = self.resource_host.enter();
        let (mut body_pending, mut list_pending, mut preview_pending) =
            (body.clone(), list.clone(), preview.clone());
        let result =
            crate::text_metrics::measurement_owner::with_owner(&mut self.measurement_fonts, || {
                let scale = view.scale.max(0.01);
                let layout = PreparedScene::native_dialog_layout(view.width, view.height, scale);
                let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
                paint_form(
                    form,
                    navigation,
                    &layout,
                    &mut self.control_meshes,
                    scale,
                    &mut quads,
                    &mut text,
                    &mut hits,
                    &mut body_pending,
                    &mut list_pending,
                    &mut preview_pending,
                    reveal,
                )?;
                let (mut popup_quads, mut popup_text) = (vec![], vec![]);
                if let Some(menu) = navigation.and_then(|n| n.menu.as_ref()) {
                    let viewport: RectPx = body_pending.viewport.into();
                    {
                        crate::popup_control::paint(
                            menu,
                            viewport,
                            &mut self.control_meshes,
                            scale,
                            &mut popup_quads,
                            &mut popup_text,
                            &mut hits,
                            HitTarget::FolderBrowserMenuItem,
                        )?;
                    }
                }
                crate::global_preferences_dialog::dialog_coordinates::scale_output(
                    &mut popup_quads,
                    &mut popup_text,
                    &mut vec![],
                    (0, 0, 0),
                    scale,
                );
                crate::global_preferences_dialog::dialog_coordinates::scale_output(
                    &mut quads,
                    &mut text,
                    &mut hits,
                    (0, 0, 0),
                    scale,
                );
                let mut prepared = PreparedScene::from_dialog_parts(
                    layout,
                    quads,
                    text,
                    hits,
                    scale,
                    (view.width, view.height),
                );
                if !popup_quads.is_empty() {
                    prepared.panel_vertices = std::mem::take(&mut prepared.menu_overlay_vertices);
                    prepared.text_runs = std::mem::take(&mut prepared.menu_overlay_text_runs);
                    prepared.menu_overlay_vertices = quads_to_vertices(&popup_quads);
                    prepared.menu_overlay_text_runs = popup_text;
                }
                prepared.set_native_consumer(crate::resource_consumers::Consumer::New);
                Ok(prepared)
            });
        if result.is_ok() {
            *body = body_pending;
            *list = list_pending;
            *preview = preview_pending;
        }
        result
    }
    pub fn prepare_native_new_project(
        &mut self,
        dialog: &datum_gui_protocol::NewProjectDialogState,
        width: u32,
        height: u32,
        scale: f32,
    ) -> anyhow::Result<PreparedScene> {
        self.prepare_native_new_project_scrolled(
            dialog,
            width,
            height,
            scale,
            &mut ScrollViewport::default(),
            false,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_native_new_project_scrolled(
        &mut self,
        dialog: &datum_gui_protocol::NewProjectDialogState,
        width: u32,
        height: u32,
        scale: f32,
        scroll: &mut ScrollViewport,
        reveal_focus: bool,
    ) -> anyhow::Result<PreparedScene> {
        self.prepare_native_project_form(
            FormData::New(dialog),
            None,
            crate::DialogView {
                width,
                height,
                scale,
            },
            scroll,
            &mut ScrollViewport::default(),
            &mut ScrollViewport::default(),
            reveal_focus,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_new_project_dialog(
    dialog: &datum_gui_protocol::NewProjectDialogState,
    layout: &ShellLayout,
    native_window: bool,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    if !native_window {
        return Ok(());
    }
    let starts = (quads.len(), text.len(), hits.len());
    paint_form(
        FormData::New(dialog),
        None,
        layout,
        controls,
        scale,
        quads,
        text,
        hits,
        &mut ScrollViewport::default(),
        &mut ScrollViewport::default(),
        &mut ScrollViewport::default(),
        false,
    )?;
    crate::global_preferences_dialog::dialog_coordinates::scale_output(
        quads, text, hits, starts, scale,
    );
    Ok(())
}
