//! Complete schematic gesture preview over an immutable source copy.
use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{HiddenPowerBehavior, PlacedSymbol, SchematicWire, SymbolDisplayMode};
impl SchematicAuthoring {
    pub fn active(&self) -> bool {
        self.chosen.is_some() && !self.picker || self.wire
    }
    pub fn cancel(&mut self) -> Option<BoardReviewSceneV1> {
        self.picker = false;
        self.chosen = None;
        self.position = None;
        self.vertices.clear();
        self.last_wire_click = None;
        self.wire = false;
        self.pending_request = None;
        self.base.take()
    }
    pub fn preview_scene(
        &self,
        base: &BoardReviewSceneV1,
        root: &std::path::Path,
    ) -> anyhow::Result<BoardReviewSceneV1> {
        let mut source = self
            .source
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Native schematic unavailable"))?;
        let sheet = self
            .sheet
            .ok_or_else(|| anyhow::anyhow!("Native sheet unavailable"))?;
        let selected = source
            .sheets
            .get_mut(&sheet)
            .ok_or_else(|| anyhow::anyhow!("Native sheet changed"))?;
        if let Some(index) = self.chosen
            && let Some(at) = self.position
            && !self.picker
        {
            let choice = &self.choices[index];
            let mut pins = choice.pins.clone();
            for pin in &mut pins {
                pin.position = eda_engine::api::native_write::schematic_edit::place_local_point(
                    pin.position,
                    Point { x: at.x, y: at.y },
                    0,
                    false,
                )?;
            }
            let symbol = PlacedSymbol {
                uuid: Uuid::nil(),
                part: None,
                entity: None,
                gate: None,
                lib_id: Some(choice.id.to_string()),
                reference: self.reference.clone(),
                value: self.value.clone(),
                fields: Vec::new(),
                pins,
                position: Point { x: at.x, y: at.y },
                rotation: 0,
                mirrored: false,
                unit_selection: None,
                display_mode: SymbolDisplayMode::LibraryDefault,
                pin_overrides: Vec::new(),
                hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
            };
            selected.symbols.insert(symbol.uuid, symbol);
        }
        if self.wire {
            let mut points = self.vertices.clone();
            if let Some(at) = self.position
                && points.last() != Some(&at)
            {
                points.push(at);
            }
            for pair in points.windows(2) {
                let wire = SchematicWire {
                    uuid: Uuid::new_v4(),
                    from: Point {
                        x: pair[0].x,
                        y: pair[0].y,
                    },
                    to: Point {
                        x: pair[1].x,
                        y: pair[1].y,
                    },
                };
                selected.wires.insert(wire.uuid, wire);
            }
        }
        self.project_source(&source, sheet, base, root)
    }
    pub(crate) fn project_source(
        &self,
        source: &eda_engine::schematic::Schematic,
        sheet: Uuid,
        base: &BoardReviewSceneV1,
        root: &std::path::Path,
    ) -> anyhow::Result<BoardReviewSceneV1> {
        let mut scene = crate::schematic_scene_import::project_schematic_sheet_source(
            &source,
            &source.sheets[&sheet],
            &root.join("schematic/schematic.json"),
        )?;
        for placed in source.sheets[&sheet].symbols.values() {
            if let Some(choice) = self
                .choices
                .iter()
                .find(|choice| placed.lib_id.as_deref() == Some(&choice.id.to_string()))
            {
                crate::schematic_scene_import::native_library::replace_body(
                    &mut scene,
                    placed,
                    &choice.symbol,
                )?;
            }
        }
        scene.bounds = base.bounds.clone();
        scene.source_revision = base.source_revision.clone();
        scene.project_uuid = base.project_uuid.clone();
        scene.project_name = base.project_name.clone();
        Ok(scene)
    }
    /// Exact terminal pickup uses existing pin-terminal graphic identity, never
    /// an invented distance threshold. Background coordinates use shared snap.
    pub fn pin_point(&self, source_uuid: &str) -> Option<PointNm> {
        self.source
            .as_ref()?
            .sheets
            .get(&self.sheet?)?
            .symbols
            .values()
            .flat_map(|s| s.pins.iter())
            .find(|pin| pin.uuid.to_string() == source_uuid)
            .map(|p| PointNm {
                x: p.position.x,
                y: p.position.y,
            })
    }
}
