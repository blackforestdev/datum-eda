//! Construction/pixel proof, distinct from native S5A acceptance.
use super::*;
use crate::RectPx;
#[test]
#[ignore = "requires local hardware GPU: physical capsule footprint and upload lifetime"]
fn selection_capsule_pixels_preserve_material_clip_and_two_pixel_boundary() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    prepared.menu_overlay_vertices.clear();
    prepared.menu_overlay_text_runs.clear();
    // Exercise the production suffix through an ordinary frame descriptor.
    prepared.scene_viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 960.0,
        height: 720.0,
    };
    prepared.selection_strokes = vec![crate::selection_stroke::Stroke {
        from: [100.0, 100.0],
        to: [300.0, 100.0],
        material_width: [0.4, 0.6, 0.2, 20.0],
        clip: [110.0, 80.0, 290.0, 120.0],
        high_contrast: [1.0, 0.0, 0.0, 0.0],
    }];
    let mut host = hardware_renderer(960, 720);
    let image = capture(&mut host, &prepared);
    assert_eq!(image.get_pixel(200, 100).0[..3], [110, 165, 55]);
    for y in [88, 89, 110, 111] {
        assert_eq!(image.get_pixel(200, y).0[..3], [206, 90, 146]);
    }
    assert_ne!(image.get_pixel(200, 87).0[..3], [206, 90, 146]);
    assert_ne!(image.get_pixel(200, 112).0[..3], [206, 90, 146]);
    assert_ne!(image.get_pixel(109, 100).0[..3], [110, 165, 55]);
    assert!(host.renderer.selection_stroke_gpu.last_upload_bytes > 0);
    let reserved = host.renderer.screen_budget.used();
    assert_eq!(image, capture(&mut host, &prepared));
    assert_eq!(host.renderer.selection_stroke_gpu.last_upload_bytes, 0);
    assert_eq!(host.renderer.screen_budget.used(), reserved);
    // Reused stream is unchanged; no selection bytes enter authored world buffers.
    let snapshot = host.renderer.screen_upload_snapshot_bytes();
    assert_eq!(image, capture(&mut host, &prepared));
    assert_eq!(snapshot, host.renderer.screen_upload_snapshot_bytes());
    prepared.selection_strokes[0].high_contrast[0] = 0.0;
    let ordinary = capture(&mut host, &prepared);
    assert_ne!(ordinary.get_pixel(200, 90), image.get_pixel(200, 90));
    assert_eq!(ordinary.get_pixel(200, 100), image.get_pixel(200, 100));
    // Minimum ring/X modes use the same production stream, never the lift
    // pass. Compare their authored center against the actual baseline image.
    prepared.selection_strokes.clear();
    let baseline = capture(&mut host, &prepared);
    prepared
        .selection_strokes
        .push(crate::selection_stroke::Stroke::minimum(
            [200.0, 100.0],
            [200.0, 100.0],
            prepared.scene_viewport,
            1.0,
        ));
    let ring = capture(&mut host, &prepared);
    assert_eq!(ring.get_pixel(200, 100), baseline.get_pixel(200, 100));
    assert_eq!(ring.get_pixel(200, 97).0[..3], [206, 90, 146]);
    assert_eq!(ring.get_pixel(200, 102).0[..3], [206, 90, 146]);
    assert_eq!(ring.get_pixel(200, 95), baseline.get_pixel(200, 95));
    prepared.selection_strokes = vec![
        crate::selection_stroke::Stroke::minimum(
            [197.0, 97.0],
            [203.0, 103.0],
            prepared.scene_viewport,
            2.0,
        ),
        crate::selection_stroke::Stroke::minimum(
            [197.0, 103.0],
            [203.0, 97.0],
            prepared.scene_viewport,
            2.0,
        ),
    ];
    let cross = capture(&mut host, &prepared);
    assert_eq!(cross.get_pixel(200, 100).0[..3], [206, 90, 146]);
    assert_eq!(cross.get_pixel(200, 96), baseline.get_pixel(200, 96));
    prepared.selection_strokes.clear();
    capture(&mut host, &prepared);
    assert!(host.renderer.selection_stroke_gpu.buffer().is_none());
}

