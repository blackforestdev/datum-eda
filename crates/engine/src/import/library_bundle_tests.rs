use super::*;

fn resistor() -> (
    &'static BundleManifest,
    &'static BundleLibrary,
    &'static BundleEntry,
) {
    let catalog = catalog().unwrap();
    let library = catalog
        .libraries
        .iter()
        .find(|library| library.source == Path::new("kicad/Device.kicad_sym"))
        .unwrap();
    let entry = library
        .symbols
        .iter()
        .find(|entry| entry.name == "R")
        .unwrap();
    (catalog, library, entry)
}

#[test]
fn shipped_native_records_are_available_without_upstream_installation() {
    let (catalog, _, _) = resistor();
    assert!(catalog.converted > 0);
    assert!(catalog.complete_inventory);
    for (library, name, pins) in [
        ("Device", "R", 2),
        ("Device", "C", 2),
        ("Connector_Generic", "Conn_01x02", 2),
        ("Amplifier_Operational", "LM741", 8),
    ] {
        let source = format!("kicad/{library}.kicad_sym");
        let selected = preview(Path::new(&source), name).unwrap();
        assert_eq!(selected.source_path, Path::new(&source));
        assert_eq!(selected.source_symbol, name);
        assert_eq!(selected.symbol.pin_anchors.len(), pins);
        assert_eq!(selected.unit.pins.len(), pins);
    }
}

#[test]
fn unsupported_and_unknown_bundled_sources_return_their_actual_refusal() {
    let reason = preview(Path::new("kicad/Amplifier_Operational.kicad_sym"), "TL072")
        .unwrap_err()
        .to_string();
    assert!(reason.contains("multiunit"), "{reason}");
    assert!(preview(Path::new("/usr/share/kicad/symbols/Device.kicad_sym"), "R").is_err());
    assert!(preview(Path::new("kicad/Device.kicad_sym"), "NotASymbol").is_err());
}

#[test]
fn selected_native_record_requires_payload_digest_and_catalog_identity() {
    let (catalog, library, indexed) = resistor();
    let mut entry: BundleEntry =
        serde_json::from_value(serde_json::to_value(indexed).unwrap()).unwrap();
    let (offset, length) = match &entry.outcome {
        BundleEntryOutcome::Converted {
            offset,
            payload_bytes,
            ..
        } => (*offset, *payload_bytes),
        _ => unreachable!(),
    };
    let mut bytes = record_bytes(SYMBOLS, offset as usize, (offset + length) as usize)
        .unwrap()
        .to_vec();
    if let BundleEntryOutcome::Converted { offset, .. } = &mut entry.outcome {
        *offset = 0;
    }
    assert!(decode_record(catalog, library, &entry, &[&bytes]).is_ok());
    bytes[0] ^= 1;
    assert!(
        decode_record(catalog, library, &entry, &[&bytes])
            .unwrap_err()
            .to_string()
            .contains("digest")
    );
    bytes[0] ^= 1;
    let mut record: NativeSymbolRecord = serde_json::from_slice(&bytes).unwrap();
    record.source.symbol = "DifferentSymbol".into();
    let changed = serde_json::to_vec(&record).unwrap();
    if let BundleEntryOutcome::Converted {
        payload_bytes,
        payload_hash,
        ..
    } = &mut entry.outcome
    {
        *payload_bytes = changed.len() as u64;
        *payload_hash = compute_source_hash_bytes(&changed);
    }
    assert!(
        decode_record(catalog, library, &entry, &[&changed])
            .unwrap_err()
            .to_string()
            .contains("identity")
    );
    if let BundleEntryOutcome::Converted { offset, .. } = &mut entry.outcome {
        *offset = u64::MAX;
    }
    assert!(decode_record(catalog, library, &entry, &[&changed]).is_err());
}

#[test]
fn package_chunk_boundaries_preserve_records_without_copying_whole_package() {
    let parts: &[&[u8]] = &[b"abc", b"defg", b"hij"];
    assert!(matches!(
        record_bytes(parts, 3, 7).unwrap(),
        Cow::Borrowed(b"defg")
    ));
    assert_eq!(record_bytes(parts, 2, 8).unwrap().as_ref(), b"cdefgh");
    assert!(matches!(record_bytes(parts, 2, 8).unwrap(), Cow::Owned(_)));
    assert_eq!(record_bytes(parts, 0, 10).unwrap().as_ref(), b"abcdefghij");
    assert!(record_bytes(parts, 0, 11).is_err());
    assert!(record_bytes(parts, 5, 4).is_err());
}

#[test]
fn real_native_records_crossing_package_parts_preserve_identity_and_digest() {
    let catalog = catalog().unwrap();
    let mut boundary = 0u64;
    let boundaries: Vec<_> = SYMBOLS
        .iter()
        .take(SYMBOLS.len() - 1)
        .map(|part| {
            boundary += part.len() as u64;
            boundary
        })
        .collect();
    for library in &catalog.libraries {
        for entry in &library.symbols {
            if let BundleEntryOutcome::Converted {
                offset,
                payload_bytes,
                symbol_id,
                ..
            } = &entry.outcome
                && boundaries
                    .iter()
                    .any(|boundary| *offset < *boundary && *boundary < offset + payload_bytes)
            {
                let record = preview(&library.source, &entry.name).unwrap();
                assert_eq!(record.symbol.uuid, *symbol_id);
            }
        }
    }
}

#[test]
fn every_shipped_record_decodes_through_runtime_and_every_refusal_is_preserved() {
    let catalog = catalog().unwrap();
    let mut converted = 0u64;
    let mut refused = 0u64;
    let mut ids = std::collections::BTreeSet::new();
    for library in &catalog.libraries {
        for entry in &library.symbols {
            let result = preview(&library.source, &entry.name);
            match &entry.outcome {
                BundleEntryOutcome::Converted {
                    unit_id, symbol_id, ..
                } => {
                    let record = result.unwrap_or_else(|error| {
                        panic!("{}:{}: {error}", library.source.display(), entry.name)
                    });
                    assert_eq!(record.unit.uuid, *unit_id);
                    assert_eq!(record.symbol.uuid, *symbol_id);
                    assert_eq!(record.symbol.unit, record.unit.uuid);
                    assert!(ids.insert(*unit_id), "duplicate native Unit identity");
                    assert!(ids.insert(*symbol_id), "duplicate native Symbol identity");
                    for anchor in &record.symbol.pin_anchors {
                        assert!(
                            record.unit.pins.contains_key(&anchor.pin),
                            "{}:{} has an anchor without a logical pin",
                            library.source.display(),
                            entry.name
                        );
                    }
                    converted += 1;
                }
                BundleEntryOutcome::Refused { reason } => {
                    let error = result.unwrap_err().to_string();
                    assert!(error.contains(reason), "refusal reason changed: {error}");
                    refused += 1;
                }
            }
        }
    }
    assert_eq!(converted, catalog.converted);
    assert_eq!(refused, catalog.refused);
    assert_eq!(ids.len() as u64, converted * 2);
}
