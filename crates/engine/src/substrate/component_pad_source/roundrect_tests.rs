use super::*;

fn pin(id: Uuid) -> PinnedPadLibrarySource {
    PinnedPadLibrarySource {
        object_id: id,
        owner_id: id,
        source_shard_id: id,
        relative_path: "pool/footprints/rounded.json".into(),
        object_revision: ObjectRevision(1),
        owner_value_hash: "owner".into(),
        record_hash: "record".into(),
        incarnation: None,
        journal_tip: None,
    }
}

#[test]
fn rounded_frozen_geometry_rejects_corner_ratio_tampering_after_reload() {
    let land = pin(Uuid::new_v4());
    let mut library_pad = land.clone();
    library_pad.object_id = Uuid::new_v4();
    let stack = pin(Uuid::new_v4());
    let pad = ComponentPadSource {
        uuid: Uuid::new_v4(),
        name: "1".into(),
        position: Point::zero(),
        padstack: stack.object_id,
        layer: 1,
        drill_nm: None,
        shape: Some(PadShape::RoundRect),
        diameter_nm: 0,
        width_nm: 800000,
        height_nm: 950000,
        roundrect_rratio_ppm: Some(200000),
        source_binding: Some(ComponentPadSourceBinding {
            project_id: Uuid::new_v4(),
            board_id: Uuid::new_v4(),
            package_id: Uuid::new_v4(),
            materialized_against: ModelRevision("test".into()),
            part: pin(Uuid::new_v4()),
            package: pin(Uuid::new_v4()),
            land_pattern: land,
            library_pad,
            padstack: stack,
            component_instance: None,
            pin_pad_map: None,
            frozen_geometry: FrozenPadGeometry {
                name: "1".into(),
                position: Point::zero(),
                layer: 1,
                aperture: PadstackAperture::RoundRect {
                    width_nm: 800000,
                    height_nm: 950000,
                    rratio_ppm: 200000,
                },
                drill_nm: None,
                layer_span: PadstackLayerSpan::PadLayer,
                plated: None,
            },
            process_snapshot: None,
        }),
    };
    // This checks the frozen record contract, not live library-pin validity.
    pad.validate_frozen().unwrap();
    let mut loaded: ComponentPadSource =
        serde_json::from_value(serde_json::to_value(&pad).unwrap()).unwrap();
    assert_eq!(loaded, pad);
    loaded.validate_frozen().unwrap();
    for ratio in [None, Some(250000), Some(500001)] {
        loaded.roundrect_rratio_ppm = ratio;
        assert!(
            loaded.validate_frozen().is_err(),
            "ratio {ratio:?} must not replace frozen corners"
        );
    }
}