#[test]
#[ignore = "requires local hardware GPU: exact mixed-pane clip and capsule input parity"]
fn compact_mixed_clip_stream_matches_each_individual_capsule_footprint() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0)
            .unwrap();
    prepared.menu_overlay_vertices.clear();
    prepared.menu_overlay_text_runs.clear();
    let mut host = hardware_renderer(960, 720);
    let baseline = capture(&mut host, &prepared);
    let mut strokes = Vec::new();
    for mode in 0..3 {
        for contrast in 0..2 {
            let index = mode * 2 + contrast;
            let x = 70.0 + index as f32 * 130.0;
            strokes.push(crate::selection_stroke::Stroke {
                from: if mode == 0 {
                    [x - 10.0, 100.0]
                } else if mode == 1 {
                    [x + 40.0, 100.0]
                } else {
                    [x + 35.0, 95.0]
                },
                to: if mode == 0 {
                    [x + 110.0, 100.0]
                } else if mode == 1 {
                    [x + 40.0, 100.0]
                } else {
                    [x + 45.0, 105.0]
                },
                material_width: [
                    0.4,
                    0.6,
                    0.2,
                    if mode == 0 {
                        20.0
                    } else if mode == 1 {
                        6.0
                    } else {
                        2.0
                    },
                ],
                clip: [x, 80.0, x + 90.0, 120.0],
                high_contrast: [contrast as f32, mode as f32, 0.0, 0.0],
            });
        }
    }
    prepared.selection_strokes = strokes.clone();
    let combined = capture(&mut host, &prepared);
    assert_ne!(combined, baseline);
    for stroke in &strokes {
        prepared.selection_strokes = vec![*stroke];
        let single = capture(&mut host, &prepared);
        for y in 75..125 {
            for x in stroke.clip[0] as u32 - 2..stroke.clip[2] as u32 + 2 {
                assert_eq!(
                    combined.get_pixel(x, y),
                    single.get_pixel(x, y),
                    "mixed exact clip/mode at {x},{y}"
                );
            }
        }
        assert_eq!(
            single.get_pixel(stroke.clip[0] as u32 - 1, 100),
            baseline.get_pixel(stroke.clip[0] as u32 - 1, 100)
        );
        assert_eq!(
            single.get_pixel(stroke.clip[2] as u32, 100),
            baseline.get_pixel(stroke.clip[2] as u32, 100)
        );
    }
    prepared.selection_strokes = strokes;
    assert_eq!(capture(&mut host, &prepared), combined);
    let usage = host.renderer.screen_upload_snapshot_bytes();
    assert_eq!(capture(&mut host, &prepared), combined);
    assert_eq!(host.renderer.selection_stroke_gpu.last_upload_bytes, 0);
    assert_eq!(host.renderer.screen_upload_snapshot_bytes(), usage);
}

