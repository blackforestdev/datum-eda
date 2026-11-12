//! Clipboard placement configures the existing qualified schematic gesture.
use super::*;
use eda_engine::{
    api::native_write::{
        design_clipboard::{ClipboardPlacement, DesignClipboardBundle},
        design_clipboard_edit::{ClipboardContactPolicy, DesignEditIntent, preview_placement},
    },
    ir::geometry::Point,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignClipboardPlacement {
    pub bundle: DesignClipboardBundle,
    pub edit_seed: Uuid,
    pub symbol_definitions: std::collections::BTreeMap<Uuid, eda_engine::pool::Symbol>,
}
impl DesignClipboardPlacement {
    pub fn capture(bundle: DesignClipboardBundle, edit_seed: Uuid) -> Result<Self, String> {
        let items = bundle
            .placed(
                ClipboardPlacement {
                    position: bundle.reference,
                    rotation_deg: 0,
                    mirrored: false,
                },
                edit_seed,
            )
            .map_err(|error| error.to_string())?;
        let mut symbol_definitions = std::collections::BTreeMap::new();
        if let Some(data) = &bundle.symbols {
            for (old, placed) in bundle.items.iter().zip(&items) {
                if let Some(source) = data.sources.get(&old.id) {
                    symbol_definitions.insert(
                        placed.id,
                        serde_json::from_value(source.definition.clone())
                            .map_err(|e| e.to_string())?,
                    );
                }
            }
        }
        Ok(Self {
            bundle,
            edit_seed,
            symbol_definitions,
        })
    }

    pub fn request(
        &self,
        authoring: &SchematicAuthoring,
        at: PointNm,
    ) -> Result<SchematicEditRequest, String> {
        if authoring.pending_request.is_some() {
            return Err("Wait for the pending design edit".into());
        }
        let basis = self.basis(authoring)?;
        self.preview_sheet(authoring, at)?;
        Ok(SchematicEditRequest::DesignEdit {
            sheet: basis.source,
            instance_path: basis.document.as_ref().unwrap().instance_path.clone(),
            edit: DesignEditIntent::PlaceBundle {
                bundle: self.bundle.clone(),
                placement: Self::pose(authoring, at),
                edit_seed: self.edit_seed,
                contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
            },
        })
    }

    pub fn scene_at(
        &self,
        authoring: &SchematicAuthoring,
        base: &BoardReviewSceneV1,
        root: &std::path::Path,
        at: PointNm,
    ) -> anyhow::Result<BoardReviewSceneV1> {
        let basis = self.basis(authoring).map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            base.project_uuid == basis.project.to_string()
                && base.source_revision == basis.revision
                && base.schematic_document == basis.document,
            "Finish clipboard placement in its captured source pane"
        );
        let preview = self
            .preview_sheet(authoring, at)
            .map_err(anyhow::Error::msg)?;
        let mut source = authoring
            .source
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Native schematic source unavailable"))?;
        source.sheets.insert(basis.source, preview);
        authoring.project_source(&source, basis.source, base, root)
    }

    fn pose(authoring: &SchematicAuthoring, at: PointNm) -> ClipboardPlacement {
        ClipboardPlacement {
            position: Point::new(at.x, at.y),
            rotation_deg: authoring.rotation_deg,
            mirrored: authoring.mirrored,
        }
    }
    fn basis<'a>(
        &self,
        authoring: &'a SchematicAuthoring,
    ) -> Result<&'a crate::AuthoringViewBasis, String> {
        let basis = authoring
            .circuit_basis
            .as_ref()
            .ok_or("Clipboard placement requires its captured schematic occurrence")?;
        let document = basis
            .document
            .as_ref()
            .ok_or("Clipboard placement requires a qualified schematic document")?;
        let source = authoring
            .source
            .as_ref()
            .ok_or("Native schematic source unavailable")?;
        if basis.content != crate::PaneContent::Schematic
            || document.project != basis.project
            || document.source_sheet != basis.source
            || document.schematic != source.uuid
            || !source.sheets.contains_key(&basis.source)
        {
            return Err(
                "Clipboard placement source changed; cancel and select the target again".into(),
            );
        }
        Ok(basis)
    }
    fn preview_sheet(
        &self,
        authoring: &SchematicAuthoring,
        at: PointNm,
    ) -> Result<eda_engine::schematic::Sheet, String> {
        let basis = self.basis(authoring)?;
        let target = &authoring.source.as_ref().unwrap().sheets[&basis.source];
        preview_placement(
            target,
            &self.bundle,
            &Self::pose(authoring, at),
            &self.edit_seed,
            ClipboardContactPolicy::DrawWireEndpoints,
        )
        .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
#[path = "design_clipboard_placement_tests.rs"]
mod tests;
