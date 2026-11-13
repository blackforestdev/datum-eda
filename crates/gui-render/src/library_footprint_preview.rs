//! Reuse native Library geometry with the selected source's disposable closure.
use crate::*;
use datum_gui_protocol::LibraryJsonValue as Value;
use datum_gui_protocol::library_browser::acquisition::FootprintAcquisitionPreview;
use datum_gui_protocol::library_browser::{LibraryBrowser, LibraryEntry};

pub(super) fn render(
    preview: &FootprintAcquisitionPreview,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    let mut library = LibraryBrowser::default();
    for (kind, raw) in std::iter::once(("packages", &preview.package))
        .chain(std::iter::once(("footprints", &preview.footprint)))
        .chain(preview.padstacks.iter().map(|raw| ("padstacks", raw)))
    {
        let Some(id) = raw
            .get("uuid")
            .and_then(Value::as_str)
            .and_then(|id| uuid::Uuid::parse_str(id).ok())
        else {
            continue;
        };
        library.entries.push(LibraryEntry {
            id,
            kind: kind.into(),
            name: raw
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unnamed source")
                .into(),
            source_path: format!("preview/{kind}/{id}"),
            revision: None,
            raw: raw.clone(),
            warning: None,
        });
    }
    super::super::super::library_preview::render_footprint(
        &library,
        &preview.footprint,
        rect,
        quads,
        text,
    );
}
