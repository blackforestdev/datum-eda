//! Project a qualified source occurrence once per immutable document basis.
use super::*;
use std::path::Path;

impl SchematicDocuments {
    /// Source equality for retaining the existing immutable projections during
    /// an evidence-only refresh. Navigation is checked by the publication owner.
    pub fn same_source(&self, other: &Self) -> bool {
        self.project == other.project
            && self.revision == other.revision
            && self.document_name == other.document_name
            && self.rows == other.rows
            && self.source == other.source
            && self.unavailable == other.unavailable
    }
    pub fn is_source_pin(&self, id: Uuid) -> bool {
        self.source.as_ref().is_some_and(|source| {
            source
                .sheets
                .values()
                .flat_map(|sheet| sheet.symbols.values())
                .any(|symbol| symbol.pins.iter().any(|pin| pin.uuid == id))
        })
    }
    pub fn source_occurrences(&self, id: Uuid) -> impl Iterator<Item = &SchematicDocumentKey> {
        self.rows
            .iter()
            .filter(move |row| {
                self.source
                    .as_ref()
                    .and_then(|source| source.sheets.get(&row.key.source_sheet))
                    .is_some_and(|sheet| {
                        sheet.uuid == id
                            || sheet.symbols.contains_key(&id)
                            || sheet
                                .symbols
                                .values()
                                .any(|symbol| symbol.pins.iter().any(|pin| pin.uuid == id))
                            || sheet.wires.contains_key(&id)
                            || sheet.junctions.contains_key(&id)
                            || sheet.labels.contains_key(&id)
                            || sheet.buses.contains_key(&id)
                            || sheet.bus_entries.contains_key(&id)
                            || sheet.ports.contains_key(&id)
                            || sheet.noconnects.contains_key(&id)
                            || sheet.texts.contains_key(&id)
                            || sheet.drawings.contains_key(&id)
                    })
            })
            .map(|row| &row.key)
    }
    pub fn source_sheet(&self, key: &SchematicDocumentKey) -> Option<&Sheet> {
        self.row(key)?;
        self.source.as_ref()?.sheets.get(&key.source_sheet)
    }

    pub fn symbol_has_pin(&self, symbol: Uuid, pin: Uuid) -> bool {
        self.source.as_ref().is_some_and(|source| {
            source
                .sheets
                .values()
                .filter_map(|sheet| sheet.symbols.get(&symbol))
                .any(|symbol| symbol.pins.iter().any(|candidate| candidate.uuid == pin))
        })
    }
    pub fn scene_for_pane(&self, pane: PaneId) -> Option<&crate::BoardReviewSceneV1> {
        self.scene_for_pane_with_primary(pane, None)
    }

    pub fn scene_for_pane_with_primary<'a>(
        &'a self,
        pane: PaneId,
        primary: Option<&'a crate::BoardReviewSceneV1>,
    ) -> Option<&'a crate::BoardReviewSceneV1> {
        let key = self.panes.get(&pane)?;
        self.row(key)?;
        let scene = if self.primary.as_ref() == Some(key) {
            primary.or_else(|| self.scenes.get(key).map(AsRef::as_ref))
        } else {
            self.scenes.get(key).map(AsRef::as_ref)
        }?;
        (scene.schematic_document.as_ref() == Some(key)
            && scene.project_uuid == key.project.to_string()
            && self
                .revision
                .as_ref()
                .is_some_and(|r| scene.source_revision == r.0))
        .then_some(scene)
    }

    pub fn prepare_scene(
        &mut self,
        key: &SchematicDocumentKey,
        root: &Path,
        model: &eda_engine::substrate::DesignModel,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.project == Some(model.project.project_id)
                && self
                    .revision
                    .as_ref()
                    .is_some_and(|r| r.0 == model.model_revision.0)
                && self.row(key).is_some(),
            "Document source basis is unavailable or stale"
        );
        if self.scenes.contains_key(key) {
            return Ok(());
        }
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Document source is unavailable"))?;
        let sheet = source
            .sheets
            .get(&key.source_sheet)
            .ok_or_else(|| anyhow::anyhow!("Document source no longer exists"))?;
        let mut scene = crate::schematic_scene_import::project_schematic_sheet_source(
            source,
            sheet,
            &root.join("schematic/schematic.json"),
        )?;
        crate::schematic_scene_import::native_library::project_bodies(
            &mut scene, model, source, sheet.uuid,
        )?;
        scene.project_uuid = key.project.to_string();
        scene.project_name = self.document_name.clone();
        scene.source_revision = model.model_revision.0.clone();
        scene.schematic_document = Some(key.clone());
        scene.scene_id = format!(
            "native-schematic:{}:{}:{}",
            key.project,
            key.source_sheet,
            key.instance_path
                .iter()
                .map(Uuid::to_string)
                .collect::<Vec<_>>()
                .join("/")
        );
        self.scenes.insert(key.clone(), Arc::new(scene));
        Ok(())
    }

    /// Refresh a source basis while retaining each existing view identity. A
    /// disappeared document stays unavailable; it is never retargeted to root.
    pub fn retain_navigation(&mut self, previous: &Self) {
        if self.project.is_some() && self.project == previous.project {
            self.panes = previous.panes.clone();
            self.inspected = previous.inspected.clone();
            self.expanded = previous.expanded.clone();
        }
    }
}
