//! Pixel oracle against the installed renderer, for the Datum-owned default path.
use super::{atlas::Atlas, draw::Draw};
use glyphon::{
    Buffer, Cache, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport,
};

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn owned_draw_matches_installed_text_renderer() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut fonts = crate::load_datum_fonts();
    let mut raster = SwashCache::new();
    let mut buffer = Buffer::new(&mut fonts, Metrics::new(18.0, 18.0 * 1.22));
    buffer.set_size(&mut fonts, Some(200.0), Some(128.0));
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
    viewport.update(
        &queue,
        Resolution {
            width: 256,
            height: 128,
        },
    );
    let mut baseline_atlas = TextAtlas::new(&device, &queue, &cache, format);
    for samples in [1, 4] {
        let mut baseline = TextRenderer::new(
            &mut baseline_atlas,
            &device,
            wgpu::MultisampleState {
                count: samples,
                ..Default::default()
            },
            None,
        );
        let mut atlas = Atlas::new(&device);
        let screen_budget = super::budget::Budget::new(16 * 1024 * 1024);
        let mut draw = Draw::new(&device, &atlas, format, samples, screen_budget.clone());
        for scale in [1.0, 1.25, 2.0] {
            for face in [crate::TextFace::Ui, crate::TextFace::Terminal] {
                for (text, left, top, bounds) in [
                    ("rich", 9.5, 6.25, TextBounds::default()),
                    ("😀🌍", 12.0, 8.0, TextBounds::default()),
                    ("Datum · µm Ω\nHello", 12.0, 8.0, TextBounds::default()),
                    ("e\u{301} — 🛠 العربية", 12.25, 8.5, TextBounds::default()),
                    (
                        "Clipped text\nsecond row",
                        -3.0,
                        -9.0,
                        TextBounds {
                            left: 5,
                            top: 2,
                            right: 125,
                            bottom: 35,
                        },
                    ),
                ] {
                    buffer.set_text(
                        &mut fonts,
                        text,
                        &if text == "😀🌍" {
                            glyphon::Attrs::new().family(glyphon::Family::Name("Noto Color Emoji"))
                        } else {
                            crate::text_attrs(face)
                        },
                        Shaping::Basic,
                        None,
                    );
                    if text == "rich" {
                        let attrs = crate::text_attrs(face);
                        buffer.set_rich_text(
                            &mut fonts,
                            [
                                (
                                    "Bold ",
                                    attrs
                                        .clone()
                                        .weight(glyphon::Weight::BOLD)
                                        .color(glyphon::Color::rgb(240, 70, 30)),
                                ),
                                (
                                    "Italic",
                                    attrs
                                        .clone()
                                        .style(glyphon::Style::Italic)
                                        .color(glyphon::Color::rgb(30, 190, 230)),
                                ),
                            ],
                            &attrs,
                            Shaping::Basic,
                            None,
                        );
                    }
                    buffer.shape_until_scroll(&mut fonts, false);
                    let mut run = crate::TextRun {
                        text: text.into(),
                        rich_spans: Vec::new(),
                        x: left,
                        y: top,
                        size: 18.0,
                        color: [1.0; 3],
                        face,
                        clip_bounds: None,
                        layout_size: None,
                    };
                    if text == "rich" {
                        run.rich_spans = vec![
                            crate::TextRunSpan {
                                text: "Bold ".into(),
                                color: [240.0 / 255.0, 70.0 / 255.0, 30.0 / 255.0],
                                bold: true,
                                italic: false,
                            },
                            crate::TextRunSpan {
                                text: "Italic".into(),
                                color: [30.0 / 255.0, 190.0 / 255.0, 230.0 / 255.0],
                                bold: false,
                                italic: true,
                            },
                        ];
                    }
                    let layout = if text == "😀🌍" {
                        crate::text_layout::TextLayout::with_test_attrs(
                            &mut fonts,
                            &mut glyphon::cosmic_text::ShapeBuffer::default(),
                            &run,
                            (200, 128),
                            &glyphon::Attrs::new()
                                .family(glyphon::Family::Name("Noto Color Emoji")),
                        )
                    } else {
                        crate::text_layout::TextLayout::new(
                            &mut fonts,
                            &mut glyphon::cosmic_text::ShapeBuffer::default(),
                            &run,
                            (200, 128),
                        )
                    };
                    let area = TextArea {
                        buffer: &buffer,
                        left,
                        top,
                        scale,
                        bounds,
                        default_color: glyphon::Color::rgb(217, 163, 87),
                        custom_glyphs: &[],
                    };
                    let areas = [
                        area.clone(),
                        TextArea {
                            left: left + 10.0,
                            top: top + 4.0,
                            default_color: glyphon::Color::rgba(50, 190, 250, 127),
                            ..area
                        },
                    ];
                    baseline
                        .prepare(
                            &device,
                            &queue,
                            &mut fonts,
                            &mut baseline_atlas,
                            &viewport,
                            areas.clone(),
                            &mut raster,
                        )
                        .unwrap();
                    draw.prepare(
                        &device,
                        &queue,
                        &mut atlas,
                        &mut fonts,
                        &mut raster,
                        [256, 128],
                        areas.iter().map(|area| owned_area(area, &layout)),
                    )
                    .unwrap();
                    atlas.flush_uploads(&queue);
                    assert_eq!(
                        screen_budget.used(),
                        atlas
                            .owner
                            .records()
                            .iter()
                            .filter(|r| r.kind == super::lifetime::Kind::Instances)
                            .map(|r| r.bytes)
                            .sum::<u64>()
                    );
                    draw.flush_uploads(&queue);
                    if text == "😀🌍" {
                        assert!(
                            atlas
                                .pages
                                .iter()
                                .any(|page| page.texture.format()
                                    == wgpu::TextureFormat::Rgba8UnormSrgb),
                            "color raster path must actually be exercised"
                        );
                    }
                    let expected = pixels(&device, &queue, format, samples, |pass| {
                        baseline.render(&baseline_atlas, &viewport, pass).unwrap()
                    });
                    let actual = pixels(&device, &queue, format, samples, |pass| {
                        draw.render(&atlas, pass).unwrap()
                    });
                    let allocation = atlas
                        .owner
                        .records()
                        .into_iter()
                        .find(|record| record.kind == super::lifetime::Kind::Instances)
                        .unwrap()
                        .id;
                    let uploads = atlas.uploads;
                    draw.prepare(
                        &device,
                        &queue,
                        &mut atlas,
                        &mut fonts,
                        &mut raster,
                        [256, 128],
                        areas.iter().map(|area| owned_area(area, &layout)),
                    )
                    .unwrap();
                    atlas.flush_uploads(&queue);
                    assert_eq!(
                        screen_budget.used(),
                        atlas
                            .owner
                            .records()
                            .iter()
                            .filter(|r| r.kind == super::lifetime::Kind::Instances)
                            .map(|r| r.bytes)
                            .sum::<u64>()
                    );
                    draw.flush_uploads(&queue);
                    assert_eq!(draw.upload_bytes, 0, "unchanged instances must not upload");
                    assert_eq!(
                        atlas.uploads, uploads,
                        "unchanged glyphs must not rasterize/upload"
                    );
                    assert_eq!(
                        atlas
                            .owner
                            .records()
                            .into_iter()
                            .find(|record| record.kind == super::lifetime::Kind::Instances)
                            .unwrap()
                            .id,
                        allocation,
                        "unchanged preparation must reuse its instance allocation"
                    );
                    let differing = actual
                        .chunks_exact(4)
                        .zip(expected.chunks_exact(4))
                        .filter(|(a, b)| a != b)
                        .count();
                    if differing != 0 {
                        for (name, bytes) in [("actual", &actual), ("expected", &expected)] {
                            let colored: Vec<_> = bytes
                                .chunks_exact(4)
                                .enumerate()
                                .filter(|(_, p)| *p != &bytes[..4])
                                .map(|(i, _)| (i % 256, i / 256))
                                .collect();
                            eprintln!(
                                "{name} count={} x={:?}..{:?} y={:?}..{:?}",
                                colored.len(),
                                colored.iter().map(|p| p.0).min(),
                                colored.iter().map(|p| p.0).max(),
                                colored.iter().map(|p| p.1).min(),
                                colored.iter().map(|p| p.1).max()
                            );
                        }
                        for (index, (a, b)) in actual
                            .chunks_exact(4)
                            .zip(expected.chunks_exact(4))
                            .enumerate()
                            .filter(|(_, (a, b))| a != b)
                            .take(12)
                        {
                            eprintln!(
                                "pixel {},{} actual={a:?} expected={b:?}",
                                index % 256,
                                index / 256
                            );
                        }
                    }
                    assert_eq!(
                        differing, 0,
                        "candidate differs for {text:?} at {left},{top}, scale={scale}, samples={samples}, face={face:?}"
                    );
                }
            }
        }
    }
}

fn pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    samples: u32,
    render: impl FnOnce(&mut wgpu::RenderPass<'_>),
) -> Vec<u8> {
    let extent = wgpu::Extent3d {
        width: 256,
        height: 128,
        depth_or_array_layers: 1,
    };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("text-parity"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let multisampled = (samples > 1).then(|| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("text-parity-msaa"),
                size: extent,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("text-parity-readback"),
        size: 256 * 128 * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("text-parity"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: multisampled.as_ref().unwrap_or(&view),
                depth_slice: None,
                resolve_target: multisampled.as_ref().map(|_| &view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.03,
                        b: 0.04,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        render(&mut pass);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: Some(128),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap()
        });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    let bytes = readback.slice(..).get_mapped_range().to_vec();
    readback.unmap();
    bytes
}

fn owned_area<'a>(
    area: &TextArea<'_>,
    layout: &'a crate::text_layout::TextLayout,
) -> super::Area<crate::text_layout::Runs<'a>> {
    super::Area {
        rows: layout.layout_runs(),
        left: area.left,
        top: area.top,
        scale: area.scale,
        bounds: area.bounds,
        default_color: area.default_color,
    }
}
