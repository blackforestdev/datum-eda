//! Complete schematic gesture preview over an immutable source copy.
use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{HiddenPowerBehavior, PlacedSymbol, SchematicWire, SymbolDisplayMode};
impl SchematicAuthoring {
    pub fn active(&self) -> bool {
        if self.label_edit.is_some() {
            return true;
        }
        (self.chosen.is_some() || self.label.is_some() || self.marker.is_some()) && !self.picker
            || self.wire
    }
    pub fn cancel(&mut self) -> Option<BoardReviewSceneV1> {
        self.picker = false;
        self.chosen = None;
        self.label = None;
        self.marker = None;
        self.editing = None;
        self.label_edit = None;
        self.position = None;
        self.rotation_deg = 0;
        self.mirrored = false;
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
        if let Some(kind) = self.marker
            && let Some(at) = self.position
            && !self.picker
        {
            let position = Point { x: at.x, y: at.y };
            match kind {
                MarkerKind::Junction => {
                    selected.junctions.insert(
                        Uuid::nil(),
                        eda_engine::schematic::Junction {
                            uuid: Uuid::nil(),
                            position,
                        },
                    );
                }
                MarkerKind::NoConnect => {
                    // An off-pin X is only cursor ink. The request owner below
                    // requires one exact source pin before any canonical write.
                    let (symbol, pin) =
                        self.marker_target(at).unwrap_or((Uuid::nil(), Uuid::nil()));
                    selected.noconnects.insert(
                        Uuid::nil(),
                        eda_engine::schematic::NoConnectMarker {
                            uuid: Uuid::nil(),
                            symbol,
                            pin,
                            position,
                        },
                    );
                }
            }
        }
        if let Some(label) = &self.label {
            if let Some(edit) = &self.label_edit {
                // Invalid intermediate text retains source ink. It remains editable
                // in the form and cannot become a request until validation passes.
                if selected.labels.get(&edit.target.id) == Some(&edit.original)
                    && let Ok(replacement) = edit.replacement(&label.name, label.kind.clone())
                {
                    selected.labels.insert(replacement.uuid, replacement);
                }
            } else if let Some(at) = self.position
                && !self.picker
            {
                selected.labels.insert(
                    Uuid::nil(),
                    eda_engine::schematic::NetLabel {
                        uuid: Uuid::nil(),
                        kind: label.kind.clone(),
                        name: label.name.clone(),
                        position: Point { x: at.x, y: at.y },
                    },
                );
            }
        }
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
                    self.rotation_deg,
                    self.mirrored,
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
                rotation: self.rotation_deg,
                mirrored: self.mirrored,
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
            source,
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
        // Preview ink retains the immutable source occurrence it was acquired from.
        // Dropping this envelope would make the named pane reject its own preview.
        scene.schematic_document = base.schematic_document.clone();
        scene.scene_id = base.scene_id.clone();
        scene.bounds = base.bounds.clone();
        scene.source_revision = base.source_revision.clone();
        scene.project_uuid = base.project_uuid.clone();
        scene.project_name = base.project_name.clone();
        Ok(scene)
    }
    /// Exact terminal pickup uses source-qualified pin or Power-port graphics,
    /// never an invented distance threshold. Background uses shared grid snap.
    pub fn pin_point(&self, source_uuid: &str) -> Option<PointNm> {
        if self.marker == Some(MarkerKind::NoConnect) {
            return self.marker_pin_point(source_uuid);
        }
        let sheet = self.source.as_ref()?.sheets.get(&self.sheet?)?;
        let id: Uuid = source_uuid.parse().ok()?;
        let position = sheet
            .symbols
            .values()
            .flat_map(|s| s.pins.iter())
            .find(|pin| pin.uuid == id)
            .map(|pin| pin.position)
            .or_else(|| {
                sheet
                    .labels
                    .get(&id)
                    .filter(|label| label.kind == eda_engine::schematic::LabelKind::Power)
                    .map(|label| label.position)
            })?;
        Some(PointNm {
            x: position.x,
            y: position.y,
        })
    }
    /// Source-pin-only pickup for No-Connect; Power labels are not pin identity.
    pub fn marker_pin_point(&self, source_uuid: &str) -> Option<PointNm> {
        let sheet = self.source.as_ref()?.sheets.get(&self.sheet?)?;
        let id: Uuid = source_uuid.parse().ok()?;
        let mut matches = sheet
            .symbols
            .values()
            .flat_map(|s| &s.pins)
            .filter(|p| p.uuid == id);
        let pin = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(PointNm {
            x: pin.position.x,
            y: pin.position.y,
        })
    }
    /// Exact immutable source identity; no proximity or rendered bounds become authority.
    fn marker_target(&self, at: PointNm) -> Result<(Uuid, Uuid), String> {
        let sheet = self
            .source
            .as_ref()
            .and_then(|s| self.sheet.and_then(|id| s.sheets.get(&id)))
            .ok_or("Native marker sheet unavailable")?;
        let mut pins = sheet
            .symbols
            .values()
            .flat_map(|s| s.pins.iter().map(move |p| (s.uuid, p)))
            .filter(|(_, p)| p.position.x == at.x && p.position.y == at.y);
        let (symbol, pin) = pins
            .next()
            .ok_or("Place No-Connect on an exact symbol pin")?;
        if pins.next().is_some() {
            return Err("No-Connect requires one unambiguous symbol pin".into());
        }
        Ok((symbol, pin.uuid))
    }
    pub fn marker_request(&self, at: PointNm) -> Result<SchematicEditRequest, String> {
        let kind = self.marker.ok_or("Choose a marker tool first")?;
        let sheet = self.sheet.ok_or("Native marker sheet unavailable")?;
        if self
            .source
            .as_ref()
            .is_none_or(|source| !source.sheets.contains_key(&sheet))
        {
            return Err("Native marker sheet unavailable".into());
        }
        let (symbol, pin) = match kind {
            MarkerKind::Junction => (None, None),
            MarkerKind::NoConnect => {
                let (symbol, pin) = self.marker_target(at)?;
                (Some(symbol), Some(pin))
            }
        };
        Ok(SchematicEditRequest::PlaceMarker {
            sheet,
            kind,
            x_nm: at.x,
            y_nm: at.y,
            symbol,
            pin,
        })
    }
}