#[test]
#[ignore = "requires local hardware GPU: admitted owned encoding refusal and retry"]
fn owned_compact_preparation_refusal_preserves_pending_streams_for_retry() {
    use crate::text_gpu::{budget::Budget, upload::plan::Source};
    let host = hardware_renderer(320, 240);
    let staging = Budget::new(8192);
    let screen = Budget::new(128);
    let mut clips =
        crate::selection_stroke::gpu::Owner::new(&host.device, screen.clone(), staging.clone());
    let mut instances = crate::gpu_data::screen_buffer::ScreenBuffer::with_budget(screen.clone())
        .with_staging_budget(staging.clone());
    let stroke = crate::selection_stroke::Stroke {
        from: [20.0, 20.0],
        to: [40.0, 20.0],
        material_width: [0.4, 0.6, 0.2, 4.0],
        clip: [0.0, 0.0, 100.0, 100.0],
        high_contrast: [0.0; 4],
    };
    clips
        .prepare(&mut instances, &host.device, &[stroke])
        .unwrap();
    fn payloads(
        clips: &crate::selection_stroke::gpu::Owner,
        instances: &crate::gpu_data::screen_buffer::ScreenBuffer,
    ) -> Vec<(u64, Vec<u8>)> {
        let mut sources: Vec<Source<'_>> = Vec::new();
        clips.gpu.append_sources(&mut sources);
        instances.append_sources(&mut sources);
        crate::text_gpu::upload::plan::Uploads::sources(&sources)
            .map(|upload| (upload.offset, upload.bytes.to_vec()))
            .collect()
    }
    let pending = payloads(&clips, &instances);
    let reserved = staging.used();
    let previous_targets = (
        clips.gpu.submission_ref().unwrap().allocation_id,
        instances.submission_ref().unwrap().allocation_id,
    );
    let previous_binding = format!("{:?}", clips.binding);

    assert!(
        clips
            .prepare(&mut instances, &host.device, &vec![stroke; 1000])
            .is_err()
    );
    assert_eq!(payloads(&clips, &instances), pending);
    assert_eq!(staging.used(), reserved);
    // Encoding fits staging, but the complete new GPU generation cannot fit
    // alongside the prior generation. No old pending source may be consumed.
    let mut changed_clip = stroke;
    changed_clip.clip = [11.0, 12.0, 71.0, 72.0];
    assert!(
        clips
            .prepare(&mut instances, &host.device, &[changed_clip, changed_clip])
            .is_err()
    );
    assert_eq!(payloads(&clips, &instances), pending);
    assert_eq!(staging.used(), reserved);
    assert_eq!(
        (
            clips.gpu.submission_ref().unwrap().allocation_id,
            instances.submission_ref().unwrap().allocation_id
        ),
        previous_targets
    );
    assert_eq!(format!("{:?}", clips.binding), previous_binding);
    let mut changed = changed_clip;
    changed.material_width[0] = 0.75;
    clips
        .prepare(&mut instances, &host.device, &[changed])
        .unwrap();
    let retry = payloads(&clips, &instances);
    assert_eq!(
        retry.iter().map(|(_, bytes)| bytes.len()).sum::<usize>(),
        56
    );
    let mut sources = Vec::new();
    clips.gpu.append_sources(&mut sources);
    instances.append_sources(&mut sources);
    let uploads = crate::text_gpu::upload::plan::Uploads::sources(&sources).collect::<Vec<_>>();
    crate::text_gpu::upload::submit_buffers_for_test(&host.device, &host.queue, &uploads);
    drop(uploads);
    drop(sources);
    clips.gpu.finish_uploads();
    instances.finish_uploads();
    assert!(payloads(&clips, &instances).is_empty());
    clips
        .prepare(&mut instances, &host.device, &[changed])
        .unwrap();
    assert!(payloads(&clips, &instances).is_empty());
    drop(clips);
    drop(instances);
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert_eq!(staging.used(), 0);
    assert_eq!(screen.used(), 0);
}

