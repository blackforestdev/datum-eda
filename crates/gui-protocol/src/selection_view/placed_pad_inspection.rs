//! Read-only materialization status in the existing typed Inspector vocabulary.
use super::*;
use eda_engine::substrate::DesignModel;
pub(super) fn append(view: &mut SelectionView, model: &DesignModel, board: Option<&Value>) {
    use eda_engine::substrate::component_pad_source::{
        ComponentPadMaterializationStatus as Status, ComponentPadSource,
    };
    let Some(board) = board else {
        return;
    };
    if view.members.iter().any(|id| id.class == Class::Pad) {
        if let Ok(projection) =
            eda_engine::substrate::component_pad_source::component_pad_projection(model, board)
        {
            for id in view.members.iter().filter(|id| id.class == Class::Pad) {
                if let Some(p) = projection.query_pads.get(&id.id) {
                    let values = view.values.entry(id.clone()).or_default();
                    let local = p.terminal();
                    values.extend(properties::values(
                        Class::Pad,
                        &serde_json::to_value(local).unwrap(),
                    ));
                    if let eda_engine::board::query_pad::QueryPadGeometry::SourceRelative(exact) = p
                    {
                        values.insert(
                            "local_source_position_nm".into(),
                            serde_json::to_value(local.position).unwrap(),
                        );
                        values.insert(
                            "terminal_position_nm".into(),
                            serde_json::json!({"x":exact.center[0],"y":exact.center[1]}),
                        );
                        view.authored_anchors.remove(id);
                    }
                }
            }
        }
    }
    for id in &view.members {
        if id.class != Class::Footprint {
            continue;
        }
        let key = id.id.to_string();
        let mut status = Status::Current;
        let pads = board["component_pads"][&key].as_array();
        if let Some(pads) = pads.filter(|p| !p.is_empty()) {
            let part = serde_json::from_value(board["packages"][&key]["part"].clone());
            let package = serde_json::from_value(board["packages"][&key]["package"].clone());
            if let (Ok(part), Ok(package)) = (part, package) {
                for value in pads {
                    let s = serde_json::from_value::<ComponentPadSource>(value.clone())
                        .map_or(Status::Unverified, |p| {
                            p.materialization_status(model, part, package)
                        });
                    if s == Status::Stale {
                        status = s;
                        break;
                    }
                    if s == Status::Unverified {
                        status = s;
                    }
                }
            } else {
                status = Status::Unverified;
            }
        } else {
            status = Status::Unverified;
        }
        view.values.entry(id.clone()).or_default().insert(
            "pad_materialization_status".into(),
            serde_json::to_value(status).unwrap(),
        );
    }
}
