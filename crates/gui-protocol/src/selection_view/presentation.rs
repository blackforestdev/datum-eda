//! Read-only ownership and canonical authored anchors for presentation LOD.
use super::*;
impl SelectionView {
    pub(super) fn document_presentation(
        &self,
        scene: &crate::BoardReviewSceneV1,
    ) -> Option<&SchematicSelectionPresentation> {
        let document = scene.schematic_document.as_ref()?;
        self.schematic_documents
            .iter()
            .find(|p| &p.document == document && p.scene_id == scene.scene_id)
    }

    pub fn related_projection_keys(
        &self,
        scene: &crate::BoardReviewSceneV1,
        surface: crate::PaneContent,
    ) -> Option<&BTreeSet<String>> {
        self.projection_keys(scene, surface)?;
        match surface {
            crate::PaneContent::Board => Some(&self.related_board_keys),
            crate::PaneContent::Schematic => match &scene.schematic_document {
                Some(_) => self.document_presentation(scene).map(|p| &p.related),
                None => Some(&self.related_schematic_keys),
            },
            _ => None,
        }
    }

    pub fn presentation_owner(
        &self,
        scene: &crate::BoardReviewSceneV1,
        surface: crate::PaneContent,
        key: &str,
    ) -> Option<&Id> {
        self.projection_keys(scene, surface)?
            .contains(key)
            .then_some(())?;
        let owner = match surface {
            crate::PaneContent::Board => self.board_presentations.get(key),
            crate::PaneContent::Schematic => match &scene.schematic_document {
                Some(_) => self.document_presentation(scene)?.owners.get(key),
                None => self.schematic_presentations.get(key),
            },
            crate::PaneContent::Library | crate::PaneContent::Revision(_) => None,
        }?;
        self.members.contains(owner).then_some(owner)
    }
    /// No bounds-center, text-layout or scene-name fallback can invent an anchor.
    pub fn authored_anchor(&self, owner: &Id) -> Option<crate::PointNm> {
        if !self.members.contains(owner) {
            return None;
        }
        if let Some(anchor) = self.authored_anchors.get(owner) {
            return Some(*anchor);
        }
        let values = self.values.get(owner)?;
        let field = match owner.class {
            Class::Footprint | Class::Symbol => "placement_position_nm",
            Class::Track => "section_start_nm",
            Class::Wire | Class::BusSection => "section_start_nm",
            Class::Pin => "terminal_position_nm",
            _ => "position_nm",
        };
        let value = values.get(field)?;
        Some(crate::PointNm {
            x: value.get("x")?.as_i64()?,
            y: value.get("y")?.as_i64()?,
        })
    }
}

/// Compile all visible source occurrences against the one complete authored set.
pub(super) fn compile(
    view: &mut SelectionView,
    authority: &NativeSelectionResolution,
    electrical: Option<&eda_engine::substrate::ElectricalMembership>,
    board: &crate::BoardReviewSceneV1,
    schematics: &[&crate::BoardReviewSceneV1],
    visible: &impl Fn(&str) -> bool,
) -> (BTreeSet<Id>, BTreeSet<Id>) {
    let mut rendered = BTreeSet::new();
    let mut represented = BTreeSet::new();
    for (is_board, scene) in
        std::iter::once((true, board)).chain(schematics.iter().map(|s| (false, *s)))
    {
        if scene.project_uuid != view.project.to_string()
            || scene.source_revision != view.revision.0
        {
            continue;
        }
        if let Some(document) = &scene.schematic_document {
            if document.project != view.project
                || scene.board_uuid != document.source_sheet.to_string()
                || authority
                    .snapshot()
                    .schematic_source()
                    .is_none_or(|source| source.uuid != document.schematic)
            {
                continue;
            }
        }
        let path = scene
            .schematic_document
            .as_ref()
            .map_or(&[][..], |d| d.instance_path.as_slice());
        let entry_sources: BTreeSet<_> = electrical
            .into_iter()
            .flat_map(|e| e.owned.iter())
            .filter(|e| e.class == "bus_entries" && e.instance_path == path)
            .map(|e| e.source_id.to_string())
            .collect();
        let mut own = BTreeSet::new();
        let mut related = BTreeSet::new();
        let mut presentations = BTreeMap::new();
        for graphic in &scene.board_graphics {
            if entry_sources.contains(&graphic.source_object_uuid) {
                own.insert(graphic.object_id.clone());
                // Only an engine-qualified owned entry can use its explicit
                // source spine for whole-owner presentation LOD. It never
                // becomes a separate selected member or scalar-net identity.
                if let Some(entry) = authority
                    .snapshot()
                    .schematic_source()
                    .and_then(|s| {
                        scene
                            .board_uuid
                            .parse()
                            .ok()
                            .and_then(|id| s.sheets.get(&id))
                    })
                    .and_then(|sheet| {
                        graphic
                            .source_object_uuid
                            .parse()
                            .ok()
                            .and_then(|id| sheet.bus_entries.get(&id))
                    })
                {
                    let owner = Id {
                        class: Class::BusSection,
                        id: entry.bus,
                        instance_path: path.to_vec(),
                    };
                    if view.members.contains(&owner) {
                        presentations.insert(graphic.object_id.clone(), owner);
                    }
                }
            }
        }
        let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, authority);
        for (key, binding) in bindings.iter() {
            for id in [&binding.projected, &binding.acquired] {
                represented.insert(id.clone());
                if binding.visible(visible) {
                    rendered.insert(id.clone());
                }
                if view.members.contains(id) {
                    own.insert(key.to_owned());
                    // Projected member wins when both it and its parent are
                    // selected; repeated owned parts never mint a new member.
                    presentations
                        .entry(key.to_owned())
                        .or_insert_with(|| id.clone());
                }
                if view.related.contains(id) {
                    related.insert(key.to_owned());
                }
            }
        }

        if is_board {
            view.board_keys = own;
            view.related_board_keys = related;
            view.board_presentations = presentations;
        } else if let Some(document) = &scene.schematic_document {
            if !view
                .schematic_documents
                .iter()
                .any(|p| &p.document == document && p.scene_id == scene.scene_id)
            {
                view.schematic_documents
                    .push(SchematicSelectionPresentation {
                        document: document.clone(),
                        scene_id: scene.scene_id.clone(),
                        keys: own,
                        related,
                        owners: presentations,
                    });
            }
        } else {
            view.schematic_keys.extend(own);
            view.related_schematic_keys.extend(related);
            view.schematic_presentations.extend(presentations);
        }
    }
    (represented, rendered)
}
