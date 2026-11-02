//! Disposable local-source selection. Only an explicit import commits a preview.
use crate::single_line_editor::SingleLineEditor;
use serde_json::Value;
use uuid::Uuid;
#[path = "library_source_catalog.rs"]
pub mod catalog;
pub use catalog::LibrarySourceSymbol;

#[derive(Clone, Debug, PartialEq)]
pub struct LibraryAcquisitionPreview {
    pub name: String,
    pub source_digest: String,
    pub source_path: std::path::PathBuf,
    pub unit: Value,
    pub symbol: Value,
    pub notices: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LibraryAcquisition {
    pub session: Uuid,
    pub revision: String,
    pub tip: Option<Uuid>,
    pub path: String,
    pub bundled: bool,
    pub include_unavailable: bool,
    pub path_editor: SingleLineEditor,
    pub search: String,
    pub search_editor: SingleLineEditor,
    pub names: Vec<String>,
    pub sources: Vec<LibrarySourceSymbol>,
    pub catalog_notices: Vec<String>,
    pub selected: Option<String>,
    pub selected_source: Option<std::path::PathBuf>,
    pub placement_pane: Option<crate::PaneId>,
    pub placement_document: Option<crate::schematic_documents::SchematicDocumentKey>,
    pub placement_zoom: Option<crate::PaneId>,
    pub placement_temporary_pane: Option<crate::PaneId>,
    /// Explicit confirmation belongs to one source read, never a later query.
    pub placement_confirmation: Option<Uuid>,
    pub preview: Option<LibraryAcquisitionPreview>,
    pub refusal: Option<String>,
    pub loading: bool,
    pub loaded: bool,
    pub offset: usize,
}

impl LibraryAcquisition {
    pub fn begin(revision: String, tip: Option<Uuid>) -> Self {
        let path = "/usr/share/kicad/symbols".to_owned();
        Self {
            session: Uuid::new_v4(),
            revision,
            tip,
            path_editor: SingleLineEditor::at_end(&path),
            path,
            bundled: false,
            include_unavailable: false,
            search: String::new(),
            search_editor: SingleLineEditor::default(),
            names: Vec::new(),
            sources: Vec::new(),
            catalog_notices: Vec::new(),
            selected: None,
            selected_source: None,
            placement_pane: None,
            placement_document: None,
            placement_zoom: None,
            placement_temporary_pane: None,
            placement_confirmation: None,
            preview: None,
            refusal: None,
            loading: false,
            loaded: false,
            offset: 0,
        }
    }