// Frozen pre-compact shader SHA256 e42ac3fe160a9e78be6e8baa22501564e751d53bb605540636cc233b177db6bb.
// The only test suffix supplies full-frame damage eligibility identically to both shaders.
const ORIGINAL_STROKE_SHADER: &str = r#"struct Screen { resolution: vec2<f32>, pad: vec2<f32> };
@group(0) @binding(0) var<uniform> screen: Screen;
struct In {
 @builtin(vertex_index) vertex: u32,
 @location(0) a: vec2<f32>, @location(1) b: vec2<f32>,
 @location(2) material_width: vec4<f32>, @location(3) clip: vec4<f32>, @location(4) contrast: vec4<f32>,
};
struct Out {
 @builtin(position) position: vec4<f32>,
 @location(0) local: vec2<f32>,
 @location(1) @interpolate(flat) length_radius: vec2<f32>,
 @location(2) @interpolate(flat) material: vec3<f32>,
 @location(3) @interpolate(flat) clip: vec4<f32>,
 @location(4) @interpolate(flat) contrast: vec2<f32>,
};
@vertex fn vs_main(v:In)->Out {
 let direction=v.b-v.a; let length=length(direction);
 let u=select(vec2<f32>(1.0,0.0),direction/max(length,0.000001),length>0.000001); let n=vec2<f32>(-u.y,u.x);
 let radius=v.material_width.w*0.5; let extent=radius+2.0;
 let corners=array<vec2<f32>,6>(vec2<f32>(-extent,-extent),vec2<f32>(length+extent,-extent),vec2<f32>(length+extent,extent),vec2<f32>(-extent,-extent),vec2<f32>(length+extent,extent),vec2<f32>(-extent,extent));
 let local=corners[v.vertex]; let p=v.a+u*local.x+n*local.y;
 var o:Out; o.position=vec4<f32>(p.x/screen.resolution.x*2.0-1.0,1.0-p.y/screen.resolution.y*2.0,0.0,1.0);
 o.local=local;o.length_radius=vec2<f32>(length,radius);o.material=v.material_width.xyz;o.clip=v.clip;o.contrast=v.contrast.xy;return o;
}
fn linear(c:vec3<f32>)->vec3<f32> {
 return select(pow((c+vec3<f32>(0.055))/1.055,vec3<f32>(2.4)),c/12.92,c<=vec3<f32>(0.04045));
}
fn eligible(v:Out)->bool {
 return datum_damaged(v.position) && v.position.x>=v.clip.x && v.position.y>=v.clip.y && v.position.x<v.clip.z && v.position.y<v.clip.w;
}
fn distance(v:Out)->f32 {return length(vec2<f32>(v.local.x-clamp(v.local.x,0.0,v.length_radius.x),v.local.y));}
@fragment fn fs_lift(v:Out,@builtin(sample_index) sample_index:u32)->@location(0) vec4<f32> {
 if (!eligible(v) || v.contrast.y>0.5 || distance(v)>v.length_radius.y) {discard;}
 return vec4<f32>(linear(min(v.material*1.08,vec3<f32>(1.0))),1.0);
}
@fragment fn fs_cue(v:Out,@builtin(sample_index) sample_index:u32)->@location(0) vec4<f32> {
 if (!eligible(v)) { discard; }
 let d=distance(v); let radius=v.length_radius.y;
 if (d>radius+2.0) {discard;}
 let accent=linear(vec3<f32>(206.0,90.0,146.0)/255.0);
 // Minimum cues have no material lift or glow. The ring remains open;
 // NoConnect retains its X rather than becoming a junction.
 if (v.contrast.y>1.5) {
   if (d>radius) {discard;} return vec4<f32>(accent,1.0);
 }
 if (v.contrast.y>0.5) {
   if (d<radius-1.0 || d>radius+1.0) {discard;}
   return vec4<f32>(accent,1.0);
 }
 if (d>=radius) {return vec4<f32>(accent,1.0);}
 // Glow is supplementary and translucent so an overlapping proposal survives.
 if (v.contrast.x>0.5 || radius-d>=2.0) {discard;}
 let opacity=0.30*(1.0-smoothstep(0.0,2.0,radius-d));
 return vec4<f32>(accent,opacity);
}
"#;

