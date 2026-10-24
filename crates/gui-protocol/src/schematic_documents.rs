//! Source-qualified schematic document views; Navigator inspection is not selection.
use crate::PaneId;
use crate::selection_resolution::SelectionModelRevision;
use eda_engine::schematic::{Schematic, Sheet};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use uuid::Uuid;
mod lifecycle;
mod navigation;
mod projection;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchematicDocumentKey {
    pub project: Uuid,
    pub schematic: Uuid,
    pub source_sheet: Uuid,
    pub instance_path: Vec<Uuid>,
}

/// One occurrence qualification rule for scoped counts and diagnostic markers.
/// An omitted path is evidence only when the source has one captured occurrence.
pub(crate) fn qualify_source_occurrence<'a>(
    occurrences: impl Iterator<Item = &'a SchematicDocumentKey>,
    declared: Option<&[Uuid]>,
) -> Result<&'a SchematicDocumentKey, &'static str> {
    let mut matching =
        occurrences.filter(|key| declared.is_none_or(|path| key.instance_path == path));
    let key = matching
        .next()
        .ok_or("Finding target has no qualified captured source occurrence")?;
    if matching.next().is_some() {
        return Err("Finding target occurrence is ambiguous; an explicit path is required");
    }
    Ok(key)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchematicDocumentRow {
    pub key: SchematicDocumentKey,
    pub parent: Option<SchematicDocumentKey>,
    pub name: String,
    pub source_name: String,
    pub source_sheet_name: String,
    pub definition: Option<Uuid>,
    pub breadcrumb: Vec<String>,
}

/// View intent survives a source refresh without retaining the previous source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchematicNavigation {
    project: Option<Uuid>,
    panes: BTreeMap<PaneId, SchematicDocumentKey>,
    inspected: Option<SchematicDocumentKey>,
    expanded: BTreeSet<SchematicDocumentKey>,
    scroll_offset: usize,
}

