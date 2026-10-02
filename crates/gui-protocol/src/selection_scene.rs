//! Typed hit/source binding. Scene strings never establish authored identity.
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, BoardReviewSceneV1, PointNm,
};

pub fn authored_candidate(
    scene: &BoardReviewSceneV1,
    key: &str,
    authority: &NativeSelectionResolution,
) -> Option<Id> {
    let mut source = scene
        .components
        .iter()
        .find(|v| v.object_id == key)
        .map(|v| (Class::Footprint, v.source_object_uuid.as_str()))
        .or_else(|| {
            scene
                .tracks
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| (Class::Track, v.source_object_uuid.as_str()))
        })
        .or_else(|| {
            scene
                .vias
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| (Class::Via, v.source_object_uuid.as_str()))
        })
        .or_else(|| {
            scene
                .zones
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| (Class::Zone, v.source_object_uuid.as_str()))
        })
        .or_else(|| {
            scene
                .board_texts
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| (Class::BoardText, v.text_uuid.as_str()))
        })
        .or_else(|| {
            scene
                .outline
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| (Class::BoardOutline, v.source_object_uuid.as_str()))
        })
        .and_then(|(class, uuid)| uuid.parse().ok().map(|id| (class, id)));
    // In placement workspaces a pad acquires its actual package, never the child.
    if let Some(pad) = scene.pads.iter().find(|v| v.object_id == key) {
        source = Some((Class::Footprint, pad.component_uuid.parse().ok()?));
    }
    if let Some(graphic) = scene.board_graphics.iter().find(|v| v.object_id == key) {
        let class = match graphic.schematic_hit_kind()? {
            crate::SchematicHitKind::Symbol => Class::Symbol,
            crate::SchematicHitKind::Wire => Class::Wire,
            crate::SchematicHitKind::Bus => Class::BusSection,
            crate::SchematicHitKind::Label => Class::Label,
            crate::SchematicHitKind::Port => Class::Port,
            crate::SchematicHitKind::Junction => Class::Junction,
            crate::SchematicHitKind::NoConnect => Class::NoConnect,
            crate::SchematicHitKind::Pin => {
                let pin: crate::SelectionProjectId = graphic.source_object_uuid.parse().ok()?;
                let schematic = authority.snapshot().schematic_source()?;
                let sheet = schematic.sheets.get(&scene.board_uuid.parse().ok()?)?;
                let parent = sheet
                    .symbols
                    .values()
                    .find(|symbol| symbol.pins.iter().any(|v| v.uuid == pin))?;
                source = Some((Class::Symbol, parent.uuid));
                Class::Symbol
            }
        };
        if !matches!(
            graphic.schematic_hit_kind(),
            Some(crate::SchematicHitKind::Pin)
        ) {
            source = Some((class, graphic.source_object_uuid.parse().ok()?));
        }
    }
    let (class, id) = source?;
    let candidate = Id {
        class,
        id,
        instance_path: vec![],
    };
    authority.authored_contains(&candidate).then_some(candidate)
}

pub fn candidate_visible(
    scene: &BoardReviewSceneV1,
    key: &str,
    layer_visible: impl Fn(&str) -> bool,
) -> bool {
    if let Some(v) = scene.components.iter().find(|v| v.object_id == key) {
        return layer_visible(&v.placement_layer);
    }
    if let Some(v) = scene.pads.iter().find(|v| v.object_id == key) {
        return if v.copper_layer_ids.is_empty() {
            layer_visible(&v.layer_id)
        } else {
            v.copper_layer_ids.iter().any(|layer| layer_visible(layer))
        };
    }
    if let Some(v) = scene.vias.iter().find(|v| v.object_id == key) {
        return layer_visible(&v.start_layer_id) || layer_visible(&v.end_layer_id);
    }
    let layer = scene
        .tracks
        .iter()
        .find(|v| v.object_id == key)
        .map(|v| v.layer_id.as_str())
        .or_else(|| {
            scene
                .zones
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| v.layer_id.as_str())
        })
        .or_else(|| {
            scene
                .board_graphics
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| v.layer_id.as_str())
        })
        .or_else(|| {
            scene
                .outline
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| v.layer_id.as_str())
        })
        .or_else(|| {
            scene
                .board_texts
                .iter()
                .find(|v| v.object_id == key)
                .map(|v| v.layer_id.as_str())
        });
    layer.is_some_and(layer_visible)
}

pub fn native_run_at(
    authority: &NativeSelectionResolution,
    origin: Id,
    point: Option<PointNm>,
) -> Result<crate::SelectionSubject, crate::selection_resolution::SelectionResolutionError> {
    authority.acquire_run(
        origin,
        point.map(|p| eda_engine::ir::geometry::Point::new(p.x, p.y)),
    )
}
