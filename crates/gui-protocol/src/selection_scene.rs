//! Typed hit/source binding. Scene strings never establish authored identity.
mod index;
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, BoardReviewSceneV1, PointNm,
};
pub use index::{SceneSelectionBinding, SceneSelectionBindings, acquisition_eligible};

pub fn projection_candidate(
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
    if let Some(pad) = scene.pads.iter().find(|v| v.object_id == key) {
        source = Some((Class::Pad, pad.pad_uuid.parse().ok()?));
    }
    if let Some(text) = scene.board_texts.iter().find(|v| v.object_id == key)
        && let Some(uuid) = text
            .source_object_uuid
            .as_ref()
            .and_then(|s| s.parse::<crate::SelectionProjectId>().ok())
    {
        source = authority
            .authored_identities()
            .iter()
            .find(|id| id.id == uuid && id.instance_path.is_empty())
            .map(|id| (id.class, id.id));
    }
    if let Some(graphic) = scene.board_graphics.iter().find(|v| v.object_id == key) {
        if graphic.object_kind == "board_outline" {
            source = Some((
                Class::BoardOutline,
                graphic.source_object_uuid.parse().ok()?,
            ));
        } else if graphic.object_kind == "board_graphic" {
            source = Some((
                Class::BoardGraphic,
                graphic.source_object_uuid.parse().ok()?,
            ));
        } else {
            let class = match graphic.schematic_hit_kind()? {
                crate::SchematicHitKind::Symbol => Class::Symbol,
                crate::SchematicHitKind::Wire => Class::Wire,
                crate::SchematicHitKind::Bus => Class::BusSection,
                crate::SchematicHitKind::Label => Class::Label,
                crate::SchematicHitKind::Port => Class::Port,
                crate::SchematicHitKind::Junction => Class::Junction,
                crate::SchematicHitKind::NoConnect => Class::NoConnect,
                crate::SchematicHitKind::Pin => Class::Pin,
                crate::SchematicHitKind::Drawing => Class::SchematicGraphic,
            };
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

/// Placement-workspace acquisition collapses only actual owned children.
pub fn authored_candidate(
    scene: &BoardReviewSceneV1,
    key: &str,
    authority: &NativeSelectionResolution,
) -> Option<Id> {
    let mut id = projection_candidate(scene, key, authority)?;
    match id.class {
        Class::Pad if !authority.is_standalone_pad(&id) => {
            let pad = scene.pads.iter().find(|p| p.object_id == key)?;
            id = Id {
                class: Class::Footprint,
                id: pad.component_uuid.parse().ok()?,
                instance_path: vec![],
            };
        }
        Class::Pin => {
            let sheet = authority
                .snapshot()
                .schematic_source()?
                .sheets
                .get(&scene.board_uuid.parse().ok()?)?;
            let parent = sheet
                .symbols
                .values()
                .find(|s| s.pins.iter().any(|p| p.uuid == id.id))?;
            id = Id {
                class: Class::Symbol,
                id: parent.uuid,
                instance_path: vec![],
            };
        }
        _ => {}
    }
    authority.authored_contains(&id).then_some(id)
}