/// One configured scene inventory for selection, context and runtime workers.
pub fn visible_scenes(state: &crate::ReviewWorkspaceState) -> Vec<&crate::BoardReviewSceneV1> {
    let mut scenes: Vec<&crate::BoardReviewSceneV1> = Vec::new();
    for pane in state.ui.layout.leaves() {
        if state.ui.layout.content_for(pane) != Some(crate::PaneContent::Schematic) {
            continue;
        }
        if let Some(scene) = crate::camera_scene_for_pane(state, pane) {
            if !scenes.iter().any(|s| {
                s.scene_id == scene.scene_id && s.schematic_document == scene.schematic_document
            }) {
                scenes.push(scene);
            }
        }
    }
    scenes
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SchematicDocuments {
    pub project: Option<Uuid>,
    pub revision: Option<SelectionModelRevision>,
    pub document_name: String,
    pub rows: Vec<SchematicDocumentRow>,
    /// Each pane retains its own view. Row inspection never changes this map.
    pub panes: BTreeMap<PaneId, SchematicDocumentKey>,
    pub inspected: Option<SchematicDocumentKey>,
    pub expanded: BTreeSet<SchematicDocumentKey>,
    pub scroll_offset: usize,
    pub unavailable: Option<String>,
    source: Option<Arc<Schematic>>,
    scenes: BTreeMap<SchematicDocumentKey, Arc<crate::BoardReviewSceneV1>>,
    primary: Option<SchematicDocumentKey>,
}

impl SchematicDocuments {
    pub fn from_source(
        project: Uuid,
        revision: SelectionModelRevision,
        document_name: String,
        schematic: Schematic,
    ) -> anyhow::Result<Self> {
        let mut result = Self {
            project: Some(project),
            revision: Some(revision),
            document_name,
            ..Self::default()
        };
        let mut instantiated = BTreeSet::new();
        for instance in schematic.sheet_instances.values() {
            let definition = schematic
                .sheet_definitions
                .get(&instance.definition)
                .ok_or_else(|| anyhow::anyhow!("Document definition is unavailable"))?;
            instantiated.insert(definition.root_sheet);
        }
        let mut roots: Vec<_> = schematic
            .sheets
            .values()
            .filter(|sheet| !instantiated.contains(&sheet.uuid))
            .collect();
        roots.sort_by(|a, b| a.name.cmp(&b.name).then(a.uuid.cmp(&b.uuid)));
        for sheet in roots {
            result.visit(
                &schematic,
                sheet,
                Vec::new(),
                None,
                None,
                sheet.name.clone(),
                Vec::new(),
                &mut BTreeSet::new(),
            )?;
        }
        let mut instances: Vec<_> = schematic
            .sheet_instances
            .values()
            .filter(|instance| instance.parent_sheet.is_none())
            .collect();
        instances.sort_by(|a, b| a.name.cmp(&b.name).then(a.uuid.cmp(&b.uuid)));
        for instance in instances {
            let definition = &schematic.sheet_definitions[&instance.definition];
            let sheet = schematic
                .sheets
                .get(&definition.root_sheet)
                .ok_or_else(|| anyhow::anyhow!("Document source is unavailable"))?;
            result.visit(
                &schematic,
                sheet,
                vec![instance.uuid],
                None,
                Some(definition.uuid),
                instance.name.clone(),
                Vec::new(),
                &mut BTreeSet::new(),
            )?;
        }
        anyhow::ensure!(
            schematic
                .sheet_instances
                .values()
                .all(|instance| result
                    .rows
                    .iter()
                    .any(|row| row.key.instance_path.last() == Some(&instance.uuid))),
            "Document occurrence is cyclic or unreachable"
        );
        result.expanded = result
            .rows
            .iter()
            .filter(|row| row.parent.is_none())
            .map(|row| row.key.clone())
            .collect();
        result.source = Some(Arc::new(schematic));
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn visit(
        &mut self,
        schematic: &Schematic,
        sheet: &Sheet,
        path: Vec<Uuid>,
        parent: Option<SchematicDocumentKey>,
        definition: Option<Uuid>,
        name: String,
        mut breadcrumb: Vec<String>,
        ancestors: &mut BTreeSet<Uuid>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(ancestors.insert(sheet.uuid), "Cyclic document occurrence");
        let key = SchematicDocumentKey {
            project: self.project.expect("captured Project"),
            schematic: schematic.uuid,
            source_sheet: sheet.uuid,
            instance_path: path.clone(),
        };
        anyhow::ensure!(
            !self.rows.iter().any(|row| row.key == key),
            "Duplicate document occurrence"
        );
        breadcrumb.push(name.clone());
        let source_name = definition
            .and_then(|id| schematic.sheet_definitions.get(&id))
            .map_or_else(|| sheet.name.clone(), |definition| definition.name.clone());
        self.rows.push(SchematicDocumentRow {
            key: key.clone(),
            parent,
            name,
            source_name,
            source_sheet_name: sheet.name.clone(),
            definition,
            breadcrumb: breadcrumb.clone(),
        });
        let mut children: Vec<_> = schematic
            .sheet_instances
            .values()
            .filter(|instance| instance.parent_sheet == Some(sheet.uuid))
            .collect();
        children.sort_by(|a, b| a.name.cmp(&b.name).then(a.uuid.cmp(&b.uuid)));
        for instance in children {
            let definition = &schematic.sheet_definitions[&instance.definition];
            let child = schematic
                .sheets
                .get(&definition.root_sheet)
                .ok_or_else(|| anyhow::anyhow!("Document source is unavailable"))?;
            let mut child_path = path.clone();
            child_path.push(instance.uuid);
            self.visit(
                schematic,
                child,
                child_path,
                Some(key.clone()),
                Some(definition.uuid),
                instance.name.clone(),
                breadcrumb.clone(),
                ancestors,
            )?;
        }
        ancestors.remove(&sheet.uuid);
        Ok(())
    }

    pub fn row(&self, key: &SchematicDocumentKey) -> Option<&SchematicDocumentRow> {
        self.rows.iter().find(|row| &row.key == key)
    }

    pub fn visible_rows(&self) -> Vec<&SchematicDocumentRow> {
        self.rows
            .iter()
            .filter(|row| {
                let mut parent = row.parent.as_ref();
                while let Some(key) = parent {
                    if !self.expanded.contains(key) {
                        return false;
                    }
                    parent = self.row(key).and_then(|row| row.parent.as_ref());
                }
                true
            })
            .collect()
    }

    pub fn inspect(&mut self, key: &SchematicDocumentKey) -> bool {
        if self.row(key).is_none() {
            return false;
        }
        self.inspected = Some(key.clone());
        true
    }

    pub fn toggle_expanded(&mut self, key: &SchematicDocumentKey) -> bool {
        if self.row(key).is_none() {
            return false;
        }
        if !self.expanded.remove(key) {
            self.expanded.insert(key.clone());
        }
        true
    }
}
