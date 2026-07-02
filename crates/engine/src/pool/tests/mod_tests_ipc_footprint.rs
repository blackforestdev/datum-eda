use super::*;

fn chip_spec(density_level: IpcDensityLevel) -> IpcTwoTerminalChipSpec {
    IpcTwoTerminalChipSpec {
        footprint_uuid: Uuid::from_u128(0x7351),
        package_uuid: Uuid::from_u128(0x1005),
        padstack_uuid: Uuid::from_u128(0x2005),
        pad_a_uuid: Uuid::from_u128(1),
        pad_b_uuid: Uuid::from_u128(2),
        name: None,
        metric_code: "1005".to_string(),
        dimensions: IpcSourceDimensions {
            body_length_nm: 1_000_000,
            body_width_nm: 500_000,
            terminal_length_nm: 200_000,
            terminal_width_nm: 500_000,
        },
        density_level,
        mask_expansion_nm: 50_000,
        paste_reduction_nm: 50_000,
    }
}

#[test]
fn ipc7351b_two_terminal_chip_generates_structured_basis_and_geometry() {
    let generated = generate_ipc7351b_two_terminal_chip(chip_spec(IpcDensityLevel::Nominal))
        .expect("nominal chip footprint should generate");

    assert_eq!(generated.footprint.name, "CHIP-1005_IPC7351B_B");
    assert_eq!(generated.footprint.package, Uuid::from_u128(0x1005));
    assert_eq!(
        generated.footprint.standards_basis.as_deref(),
        Some("IPC-7351B two-terminal chip density B")
    );
    let basis = generated
        .footprint
        .ipc_basis
        .as_ref()
        .expect("generated footprint should carry structured IPC basis");
    assert_eq!(basis.family, "IPC-7351");
    assert_eq!(basis.revision, "B");
    assert_eq!(basis.density_level, IpcDensityLevel::Nominal);
    assert_eq!(basis.package_family, "two_terminal_chip");
    assert_eq!(basis.package_code, "1005");
    assert_eq!(basis.source_j_values.toe_nm, 350_000);
    assert_eq!(basis.source_j_values.heel_nm, 350_000);
    assert_eq!(basis.source_j_values.side_nm, 0);
    assert_eq!(basis.courtyard_excess_nm, 250_000);
    assert_eq!(basis.mask_expansion_nm, 50_000);
    assert_eq!(basis.paste_reduction_nm, 50_000);

    assert_eq!(
        generated.padstack.aperture,
        Some(PadstackAperture::Rect {
            width_nm: 900_000,
            height_nm: 500_000,
        })
    );
    assert_eq!(
        generated.padstack.mask_policy,
        PadstackMaskPolicy::ExpansionNm(50_000)
    );
    assert_eq!(
        generated.padstack.paste_policy,
        PadstackPastePolicy::ExpansionNm(-50_000)
    );

    let left = generated
        .footprint
        .pads
        .get(&Uuid::from_u128(1))
        .expect("left pad should exist");
    let right = generated
        .footprint
        .pads
        .get(&Uuid::from_u128(2))
        .expect("right pad should exist");
    assert_eq!(left.name, "1");
    assert_eq!(right.name, "2");
    assert_eq!(left.position, Point::new(-500_000, 0));
    assert_eq!(right.position, Point::new(500_000, 0));
    assert_eq!(left.padstack, generated.padstack.uuid);
    assert_eq!(right.padstack, generated.padstack.uuid);

    assert_eq!(
        generated.footprint.courtyard.vertices,
        vec![
            Point::new(-1_200_000, -500_000),
            Point::new(1_200_000, -500_000),
            Point::new(1_200_000, 500_000),
            Point::new(-1_200_000, 500_000),
        ]
    );
}

#[test]
fn ipc7351b_two_terminal_density_changes_pad_and_courtyard_policy() {
    let most = generate_ipc7351b_two_terminal_chip(chip_spec(IpcDensityLevel::Most))
        .expect("most density should generate");
    let least = generate_ipc7351b_two_terminal_chip(chip_spec(IpcDensityLevel::Least))
        .expect("least density should generate");

    assert_eq!(
        most.padstack.aperture,
        Some(PadstackAperture::Rect {
            width_nm: 1_200_000,
            height_nm: 600_000,
        })
    );
    assert_eq!(
        least.padstack.aperture,
        Some(PadstackAperture::Rect {
            width_nm: 600_000,
            height_nm: 400_000,
        })
    );
    assert_eq!(most.footprint.name, "CHIP-1005_IPC7351B_A");
    assert_eq!(least.footprint.name, "CHIP-1005_IPC7351B_C");
    assert_ne!(most.footprint.courtyard, least.footprint.courtyard);
}

#[test]
fn ipc7351b_structured_basis_round_trips_with_footprint() {
    let generated = generate_ipc7351b_two_terminal_chip(chip_spec(IpcDensityLevel::Nominal))
        .expect("nominal chip footprint should generate");

    let json =
        serde_json::to_string(&generated.footprint).expect("generated footprint should serialize");
    let decoded: Footprint =
        serde_json::from_str(&json).expect("generated footprint should deserialize");

    assert_eq!(decoded, generated.footprint);
    assert_eq!(
        decoded
            .ipc_basis
            .expect("structured basis should round-trip")
            .derivation_version,
        "datum-ipc7351b-two-terminal-chip-v1"
    );
}

#[test]
fn ipc7351b_two_terminal_rejects_invalid_source_dimensions() {
    let mut spec = chip_spec(IpcDensityLevel::Nominal);
    spec.dimensions.terminal_width_nm = 0;

    let error =
        generate_ipc7351b_two_terminal_chip(spec).expect_err("zero terminal width should fail");
    assert!(error.contains("terminal_width_nm must be positive"));
}

#[test]
fn ipc7351b_library_graph_reports_basis_and_process_policy_mismatch() {
    let generated = generate_ipc7351b_two_terminal_chip(chip_spec(IpcDensityLevel::Nominal))
        .expect("nominal chip footprint should generate");
    let mut graph = LibraryGraph::default();
    graph.packages.insert(
        generated.footprint.package,
        serde_json::json!({
            "uuid": generated.footprint.package,
            "pads": {}
        }),
    );
    let mut bad_padstack =
        serde_json::to_value(&generated.padstack).expect("padstack should serialize");
    bad_padstack
        .as_object_mut()
        .expect("padstack should be object")
        .insert(
            "mask_policy".to_string(),
            serde_json::json!({"expansion_nm": 125000}),
        );
    graph
        .padstacks
        .insert(generated.padstack.uuid, bad_padstack);
    let mut bad_footprint =
        serde_json::to_value(&generated.footprint).expect("footprint should serialize");
    bad_footprint
        .as_object_mut()
        .expect("footprint should be object")
        .insert(
            "standards_basis".to_string(),
            serde_json::json!("not the IPC naming basis"),
        );
    graph
        .footprints
        .insert(generated.footprint.uuid, bad_footprint);
    graph.subjects.insert(
        generated.footprint.uuid,
        "pool/footprints/ipc.json".to_string(),
    );

    let codes = graph
        .dependency_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(codes.contains("ipc_basis_standards_mismatch"));
    assert!(codes.contains("ipc_basis_process_policy_mismatch"));
}
