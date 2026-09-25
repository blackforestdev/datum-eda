use super::*;

#[test]
fn released_transient_peak_rejects_without_using_lifetime_high_water() {
    let scope = Scope::new("private-call-transient");
    let host = Budget::new(512);
    let process = Budget::new(1024);
    // An old lifetime high-water must not contaminate the next call.
    drop(scope.with(|| vec![0_u8; 2048]));
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    let payload = scope.with(|| vec![0_u8; 128]);
    let exact = scope.usage().payload_bytes + scope.usage().tracking_bytes;
    let (permits, report) = call.finish(exact, None, None).unwrap();
    assert_eq!(report.peak_bytes, exact);
    assert_eq!(host.used(), exact);
    drop(payload);
    drop(permits);
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, 0).unwrap();
    drop(scope.with(|| vec![0_u8; 600]));
    let error = call.finish(0, None, None).err().unwrap();
    let report = error.downcast_ref::<Overrun>().unwrap().0;
    assert!(report.peak_bytes >= 600);
    assert_eq!(report.final_bytes, 0);
    assert!(report.exceeded);
    assert_eq!(host.used(), 0);
    assert_eq!(process.used(), 0);
}

#[test]
fn concurrent_hosts_observe_simultaneous_process_peak_and_rollback() {
    let process = Budget::new(1000);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let process = process.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let scope = Scope::new("private-call-concurrent");
                let host = Budget::new(1000);
                let call = Call::begin(&scope, host.clone(), process, 0, 0).unwrap();
                let payload = scope.with(|| vec![0_u8; 600]);
                barrier.wait();
                barrier.wait();
                drop(payload);
                let error = call.finish(0, None, None).err().unwrap();
                let report = error.downcast_ref::<Overrun>().unwrap().0;
                assert!(report.process_peak_bytes >= 1200);
                assert!(report.host_peak_bytes < 1000);
                assert_eq!(host.used(), 0);
            })
        })
        .collect();
    barrier.wait();
    assert!(
        process.reserve(1).is_err(),
        "admission observes live private calls"
    );
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(process.used(), 0);
}

#[test]
fn cache_and_scratch_credits_transfer_without_double_counting() {
    let scope = Scope::new("private-call-transfer");
    let host = Budget::new(1024);
    let process = Budget::new(1024);
    let payload = scope.with(|| vec![0_u8; 400]);
    let exact = scope.usage().payload_bytes + scope.usage().tracking_bytes;
    let old = [
        host.reserve(exact).unwrap(),
        process.reserve(exact).unwrap(),
    ];
    let scratch = [host.reserve(500).unwrap(), process.reserve(500).unwrap()];
    let call = Call::begin(&scope, host.clone(), process.clone(), 0, exact + 500).unwrap();
    assert_eq!(host.available(), 1024 - exact - 500);
    drop(scope.with(|| vec![0_u8; 300]));
    let (permits, report) = call.finish(exact, Some(old), Some(scratch)).unwrap();
    assert!(report.peak_bytes < 800);
    assert_eq!(
        report.host_peak_bytes,
        exact + 500,
        "pre-call scratch capacity remains reserved until transfer"
    );
    assert_eq!(host.used(), exact);
    assert_eq!(process.used(), exact);
    drop(payload);
    drop(permits);
    assert_eq!(host.used(), 0);
}

#[test]
fn default_font_loading_overrun_preserves_shapes_and_retries_identically() {
    use crate::text_layout::fonts::{Fonts, Source};
    let host = Budget::new(16 * 1024 * 1024);
    let mut fonts = Fonts::new(host.clone()).unwrap();
    let attrs = glyphon::AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
    let text = "Preserve source and glyphs through private construction refusal";
    let expected = fonts.shape(text, &attrs).unwrap();
    let glyphs = format!("{:?}", *expected);
    fonts.release_for(host.available() + 1);
    let mut catalog = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut catalog, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut catalog,
        "A",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut catalog, false);
    let key = buffer.layout_runs().next().unwrap().glyphs[0]
        .physical((0.0, 0.0), 1.0)
        .cache_key;
    let mut raster = glyphon::SwashCache::new();
    let scope = Scope::new("font-overrun-raster");
    let filler = host.reserve(host.available()).unwrap();
    let error = fonts
        .raster(&mut raster, &scope, key)
        .err()
        .expect("private font load must refuse");
    assert!(error.downcast_ref::<Overrun>().is_some(), "{error}");
    assert!(fonts.usage().last_call.unwrap().exceeded);
    assert_eq!(format!("{:?}", *expected), glyphs);
    assert_eq!(fonts.reserved_bytes(), 0);
    drop(filler);
    let image = fonts.raster(&mut raster, &scope, key).unwrap().unwrap();
    let expected_image = raster.get_image_uncached(&mut catalog, key).unwrap();
    assert_eq!(image.data, expected_image.data);
    let actual = fonts.shape(text, &attrs).unwrap();
    assert_eq!(format!("{:?}", *actual), glyphs);
    assert!(!fonts.usage().last_call.unwrap().exceeded);
}

#[test]
fn failed_font_call_never_turns_into_an_absent_raster_glyph() {
    use crate::text_layout::fonts::{Fonts, Source};
    struct Refusing;
    impl Source for Refusing {
        fn shape(
            &mut self,
            _: &str,
            _: &glyphon::AttrsList,
        ) -> anyhow::Result<crate::text_layout::fonts::Shape> {
            unreachable!()
        }
        fn raster(
            &mut self,
            _: &mut glyphon::SwashCache,
            _: &Scope,
            _: glyphon::CacheKey,
        ) -> anyhow::Result<Option<glyphon::SwashImage>> {
            Err(Overrun(Report {
                exceeded: true,
                ..Report::default()
            })
            .into())
        }
    }
    let mut catalog = crate::load_datum_fonts();
    let mut buffer = glyphon::Buffer::new(&mut catalog, glyphon::Metrics::new(18.0, 22.0));
    buffer.set_text(
        &mut catalog,
        "A",
        &crate::text_attrs(crate::TextFace::Ui),
        glyphon::Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut catalog, false);
    let key = buffer.layout_runs().next().unwrap().glyphs[0]
        .physical((0.0, 0.0), 1.0)
        .cache_key;
    let host = Budget::new(16 * 1024 * 1024);
    let mut raster = crate::text_gpu::raster::Raster::new();
    assert!(
        raster
            .image(&mut Refusing, key, &host)
            .err()
            .unwrap()
            .downcast_ref::<Overrun>()
            .is_some()
    );
    assert_eq!(host.used(), 0);
    let mut fonts = Fonts::new(host.clone()).unwrap();
    let (_, actual) = raster.image(&mut fonts, key, &host).unwrap().unwrap();
    let (_, expected) = crate::text_gpu::raster::Raster::new()
        .image(&mut catalog, key, &host)
        .unwrap()
        .unwrap();
    assert_eq!(actual.as_slice(), expected.as_slice());
    drop((actual, expected, raster, fonts));
    assert_eq!(host.used(), 0);
}
