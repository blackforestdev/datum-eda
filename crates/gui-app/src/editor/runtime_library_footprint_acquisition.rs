//! Selected-source reads return disposable data; native writes reread the source.
use super::{LibraryAcquisitionPreview, LibrarySourceSymbol, ReadResult, catalog};
use datum_gui_protocol::library_browser::acquisition::FootprintAcquisitionPreview;
use std::path::Path;

pub(super) fn read(
    path: &Path,
    bundled: bool,
    footprints: bool,
    source: Option<LibrarySourceSymbol>,
) -> Result<ReadResult, String> {
    let Some(source) = source else {
        return if footprints {
            catalog::discover_footprints(path)
        } else if bundled {
            catalog::bundled()
        } else {
            catalog::discover(path)
        }
        .map(ReadResult::Names);
    };
    if footprints {
        let converted = eda_engine::import::kicad_footprint_library::convert(&source.path)
            .map_err(|error| error.to_string())?;
        if converted.footprint.name != source.name {
            return Err("Footprint source changed during selection; refresh the source".into());
        }
        let value = |v| serde_json::to_value(v).map_err(|e| e.to_string());
        return Ok(ReadResult::Preview(LibraryAcquisitionPreview {
            native_part: None,
            name: source.name,
            source_digest: converted.source_hash,
            source_path: converted.source_path,
            unit: serde_json::Value::Null,
            symbol: serde_json::Value::Null,
            footprint: Some(FootprintAcquisitionPreview {
                package: value(converted.package)?,
                footprint: serde_json::to_value(converted.footprint).map_err(|e| e.to_string())?,
                padstacks: converted
                    .padstacks
                    .into_iter()
                    .map(|stack| serde_json::to_value(stack).map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            notices: converted.warnings,
        }));
    }
    let converted = if bundled {
        eda_engine::import::library_bundle::preview(&source.path, &source.name)
    } else {
        eda_engine::import::kicad_library::convert(&source.path, &source.name)
    }
    .map_err(|error| error.to_string())?;
    Ok(ReadResult::Preview(
        LibraryAcquisitionPreview::from_converted_symbol(converted)?,
    ))
}
