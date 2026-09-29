//! Exhaustive byte/sample oracle for the approved UNORM restoration primitive.
//! Scratch textures are oracle fixtures, not candidate resource-budget evidence.
use super::Restoration;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 2;
const BYTES: u64 = (WIDTH * HEIGHT * 8 * 4) as u64;

fn expected(x: u32, y: u32, sample: u32) -> u32 {
    let channels = [
        x,
        (x + sample * 31 + y * 7) % 256,
        255 - x,
        (x * 3 + sample * 17) % 256,
    ];
    channels[0] | channels[1] << 8 | channels[2] << 16 | channels[3] << 24
}

fn texture(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("r3-sample-oracle-image"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 8,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[format.remove_srgb_suffix()],
    })
}

fn initialize(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    format: wgpu::TextureFormat,
) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("r3-asymmetric-sample-pattern"),
        source: wgpu::ShaderSource::Wgsl(r#"
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(p[i], 0.0, 1.0);
}
@fragment fn fs(@builtin(position) p: vec4<f32>, @builtin(sample_index) s: u32) -> @location(0) vec4<f32> {
    let x = u32(p.x); let y = u32(p.y);
    return vec4<f32>(f32(x), f32((x + s * 31u + y * 7u) % 256u), f32(255u - x), f32((x * 3u + s * 17u) % 256u)) / 255.0;
}
"#.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("r3-sample-pattern-pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 8,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    });
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("r3-write-known-samples"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    pass.set_pipeline(&pipeline);
    pass.draw(0..3, 0..1);
}

fn samples(device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView) -> Vec<u32> {
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("r3-sample-oracle-output"),
        size: BYTES,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("r3-sample-oracle-readback"),
        size: BYTES,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("r3-read-every-sample"),
        source: wgpu::ShaderSource::Wgsl(r#"
@group(0) @binding(0) var image: texture_multisampled_2d<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@compute @workgroup_size(8, 1, 1) fn main(@builtin(global_invocation_id) p: vec3<u32>) {
    let pixel = p.x / 8u; let sample = p.x % 8u;
    output[p.x] = pack4x8unorm(textureLoad(image, vec2<i32>(i32(pixel % 256u), i32(pixel / 256u)), i32(sample)));
}
"#.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("r3-read-samples-pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.dispatch_workgroups(WIDTH * HEIGHT, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, BYTES);
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap()
        });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = readback.slice(..).get_mapped_range();
    let result = bytemuck::cast_slice::<u8, u32>(&mapped).to_vec();
    drop(mapped);
    readback.unmap();
    result
}

pub(crate) fn prove(device: &wgpu::Device, queue: &wgpu::Queue, adapter: &wgpu::Adapter) {
    for format in [
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    ] {
        let alias = format.remove_srgb_suffix();
        let source = texture(device, format);
        let destination = texture(device, format);
        let source_view = source.create_view(&wgpu::TextureViewDescriptor {
            format: Some(alias),
            ..Default::default()
        });
        let destination_view = destination.create_view(&wgpu::TextureViewDescriptor {
            format: Some(alias),
            ..Default::default()
        });
        let restoration = pollster::block_on(Restoration::admit(device, adapter, format, 8))
            .expect("P630 restoration capability required");
        let mut encoder = device.create_command_encoder(&Default::default());
        initialize(device, &mut encoder, &source_view, alias);
        // Distinct destination state proves unscissored pixels remain untouched.
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &destination_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        restoration.encode(
            device,
            &mut encoder,
            &source_view,
            &destination_view,
            &[[0, 0, 129, 1], [128, 0, 256, 1], [255, 1, 256, 2]],
            None,
        );
        queue.submit([encoder.finish()]);
        let actual = samples(device, queue, &destination_view);
        let original = samples(device, queue, &source_view);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                for sample in 0..8 {
                    let index = ((y * WIDTH + x) * 8 + sample) as usize;
                    assert_eq!(
                        original[index],
                        expected(x, y, sample),
                        "source pattern {format:?} {x}/{y}/{sample}"
                    );
                    let wanted = if y == 0 || x == 255 {
                        expected(x, y, sample)
                    } else {
                        0
                    };
                    assert_eq!(
                        actual[index], wanted,
                        "restored {format:?} {x}/{y}/{sample}"
                    );
                }
            }
        }
        let wrong_shader = include_str!("gpu_damage_restore.wgsl").replace("i32(sample)", "0");
        let wrong = pollster::block_on(Restoration::create(device, alias, &wrong_shader)).unwrap();
        let mut encoder = device.create_command_encoder(&Default::default());
        wrong.encode(
            device,
            &mut encoder,
            &source_view,
            &destination_view,
            &[[0, 0, WIDTH, HEIGHT]],
            None,
        );
        queue.submit([encoder.finish()]);
        let faulty = samples(device, queue, &destination_view);
        assert!(
            faulty != original,
            "oracle must reject sample-zero substitution"
        );
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let wrong_view = destination.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        restoration.encode(
            device,
            &mut encoder,
            &source_view,
            &wrong_view,
            &[[0, 0, WIDTH, HEIGHT]],
            None,
        );
        let _ = encoder.finish();
        assert!(
            pollster::block_on(scope.pop()).is_some(),
            "sRGB destination must fail pipeline format validation"
        );
        eprintln!(
            "sample oracle passed {format:?}: all256 channel values/all8 samples/alpha/order/overlap/edges; wrong sample and sRGB view rejected"
        );
    }
}