fn capsule_shader_reference(
    host: &OffscreenRenderer,
    strokes: &[crate::selection_stroke::Stroke],
    compact: bool,
) -> RgbaImage {
    use crate::text_gpu::budget::Budget;
    let device = &host.device;
    let screen = Budget::new(1024 * 1024);
    let staging = Budget::new(1024 * 1024);
    let uniform = crate::gpu_data::uniform_buffer::UniformBuffer::new(
        device,
        "reference screen",
        crate::gpu_data::ScreenUniform {
            resolution: [host.width as f32, host.height as f32],
            _pad: [0.0; 2],
        },
        &screen,
    )
    .unwrap();
    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("reference screen"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("reference screen"),
        layout: &uniform_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.buffer().as_entire_binding(),
        }],
    });
    let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("reference full damage"),
        entries: &[],
    });
    let empty_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("reference full damage"),
        layout: &empty_layout,
        entries: &[],
    });
    let mut clips =
        crate::selection_stroke::gpu::Owner::new(device, screen.clone(), staging.clone());
    let mut vertices = crate::gpu_data::screen_buffer::ScreenBuffer::with_budget(screen.clone())
        .with_staging_budget(staging.clone());
    if compact {
        clips.prepare(&mut vertices, device, strokes).unwrap();
    } else {
        vertices
            .sync(device, &host.queue, "original64 input", strokes)
            .unwrap();
    }
    let mut sources = Vec::new();
    vertices.append_sources(&mut sources);
    if compact {
        clips.gpu.append_sources(&mut sources);
    }
    let uploads = crate::text_gpu::upload::plan::Uploads::sources(&sources).collect::<Vec<_>>();
    crate::text_gpu::upload::submit_buffers_for_test(device, &host.queue, &uploads);
    drop(uploads);
    drop(sources);
    vertices.finish_uploads();
    clips.gpu.finish_uploads();
    let source = if compact {
        include_str!("selection_stroke.wgsl")
    } else {
        ORIGINAL_STROKE_SHADER
    };
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("capsule old/new parity"),
        source: wgpu::ShaderSource::Wgsl(
            format!("{source}\nfn datum_damaged(position:vec4<f32>)->bool {{ return true; }}")
                .into(),
        ),
    });
    let groups = if compact {
        vec![&uniform_layout, &empty_layout, &clips.layout]
    } else {
        vec![&uniform_layout]
    };
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("capsule old/new parity"),
        bind_group_layouts: &groups,
        immediate_size: 0,
    });
    const OLD_ATTRIBUTES: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Float32x4,3=>Float32x4,4=>Float32x4];
    const NEW_ATTRIBUTES: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2,2=>Float32x4,3=>Uint32,4=>Uint32];
    let vertex_layout = wgpu::VertexBufferLayout {
        array_stride: if compact { 40 } else { 64 },
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: if compact {
            &NEW_ATTRIBUTES
        } else {
            &OLD_ATTRIBUTES
        },
    };
    let pipeline = |entry, blend| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("capsule old/new parity"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OUTPUT_FORMAT,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        })
    };
    let lift = pipeline("fs_lift", wgpu::BlendState::REPLACE);
    let cue = pipeline("fs_cue", wgpu::BlendState::ALPHA_BLENDING);
    let target =
        crate::capture_resource::CaptureTarget::new(device, host.extent(), OUTPUT_FORMAT).unwrap();
    let view = target.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("capsule old/new parity"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, &uniform_group, &[]);
        if compact {
            pass.set_bind_group(1, &empty_group, &[]);
            pass.set_bind_group(2, clips.binding.as_ref().unwrap(), &[]);
        }
        pass.set_vertex_buffer(0, vertices.buffer().unwrap().slice(..));
        pass.set_pipeline(&lift);
        pass.draw(0..6, 0..strokes.len() as u32);
        pass.set_pipeline(&cue);
        pass.draw(0..6, 0..strokes.len() as u32);
    }
    host.queue.submit([encoder.finish()]);
    target.hold_submission(&host.queue);
    host.read_texture(&target).unwrap()
}

#[test]
#[ignore = "requires local hardware GPU: frozen original64 versus production40 shader pixels"]
fn compact_capsule_pixels_equal_frozen_original64_shader_for_all_modes_clips_and_contrasts() {
    let host = hardware_renderer(960, 720);
    let mut strokes = Vec::new();
    for mode in 0..3 {
        for contrast in 0..2 {
            let x = 70.0 + (mode * 2 + contrast) as f32 * 130.0;
            strokes.push(crate::selection_stroke::Stroke {
                from: if mode == 0 {
                    [x - 10.25, 100.5]
                } else {
                    [x + 35.25, 95.5]
                },
                to: if mode == 0 {
                    [x + 110.25, 100.5]
                } else if mode == 1 {
                    [x + 35.25, 95.5]
                } else {
                    [x + 45.25, 105.5]
                },
                material_width: [
                    0.4,
                    0.6,
                    0.2,
                    if mode == 0 {
                        20.0
                    } else if mode == 1 {
                        6.0
                    } else {
                        2.0
                    },
                ],
                clip: [x, 80.0, x + 90.0, 120.0],
                high_contrast: [contrast as f32, mode as f32, 0.0, 0.0],
            });
        }
    }
    let old = capsule_shader_reference(&host, &strokes, false);
    let compact = capsule_shader_reference(&host, &strokes, true);
    assert_eq!(
        compact, old,
        "entire frozen old64 shader footprint, including clips/modes/contrast"
    );
    assert!(compact.pixels().any(|p| p.0[..3] == [206, 90, 146]));
}
