//! Opt-in native presentation isolation. This produces no product scene and
//! supplies no visual-quality evidence or production rendering fallback.

pub(crate) fn clear_only() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        let enabled = match std::env::var("DATUM_DIAGNOSTIC_NATIVE_FRAME").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("full") => false,
            Ok("clear") => true,
            other => {
                panic!("invalid DATUM_DIAGNOSTIC_NATIVE_FRAME: {other:?}; expected full or clear")
            }
        };
        if enabled {
            super::append_gui_diagnostic_line(
                "DIAGNOSTIC clear-only native frame; no product scene",
            );
        }
        enabled
    })
}

pub(crate) fn submit_clear(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::TextureView,
) -> wgpu::SubmissionIndex {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("datum-diagnostic-native-clear"),
    });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("datum-diagnostic-native-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.08,
                        g: 0.04,
                        b: 0.16,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
    }
    queue.submit(Some(encoder.finish()))
}
