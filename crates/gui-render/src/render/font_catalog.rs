//! One immutable catalog, published only after guarded cold construction.
use crate::cpu_alloc::{Scope, calls::Call};
use crate::text_gpu::budget::{Budget, staging_process};
use glyphon::FontSystem;
use std::sync::{Arc, Mutex};

static CATALOG: Mutex<Option<(String, glyphon::fontdb::Database)>> = Mutex::new(None);

pub(crate) fn try_load_datum_fonts(host: &Arc<Budget>) -> anyhow::Result<FontSystem> {
    let mut catalog = CATALOG.lock().unwrap_or_else(|e| e.into_inner());
    if catalog.is_none() {
        let scope = Scope::new("shared-font-catalog");
        let call = Call::begin(&scope, host.clone(), staging_process(), 0, 0)?;
        let value = scope.with(|| {
            let mut fonts = FontSystem::new();
            install_datum_font_sources(&mut fonts);
            fonts.into_locale_and_db()
        });
        // The published immutable catalog is fixed CPU ownership, not retained
        // upload scratch. Construction peaks still obey the scratch ceilings.
        call.finish(0, None, None)?;
        *catalog = Some(value);
    }
    let (locale, database) = catalog.as_ref().expect("guarded catalog published");
    Ok(FontSystem::new_with_locale_and_db(
        locale.clone(),
        database.clone(),
    ))
}

/// Compatibility/oracle clients; production renderer and measurement owners
/// propagate try_load_datum_fonts failures through their preparation boundary.
#[cfg(test)]
pub(crate) fn load_datum_fonts() -> FontSystem {
    let host = Budget::new(16 * 1024 * 1024);
    let scope = Scope::new("compatibility-font-construction");
    let call = Call::begin(&scope, host.clone(), staging_process(), 0, 0).expect("font call slot");
    let fonts = scope
        .with(|| try_load_datum_fonts(&host))
        .expect("font catalog construction");
    call.finish(0, None, None)
        .expect("font construction budget");
    fonts
}

/// Load the vendored IBM Plex faces into the glyphon font database so chrome and
/// on-canvas UI text render in the Design Book typeface rather than a system
/// fallback (`docs/gui/DATUM_RENDERING_BOOK.md` §5). Embedded at compile time
/// from the engine's vendored assets so the GUI never depends on the CWD.
pub(super) static DATUM_FONT_BYTES: [&[u8]; 6] = [
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf"
    ),
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf"
    ),
    include_bytes!(
        "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf"
    ),
    include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf"),
    include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Medium.ttf"),
    include_bytes!("../../../engine/assets/fonts/jetbrains_mono/JetBrainsMono-Regular.ttf"),
];

fn install_datum_font_sources(font_system: &mut FontSystem) {
    // The executable already owns immutable font bytes. Share six small Arc
    // handles instead of allocating another Vec for every font system/renderer.
    // Keep load order and per-system databases/shaping caches unchanged.
    let sources =
        DATUM_FONT_BYTES.map(|bytes| glyphon::fontdb::Source::Binary(std::sync::Arc::new(bytes)));
    let db = font_system.db_mut();
    for source in sources {
        db.load_font_source(source);
    }
}
