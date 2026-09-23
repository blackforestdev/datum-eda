use super::*;

fn prepare(
    draw: &mut Draw,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: &mut Atlas,
    fonts: &mut FontSystem,
    raster: &mut SwashCache,
    text: &str,
) -> anyhow::Result<()> {
    let mut buffer = glyphon::Buffer::new(fonts, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        fonts,
        text,
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(fonts, false);
    draw.prepare(
        device,
        queue,
        atlas,
        fonts,
        raster,
        [4096, 128],
        [Area {
            rich_spans: &[],
            rows: buffer.layout_runs(),
            left: 0.0,
            top: 0.0,
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgb(255, 255, 255),
        }],
    )
}

#[test]
#[ignore = "requires local GPU; glyph instance generation admission"]
fn instance_generation_limits_survive_empty_preparation_and_device_replacement() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut atlas = Atlas::new(&device);
    let screen = super::super::budget::Budget::new(16 * 1024 * 1024);
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut draw = Draw::new(&device, &atlas, format, 1, screen.clone());
    let generations = draw.generation_budget.clone();
    let mut fonts = crate::load_datum_fonts();
    let mut raster = SwashCache::new();
    macro_rules! prepare_text {
        ($text:expr) => {
            prepare(
                &mut draw,
                &device,
                &queue,
                &mut atlas,
                &mut fonts,
                &mut raster,
                $text,
            )
        };
    }
    prepare_text!("A").unwrap();
    draw.flush_uploads(&device, &queue);
    let first = draw.submission_ref().unwrap();
    prepare_text!("AAAAAAAA").unwrap();
    draw.flush_uploads(&device, &queue);
    let second = draw.submission_ref().unwrap();
    let current = draw.instances.as_ref().unwrap().id();
    let bytes = screen.used();
    assert_eq!(generations.used(), 2);
    let third_text = "A".repeat(32);
    assert!(
        prepare_text!(&third_text)
            .unwrap_err()
            .to_string()
            .contains("two live GPU allocations")
    );
    assert_eq!(screen.used(), bytes);
    assert_eq!(draw.instances.as_ref().unwrap().id(), current);
    assert!(!draw.has_pending_uploads());
    prepare_text!("AAAAAAAA").unwrap();
    draw.flush_uploads(&device, &queue);
    assert_eq!(
        draw.upload_bytes, 0,
        "warm reuse still succeeds at two generations"
    );
    prepare_text!("").unwrap();
    assert!(draw.instances.is_none());
    draw = draw.replacement(&device, &atlas, format, 1);
    assert!(prepare_text!(&third_text).is_err());
    assert_eq!(screen.used(), bytes);
    assert!(draw.instances.is_none());
    drop(first);
    prepare_text!(&third_text).unwrap();
    assert_eq!(generations.used(), 2);
    draw.flush_uploads(&device, &queue);
    drop(second);
    drop(draw);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(generations.used(), 0);
    assert_eq!(screen.used(), 0);
}
