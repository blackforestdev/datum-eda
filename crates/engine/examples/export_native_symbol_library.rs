//! Maintenance exporter, never part of application startup or the build process.
use eda_engine::import::{
    ids_sidecar::compute_source_hash_bytes,
    kicad_library::{batch, bundle_format::*},
};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut options = BTreeMap::new();
    while let Some(key) = arguments.next() {
        if !matches!(
            key.as_str(),
            "--source" | "--source-version" | "--report" | "--native-output"
        ) {
            return Err(format!("Unknown option {key}; use --source DIR --source-version VERSION --report PATH [--native-output PATH]").into());
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("Missing value for {key}"))?;
        if options.insert(key.clone(), value).is_some() {
            return Err(format!("Duplicate option {key}").into());
        }
    }
    let required = |key: &str| {
        options
            .get(key)
            .cloned()
            .ok_or_else(|| format!("Missing {key}"))
    };
    let root = PathBuf::from(required("--source")?);
    let version = required("--source-version")?;
    let report = PathBuf::from(required("--report")?);
    let mut output = options
        .get("--native-output")
        .map(|path| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map(BufWriter::new)
        })
        .transpose()?;
    let mut files = std::fs::read_dir(&root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|path| path.extension().is_some_and(|ext| ext == "kicad_sym"));
    files.sort();
    if files.is_empty() {
        return Err("No .kicad_sym source libraries found".into());
    }
    let mut libraries = Vec::new();
    let mut converted = 0u64;
    let mut refused = 0u64;
    let mut bytes = 0u64;
    let mut failures = 0u64;
    for path in files {
        let library = match batch::convert_library(&path) {
            Ok(library) => library,
            Err(error) => {
                failures += 1;
                libraries.push(BundleLibrary {
                    source: PathBuf::from("kicad").join(path.file_name().unwrap()),
                    source_hash: std::fs::read(&path)
                        .ok()
                        .map(|data| compute_source_hash_bytes(&data)),
                    format_version: None,
                    failure: Some(error.to_string()),
                    symbols: Vec::new(),
                });
                continue;
            }
        };
        let mut entries = Vec::new();
        for symbol in library.symbols {
            let outcome = match symbol.result {
                Ok(converted_symbol) => {
                    let record = NativeSymbolRecord {
                        schema_version: 1,
                        unit: converted_symbol.unit,
                        symbol: converted_symbol.symbol,
                        source: BundleSource {
                            library: library.source.clone(),
                            symbol: converted_symbol.source_symbol,
                            hash: library.source_hash.clone(),
                            version: version.clone(),
                        },
                        notices: converted_symbol.warnings,
                    };
                    // Value objects use ordered maps: no HashMap serialization
                    // order or machine-local absolute path enters the pack.
                    let encoded = serde_json::to_vec(&serde_json::to_value(&record)?)?;
                    let outcome = BundleEntryOutcome::Converted {
                        unit_id: record.unit.uuid,
                        symbol_id: record.symbol.uuid,
                        offset: bytes,
                        payload_bytes: encoded.len() as u64,
                        payload_hash: compute_source_hash_bytes(&encoded),
                        warnings: record.notices,
                    };
                    if let Some(output) = &mut output {
                        output.write_all(&encoded)?;
                        output.write_all(b"\n")?;
                    }
                    bytes += encoded.len() as u64 + 1;
                    converted += 1;
                    outcome
                }
                Err(error) => {
                    refused += 1;
                    BundleEntryOutcome::Refused {
                        reason: error.to_string(),
                    }
                }
            };
            entries.push(BundleEntry {
                name: symbol.name,
                description: symbol.description,
                outcome,
            });
        }
        libraries.push(BundleLibrary {
            source: library.source,
            source_hash: Some(library.source_hash),
            format_version: library.format_version,
            failure: None,
            symbols: entries,
        });
    }
    if let Some(output) = &mut output {
        output.flush()?;
    }
    let manifest = BundleManifest {
        schema_version: 1,
        source_format: "kicad_symbol".into(),
        source_version: version,
        converter_version: env!("CARGO_PKG_VERSION").into(),
        converter_profile: "single-unit-normal-style-v1".into(),
        complete_inventory: failures == 0,
        complete_conversion: failures == 0 && refused == 0,
        converted,
        refused,
        unreadable_libraries: failures,
        payload_total_bytes: bytes,
        report_only: output.is_none(),
        libraries,
    };
    let mut report = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(report)?,
    );
    serde_json::to_writer(&mut report, &manifest)?;
    report.write_all(b"\n")?;
    report.flush()?;
    println!(
        "Converted {converted}; refused {refused}; unreadable libraries {failures}; native payload {bytes} bytes"
    );
    Ok(())
}
