use super::*;

#[global_allocator]
static TEST_ALLOCATOR: Allocator = Allocator;

#[test]
fn vector_capacity_growth_shrink_and_cross_thread_release_reconcile() {
    let scope = Scope::new("allocator-vector-proof");
    let mut data = scope.with(|| Vec::<u8>::with_capacity(17));
    assert!(installed());
    assert_eq!(scope.usage().payload_bytes, data.capacity() as u64);
    data.extend_from_slice(b"text");
    data.reserve_exact(4096);
    assert_eq!(scope.usage().payload_bytes, data.capacity() as u64);
    assert_eq!(scope.usage().allocations, 1);
    let peak = scope.usage().peak_payload_bytes;
    data.shrink_to_fit();
    assert_eq!(scope.usage().payload_bytes, 4);
    assert_eq!(scope.usage().peak_payload_bytes, peak);
    std::thread::spawn(move || drop(data)).join().unwrap();
    assert_eq!(scope.usage().payload_bytes, 0);
    assert_eq!(scope.usage().tracking_bytes, 0);
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn nested_scope_and_panicking_work_restore_attribution() {
    let first = Scope::new("first-proof");
    let second = Scope::new("second-proof");
    let (a, b, c) = first.with(|| {
        let a = vec![7_u8; 11];
        let b = second.with(|| vec![8_u8; 23]);
        let c = vec![9_u8; 31];
        (a, b, c)
    });
    assert_eq!(first.usage().payload_bytes, 42);
    assert_eq!(second.usage().payload_bytes, 23);
    let panic_scope = Scope::new("panic-output-proof");
    let _ = std::panic::catch_unwind(|| panic_scope.with(|| panic!("scope unwind proof")));
    let before = first.usage().payload_bytes;
    let unrelated = vec![0_u8; 97];
    assert_eq!(first.usage().payload_bytes, before);
    drop((a, b, c, unrelated));
    assert_eq!(first.usage().payload_bytes, 0);
    assert_eq!(second.usage().payload_bytes, 0);
}

#[test]
fn large_alignment_zeroing_and_reallocation_preserve_payload() {
    let scope = Scope::new("aligned-proof");
    let layout = Layout::from_size_align(37, 4096).unwrap();
    // SAFETY: every pointer is allocated/reallocated/deallocated with its exact
    // original layout and is accessed only within the initialized payload extent.
    unsafe {
        let impossible = Layout::from_size_align(isize::MAX as usize, 1).unwrap();
        assert!(scope.with(|| Allocator.alloc(impossible)).is_null());
        assert_eq!(scope.usage().payload_bytes, 0);
        let p = scope.with(|| Allocator.alloc_zeroed(layout));
        assert!(!p.is_null());
        assert_eq!(p as usize % 4096, 0);
        assert_eq!(std::slice::from_raw_parts(p, 37), &[0; 37]);
        p.write(91);
        let largest_valid = (isize::MAX as usize) & !(layout.align() - 1);
        assert!(Allocator.realloc(p, layout, largest_valid).is_null());
        assert_eq!(scope.usage().payload_bytes, 37);
        assert_eq!(p.read(), 91);
        let p = Allocator.realloc(p, layout, 9000);
        assert!(!p.is_null());
        assert_eq!(p as usize % 4096, 0);
        assert_eq!(p.read(), 91);
        assert_eq!(scope.usage().payload_bytes, 9000);
        assert_eq!(scope.usage().tracking_bytes, 4096);
        Allocator.dealloc(p, Layout::from_size_align(9000, 4096).unwrap());
    }
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn allocations_keep_observation_alive_after_scope_owner_closes() {
    let scope = Scope::new("closed-owner-proof");
    let id = scope.usage().owner_id;
    let payload = scope.with(|| vec![0_u8; 51]);
    drop(scope);
    assert_eq!(
        usage()
            .iter()
            .find(|o| o.owner_id == id)
            .unwrap()
            .payload_bytes,
        51
    );
    drop(payload);
    assert!(!usage().iter().any(|o| o.owner_id == id));
}

#[test]
fn private_font_shape_and_raster_allocations_retire_with_their_owners() {
    let scope = Scope::new("private-text-proof");
    let (mut fonts, mut scratch, mut raster) = scope.with(|| {
        (
            crate::load_datum_fonts(),
            glyphon::cosmic_text::ShapeBuffer::default(),
            glyphon::SwashCache::new(),
        )
    });
    let before = scope.usage().payload_bytes;
    scope.with(|| {
        let shape = glyphon::ShapeLine::new(
            &mut fonts,
            "Datum private text",
            &glyphon::AttrsList::new(&crate::text_attrs(crate::TextFace::Ui)),
            glyphon::Shaping::Basic,
            8,
        );
        let mut lines = Vec::new();
        shape.layout_to_buffer(
            &mut scratch,
            18.0,
            Some(300.0),
            glyphon::Wrap::WordOrGlyph,
            None,
            &mut lines,
            None,
        );
        let key = lines[0].glyphs[0].physical((0.0, 0.0), 1.0).cache_key;
        let image = raster.get_image_uncached(&mut fonts, key).unwrap();
        assert!(!image.data.is_empty());
    });
    assert!(scope.usage().peak_payload_bytes > before);
    assert!(scope.usage().payload_bytes > 0);
    drop((fonts, scratch, raster));
    assert_eq!(scope.usage().payload_bytes, 0);
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn sharing_and_equal_size_replacement_count_allocations_once() {
    let scope = Scope::new("sharing-proof");
    let (mut first, shared) = scope.with(|| (vec![1_u8; 64], Arc::new(vec![2_u8; 32])));
    let baseline = scope.usage().payload_bytes;
    let clone = shared.clone();
    let weak = Arc::downgrade(&shared);
    assert_eq!(scope.usage().payload_bytes, baseline);
    let old = std::mem::replace(&mut first, scope.with(|| vec![3_u8; 64]));
    assert_ne!(old, first);
    assert_eq!(scope.usage().payload_bytes, baseline + 64);
    drop(old);
    assert_eq!(scope.usage().payload_bytes, baseline);
    drop((first, shared, clone));
    assert_eq!(scope.usage().payload_bytes, baseline - 64 - 32);
    assert_eq!(
        scope.usage().allocations,
        1,
        "weak Arc still owns its allocation header"
    );
    drop(weak);
    assert_eq!(scope.usage().payload_bytes, 0);
}
