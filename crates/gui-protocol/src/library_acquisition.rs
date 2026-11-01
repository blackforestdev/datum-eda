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
            search: String::new(),
            search_editor: SingleLineEditor::default(),
            names: Vec::new(),
            sources: Vec::new(),
            catalog_notices: Vec::new(),
            selected: None,
            selected_source: None,
            placement_pane: None,
            placement_document: None,
            preview: None,
            refusal: None,
            loading: false,
            loaded: false,
            offset: 0,
        }
    }

    pub fn results(&self) -> Vec<usize> {
        let query = self.search.to_lowercase();
        let mut results: Vec<_> =
            self.names
                .iter()
                .enumerate()
                .filter(|(index, name)| {
                    name.to_lowercase().contains(&query)
                        || self.sources.get(*index).is_some_and(|source| {
                            source.description.to_lowercase().contains(&query)
                        })
                })
                .map(|(index, _)| index)
                .collect();
        results.sort_by_cached_key(|index| {
            let exact = self.names[*index].to_lowercase() == query
                || self.sources.get(*index).is_some_and(|source| {
                    source.name.to_lowercase() == query
                        || source.description.to_lowercase() == query
                });
            (!exact, self.names[*index].to_lowercase())
        });
        results
    }

    /// Editing the source invalidates all results and outstanding reads of it.
    pub fn source_changed(&mut self) {
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