    pub fn results(&self) -> Vec<usize> {
        let query = self.search.trim().to_lowercase();
        let mut results: Vec<_> =
            self.names
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    self.include_unavailable
                        || self
                            .sources
                            .get(*index)
                            .is_none_or(|source| !source.is_unavailable())
                })
                .filter(|(index, name)| {
                    name.to_lowercase().contains(&query)
                        || self.sources.get(*index).is_some_and(|source| {
                            source.description.to_lowercase().contains(&query)
                        })
                })
                .map(|(index, _)| index)
                .collect();
        results.sort_by_cached_key(|index| {
            let exact = !query.is_empty()
                && (self.names[*index].to_lowercase() == query
                    || self.sources.get(*index).is_some_and(|source| {
                        source.name.to_lowercase() == query
                            || source.description.to_lowercase() == query
                    }));
            let unavailable = self
                .sources
                .get(*index)
                .is_some_and(LibrarySourceSymbol::is_unavailable);
            (!exact, unavailable, self.names[*index].to_lowercase())
        });
        results
    }

    pub fn unavailable_count(&self) -> usize {
        self.sources
            .iter()
            .filter(|source| source.is_unavailable())
            .count()
    }

    pub fn take_placement_confirmation(&mut self) -> bool {
        self.placement_confirmation.take() == Some(self.session) && self.placement_pane.is_some()
    }

    /// A changed query retires a selected preview, not the catalog being read.
    pub fn search_changed(&mut self) {
        self.placement_confirmation = None;
        self.offset = 0;
        self.refusal = None;
        self.preview = None;
        self.selected = None;
        self.selected_source = None;
        if self.loaded {
            self.session = Uuid::new_v4();
            self.loading = false;
        }
    }

    /// Editing the source invalidates all results and outstanding reads of it.
    pub fn source_changed(&mut self) {
        self.placement_confirmation = None;
        self.session = Uuid::new_v4();
        self.names.clear();
        self.sources.clear();
        self.catalog_notices.clear();
        self.selected = None;
        self.selected_source = None;
        self.preview = None;
        self.refusal = None;
        self.offset = 0;
        self.loading = false;
        self.loaded = false;
    }

    pub fn request(
        &self,
    ) -> Result<eda_engine::api::native_write::library_acquisition::LibraryAcquisitionRequest, String>
    {
        let preview = self
            .preview
            .as_ref()
            .ok_or("Preview a supported symbol before importing")?;
        if self.loading
            || self.selected.as_deref() != Some(preview.name.as_str())
            || self.selected_source.as_ref() != Some(&preview.source_path)
        {
            return Err("Wait for the selected symbol preview before importing".into());
        }
        Ok(
            eda_engine::api::native_write::library_acquisition::LibraryAcquisitionRequest {
                source_format: if self.bundled {
                    "datum_bundle"
                } else {
                    "kicad_symbol"
                }
                .into(),
                source_path: preview.source_path.clone(),
                symbol_name: preview.name.clone(),
                expected_source_hash: preview.source_digest.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_search_finds_resistor_and_hides_refused_symbols_until_requested() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.bundled = true;
        acquisition.sources = catalog::bundled().unwrap().symbols;
        acquisition.names = acquisition
            .sources
            .iter()
            .map(|s| s.label.clone())
            .collect();
        let manifest = eda_engine::import::library_bundle::catalog().unwrap();
        let visible = acquisition.results();
        assert_eq!(visible.len(), manifest.converted as usize);
        assert!(
            visible
                .iter()
                .all(|index| !acquisition.sources[*index].is_unavailable())
        );
        assert_eq!(acquisition.unavailable_count(), manifest.refused as usize);

        acquisition.search = "  ReSiStOr  ".into();
        let first = &acquisition.sources[acquisition.results()[0]];
        assert_eq!(first.label, "Device:R");
        assert_eq!(first.availability, catalog::SourceAvailability::Supported);
        assert_eq!(first.display_label(), "Resistor — Device:R");
        assert_eq!(first.path, std::path::Path::new("kicad/Device.kicad_sym"));

        for query in ["Amplifier_Operational:TL072", "4xxx_IEEE:4001"] {
            acquisition.search = query.into();
            acquisition.include_unavailable = false;
            assert!(
                acquisition
                    .results()
                    .iter()
                    .all(|index| acquisition.sources[*index].label != query)
            );
            acquisition.include_unavailable = true;
            let first = &acquisition.sources[acquisition.results()[0]];
            assert_eq!(first.label, query);
            assert!(first.is_unavailable());
            assert!(first.display_label().ends_with(" · unavailable"));
        }
    }

    #[test]
    fn changing_query_preserves_catalog_read_but_retires_previous_preview() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.loading = true;
        let catalog_session = acquisition.session;
        acquisition.placement_pane = Some(crate::PaneId(1));
        acquisition.placement_confirmation = Some(catalog_session);
        acquisition.search_changed();
        assert!(!acquisition.take_placement_confirmation());
        assert_eq!(acquisition.session, catalog_session);
        assert!(acquisition.loading);

        acquisition.loaded = true;
        acquisition.placement_confirmation = Some(acquisition.session);
        acquisition.names = vec!["Device:R".into()];
        acquisition.offset = 4;
        acquisition.selected = Some("R".into());
        acquisition.selected_source = Some("kicad/Device.kicad_sym".into());
        acquisition.refusal = Some("Previous selection was unavailable".into());
        acquisition.preview = Some(LibraryAcquisitionPreview {
            name: "R".into(),
            source_digest: "hash".into(),
            source_path: "kicad/Device.kicad_sym".into(),
            unit: Value::Null,
            symbol: Value::Null,
            notices: Vec::new(),
        });
        acquisition.search_changed();
        assert_ne!(acquisition.session, catalog_session);
        assert!(!acquisition.loading);
        assert!(acquisition.loaded);
        assert_eq!(acquisition.names, vec!["Device:R"]);
        assert_eq!(acquisition.offset, 0);
        assert!(acquisition.refusal.is_none());
        assert!(acquisition.preview.is_none());
        assert!(acquisition.selected.is_none());
        assert!(acquisition.selected_source.is_none());
        assert!(acquisition.request().is_err());
        assert!(!acquisition.take_placement_confirmation());
    }

    #[test]
    fn placement_confirmation_is_explicit_one_shot_and_session_qualified() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.placement_pane = Some(crate::PaneId(1));
        assert!(!acquisition.take_placement_confirmation());
        acquisition.placement_confirmation = Some(acquisition.session);
        assert!(acquisition.take_placement_confirmation());
        assert!(!acquisition.take_placement_confirmation());
        acquisition.placement_confirmation = Some(Uuid::new_v4());
        assert!(!acquisition.take_placement_confirmation());
        acquisition.placement_confirmation = Some(acquisition.session);
        acquisition.source_changed();
        assert!(!acquisition.take_placement_confirmation());
        acquisition.placement_confirmation = Some(acquisition.session);
        acquisition.placement_pane = None;
        assert!(
            !acquisition.take_placement_confirmation(),
            "inspection never implies placement"
        );
    }

    #[test]
    fn exact_name_precedes_other_substring_matches_without_hiding_them() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.names = vec!["Crystal".into(), "R".into(), "R_Small".into()];
        acquisition.search = "r".into();
        assert_eq!(acquisition.results(), vec![1, 0, 2]);
    }
    #[test]
    fn installed_catalog_searches_descriptions_and_preserves_same_name_sources() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.sources = ["Device", "Other"]
            .into_iter()
            .map(|library| LibrarySourceSymbol {
                path: format!("/libraries/{library}.kicad_sym").into(),
                name: "R".into(),
                label: format!("{library}:R"),
                description: "Resistor".into(),
                availability: catalog::SourceAvailability::Unchecked,
            })
            .collect();
        acquisition.names = acquisition
            .sources
            .iter()
            .map(|s| s.label.clone())
            .collect();
        acquisition.search = "resistor".into();
        assert_eq!(acquisition.results(), vec![0, 1]);
        acquisition.search = "other:r".into();
        assert_eq!(acquisition.results(), vec![1]);
    }
    #[test]
    fn source_changes_and_selection_changes_cannot_import_a_previous_preview() {
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        assert!(acquisition.request().is_err());
        acquisition.selected = Some("R".into());
        acquisition.selected_source = Some("/canonical/Device.kicad_sym".into());
        acquisition.preview = Some(LibraryAcquisitionPreview {
            name: "R".into(),
            source_digest: "source-hash".into(),
            source_path: "/canonical/Device.kicad_sym".into(),
            unit: Value::Null,
            symbol: Value::Null,
            notices: Vec::new(),
        });
        let request = acquisition.request().unwrap();
        assert_eq!(request.symbol_name, "R");
        assert_eq!(
            request.source_path,
            std::path::PathBuf::from("/canonical/Device.kicad_sym")
        );
        assert_eq!(request.expected_source_hash, "source-hash");
        acquisition.selected_source = Some("/another/Device.kicad_sym".into());
        assert!(acquisition.request().is_err());
        acquisition.selected_source = Some("/canonical/Device.kicad_sym".into());
        acquisition.loading = true;
        assert!(acquisition.request().is_err());
        acquisition.loading = false;
        acquisition.selected = Some("C".into());
        assert!(acquisition.request().is_err());
        let session = acquisition.session;
        acquisition.source_changed();
        assert_ne!(session, acquisition.session);
        assert!(acquisition.request().is_err());
        assert!(acquisition.preview.is_none());
    }
}
