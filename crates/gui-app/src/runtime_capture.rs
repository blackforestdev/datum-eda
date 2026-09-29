//! Native capture target and readback ownership.
use super::*;

impl Runtime {
    #[cfg(feature = "visual")]
    pub(super) fn write_visual_screenshot(&mut self, path: &Path) -> Result<()> {
        let image = self.capture_visual_screenshot()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create screenshot directory {}", parent.display()))?;
        }
        image
            .save(path)
            .with_context(|| format!("write visual shell screenshot {}", path.display()))
    }

    #[cfg(not(feature = "visual"))]
    pub(super) fn write_visual_screenshot(&mut self, _path: &Path) -> Result<()> {
        anyhow::bail!("datum-gui visual screenshots require the datum-gui-app visual feature")
    }

    #[cfg(feature = "visual")]
    fn capture_visual_screenshot(&mut self) -> Result<image::RgbaImage> {
        let target = datum_gui_render::capture_resource::CaptureTarget::new(
            &self.device,
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            self.config.format,
        )?;
        let target_view = datum_gui_render::render_input::FrameTarget::full_texture(&target)?;
        self.renderer.render_session_mut().retry_content();
        if self.renderer.render_session().prepared().is_none() {
            self.build_terminal_prepared_scene()?;
        }
        self.renderer.render_session_mut().check_content_budget()?;
        let owner = self.renderer.resource_owner_id();
        let plan = self.renderer.render_session_mut().prepare_frame(
            owner,
            owner,
            0,
            false,
            self.config.width,
            self.config.height,
        )?;
        let rendered = self
            .renderer
            .encode_capture(plan, &self.device, &self.queue, &target_view);
        target.hold_submission(&self.queue);
        let submitted = rendered?;
        let result = self.read_visual_texture(&target);
        self.renderer.render_session_mut().complete_submitted(
            submitted,
            owner,
            owner,
            0,
            result.is_ok(),
        );
        result
    }

    #[cfg(feature = "visual")]
    fn read_visual_texture(
        &self,
        texture: &datum_gui_render::capture_resource::CaptureTarget,
    ) -> Result<image::RgbaImage> {
        let width = self.config.width;
        let height = self.config.height;
        let unpadded_bytes_per_row = width * COPY_BYTES_PER_PIXEL;
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, WGPU_COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output_buffer =
            datum_gui_render::capture_resource::CaptureReadback::new(&self.device, buffer_size)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("datum-gui-layer-b-visual-readback-encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        texture.hold_submission(&self.queue);
        output_buffer.hold_submission(&self.queue);

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("poll device for visual shell readback")?;
        receiver
            .recv()
            .context("wait for visual shell readback mapping")?
            .context("map visual shell readback buffer")?;

        let mapped = buffer_slice.get_mapped_range();
        let mut pixels = vec![0_u8; (width * height * COPY_BYTES_PER_PIXEL) as usize];
        for row in 0..height as usize {
            let source_start = row * padded_bytes_per_row as usize;
            let source_end = source_start + unpadded_bytes_per_row as usize;
            let dest_start = row * unpadded_bytes_per_row as usize;
            let dest_end = dest_start + unpadded_bytes_per_row as usize;
            pixels[dest_start..dest_end].copy_from_slice(&mapped[source_start..source_end]);
        }
        drop(mapped);
        output_buffer.unmap();

        convert_texture_pixels_to_rgba(&mut pixels, self.config.format)?;
        image::RgbaImage::from_raw(width, height, pixels)
            .context("construct visual shell image from readback pixels")
    }
}

#[cfg(feature = "visual")]
pub(super) fn run_offscreen_visual_test(args: &GuiArgs) -> Result<()> {
    args.validate_visual_args()?;
    append_gui_diagnostic_line("offscreen visual test begin");
    let request = args
        .resolve_request()
        .context("resolve offscreen visual-test review context")?;
    let workspace_include_review = !args.wants_plain_project_board_view();
    let mut state = if let Some(schematic_file) = &args.schematic_file {
        load_kicad_schematic_workspace_state(schematic_file)
            .context("load schematic offscreen workspace state")?
    } else if args.wants_plain_project_board_view() {
        load_board_editor_workspace_state(&request)
            .context("load board editor offscreen workspace state")?
    } else {
        load_live_workspace_state(&request).context("load live offscreen workspace state")?
    };
    // Preset a component selection when requested, mirroring the on-screen launch
    // path in app_bootstrap. `--select` accepts a reference designator (e.g. R1)
    // Unknown selectors leave the inspector empty so captures fail loudly.
    if let Some(sel) = &args.select {
        let object_id = state
            .scene
            .components
            .iter()
            .find(|c| c.reference == *sel)
            .map(|c| c.object_id.clone())
            .unwrap_or_else(|| sel.clone());
        state.select_authored_object(&object_id);
    }
    args.apply_initial_layout(&mut state.ui.layout);
    args.apply_focus_pane(&mut state.ui.layout);
    args.apply_fixture_revision_surface(&mut state.ui);
    args.apply_layers_scroll(&mut state.ui);
    if let Some(menu) = &args.open_menu {
        state.ui.active_menu = Some(menu.clone());
    }
    let mut global_preferences =
        global_preferences_runtime::GlobalPreferencesCoordinator::from_platform()?;
    global_preferences.publish_projection(&mut state.ui);
    if args.open_global_preferences {
        state.ui.global_preferences.reset_transient_view();
        state.ui.global_preferences.open = true;
        keyboard_focus::initialize_application_focus(&mut state, ApplicationFocus::Overlay);
    }
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let (width, height) = args.visual_window_size()?;
    let scale_factor = args.visual_scale_factor.unwrap_or(1.0);
    let screenshot_out = args
        .screenshot_out
        .as_ref()
        .context("--screenshot-out is required for --visual-test")?;
    let mut renderer =
        OffscreenRenderer::new(width, height).context("create offscreen renderer")?;
    renderer
        .warm_workspace_for_surface_scale(&state, Some(camera), scale_factor)
        .context("warm offscreen visual-test renderer")?;
    let image = renderer
        .render_workspace_for_surface_scale(&state, Some(camera), scale_factor)
        .context("render offscreen visual-test workspace")?;
    if let Some(parent) = screenshot_out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create screenshot directory {}", parent.display()))?;
    }
    image.save(screenshot_out).with_context(|| {
        format!(
            "write offscreen visual-test screenshot {}",
            screenshot_out.display()
        )
    })?;
    append_gui_diagnostic_line(format!(
        "offscreen visual test end path={} include_review={workspace_include_review}",
        screenshot_out.display()
    ));
    Ok(())
}
#[cfg(not(feature = "visual"))]
pub(super) fn run_offscreen_visual_test(_args: &GuiArgs) -> Result<()> {
    anyhow::bail!("datum-gui --visual-test requires the datum-gui-app visual feature")
}
