//! Track deletion configures shared selection and native-write ownership.
//! Menu admission reads cached consumer state; only the engine reads source payloads.
use crate::{
    AuthoredSelectionClass as Class, AuthoringViewBasis, PaneContent, ReviewWorkspaceState,
    SelectionSubject, SelectionTarget,
    native_edit_client::{NativeEditAction, NativeEditIntent},
};
use eda_engine::api::native_write::board_routing::track_delete::VERB;
use uuid::Uuid;

fn context(state: &ReviewWorkspaceState) -> Result<AuthoringViewBasis, String> {
    let backing = state
        .backing
        .as_ref()
        .ok_or("Open a native Project before deleting Tracks")?;
    if backing.request.board_file.is_some() || backing.request.kicad_board_source.is_some() {
        return Err("Track deletion requires a native Project".into());
    }
    let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
    if basis.content != PaneContent::Board {
        return Err("Focus the native Board before deleting a Track".into());
    }
    if let Some(reason) = crate::gui_menu_model::native_edit_unavailable_reason(state, VERB) {
        return Err(reason.into());
    }
    if !crate::CheckProducerRequest::editor_preview_idle(state)
        || state.ui.board_transfer.is_some()
        || state.library.acquisition.is_some()
    {
        return Err("Finish or cancel the current edit before deleting a Track".into());
    }
    if let SelectionTarget::Native(view) = &state.selection {
        if view.stale || view.project != basis.project || view.revision.0 != basis.revision {
            return Err("Selection changed; select the current Track again".into());
        }
        if view.members.len() > 1
            || (view.members.len() == 1 && !matches!(&view.subject, SelectionSubject::Object(_)))
        {
            return Err(
                "Track deletion requires one explicit object, not a group or electrical scope"
                    .into(),
            );
        }
    }
    Ok(basis)
}

/// Tool activation shares the same current native source and preview admission.
pub fn available(state: &ReviewWorkspaceState) -> bool {
    context(state).is_ok()
}

/// Shared Edit/Delete and marking-menu admission for an actual source selection.
pub fn selected_track(state: &ReviewWorkspaceState) -> Result<Uuid, String> {
    context(state)?;
    let SelectionTarget::Native(view) = &state.selection else {
        return Err("Select one current native PCB Track".into());
    };
    let SelectionSubject::Object(id) = &view.subject else {
        return Err("Select one current native PCB Track".into());
    };
    if view.members.len() != 1
        || id.class != Class::Track
        || !id.instance_path.is_empty()
        || !view.members.contains(id)
    {
        return Err("This Delete action supports one native PCB Track".into());
    }
    if view.locked.contains(id)
        || view.hidden.contains(id)
        || view.unprojected.contains(id)
        || view.dropped.contains(id)
    {
        return Err("Select one visible, unlocked and current Track".into());
    }
    Ok(id.id)
}

/// An explicit pointer target is still an immutable claim, never a raw writer.
/// The native engine rechecks class, exact payload and common revision/tip guards.
pub fn capture(state: &ReviewWorkspaceState, track: Uuid) -> Result<NativeEditIntent, String> {
    let basis = context(state)?;
    if !state
        .scene
        .tracks
        .iter()
        .any(|item| item.source_object_uuid == track.to_string())
    {
        return Err("Track is not present in the current Board view".into());
    }
    if let SelectionTarget::Native(view) = &state.selection {
        if view
            .locked
            .iter()
            .chain(&view.hidden)
            .chain(&view.unprojected)
            .chain(&view.dropped)
            .any(|id| id.class == Class::Track && id.id == track)
        {
            return Err("Track is hidden, locked or no longer current".into());
        }
    }
    let model = state
        .library
        .basis
        .as_ref()
        .ok_or("Native source unavailable")?;
    let object = model.objects.get(&track).ok_or("Track no longer exists")?;
    let source = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .ok_or("Track source unavailable")?;
    if object.domain != "board" || source.kind != eda_engine::substrate::SourceShardKind::BoardRoot
    {
        return Err("Track is outside this native Board source".into());
    }
    let root = state.backing.as_ref().unwrap().request.project_root.clone();
    Ok(NativeEditIntent::new(
        root,
        basis.project,
        basis.revision,
        eda_engine::revision::transaction_tip(&model.journal),
        NativeEditAction::DeleteBoardTrack { track },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection_resolution::NativeSelectionResolution;
    use crate::{
        AuthoredSelectionIdentity as Id, CompoundSelection, LiveReviewRequest, WorkspaceBacking,
    };
    use eda_engine::{
        api::native_write as native,
        substrate::{CommitSource, ProjectResolver, SourceShardKind},
    };
    use std::{path::PathBuf, sync::Arc};

    struct Fixture {
        root: PathBuf,
        state: ReviewWorkspaceState,
        ids: Vec<Id>,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir()
                .join(format!("datum-track-delete-consumer-{}", Uuid::new_v4()));
            crate::known_good_demo::write_known_good_demo_project(&root).unwrap();
            let mut model = ProjectResolver::new(&root).resolve().unwrap();
            let board = model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap();
            let mut extra: eda_engine::board::Track = serde_json::from_value(
                board["tracks"]
                    .as_object()
                    .unwrap()
                    .values()
                    .next()
                    .unwrap()
                    .clone(),
            )
            .unwrap();
            extra.uuid = Uuid::new_v4();
            extra.from.y += 2_000_000;
            extra.to.y += 2_000_000;
            let write = native::board_routing::build_place_board_track(
                &model,
                native::WriteProvenance::new(
                    "delete-consumer-test",
                    CommitSource::Manual,
                    "Prepare selection fixture",
                ),
                &extra,
            )
            .unwrap();
            native::commit_prepared(&mut model, &root, write).unwrap();
            let model = Arc::new(ProjectResolver::new(&root).resolve().unwrap());
            let board = model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap();
            let mut state = crate::load_fixture_workspace_state();
            state.project_unopened = false;
            state.ui.layout = crate::WorkspaceLayout::single();
            state.scene.project_uuid = model.project.project_id.to_string();
            state.scene.source_revision = model.model_revision.0.clone();
            state.scene.board_uuid = board["uuid"].as_str().unwrap().into();
            state.scene.tracks = board["tracks"]
                .as_object()
                .unwrap()
                .values()
                .map(|raw| {
                    let track: eda_engine::board::Track =
                        serde_json::from_value(raw.clone()).unwrap();
                    crate::TrackPrimitive {
                        object_id: format!("track:{}", track.uuid),
                        object_kind: "track".into(),
                        source_object_uuid: track.uuid.to_string(),
                        track_uuid: track.uuid.to_string(),
                        net_uuid: Some(track.net.to_string()),
                        layer_id: format!("L{}", track.layer),
                        width_nm: track.width,
                        path: vec![
                            crate::PointNm {
                                x: track.from.x,
                                y: track.from.y,
                            },
                            crate::PointNm {
                                x: track.to.x,
                                y: track.to.y,
                            },
                        ],
                    }
                })
                .collect();
            let authority = NativeSelectionResolution::capture_shared(model.clone()).unwrap();
            let ids = authority
                .authored_identities()
                .iter()
                .filter(|id| id.class == Class::Track)
                .cloned()
                .collect();
            state.library = crate::library_browser::LibraryBrowser::capture(model);
            state.backing = Some(WorkspaceBacking {
                request: LiveReviewRequest {
                    project_root: root.clone(),
                    board_file: None,
                    artifact_path: None,
                    net_uuid: None,
                    from_anchor_pad_uuid: None,
                    to_anchor_pad_uuid: None,
                    profile: None,
                    kicad_board_source: None,
                },
                board_path: root.join("board/board.json"),
                native_write_verbs: [VERB.into()].into_iter().collect(),
            });
            let mut result = Self { root, state, ids };
            result.select(SelectionSubject::Object(result.ids[0].clone()));
            result
        }
        fn select(&mut self, subject: SelectionSubject) {
            let model = self.state.library.basis.as_ref().unwrap();
            let authority = NativeSelectionResolution::capture_shared(model.clone()).unwrap();
            self.state.selection = SelectionTarget::Native(Box::new(
                crate::selection_view::build_selection_view(
                    model,
                    &authority,
                    subject,
                    &self.state.scene,
                    None,
                    |_| true,
                )
                .unwrap(),
            ));
        }
        fn durable(&self) -> Vec<(PathBuf, Option<Vec<u8>>)> {
            [
                self.root.join("board/board.json"),
                eda_engine::substrate::transaction_journal_path(&self.root),
                self.root.join(".datum/journal/cursor.json"),
            ]
            .into_iter()
            .map(|path| {
                let bytes = std::fs::read(&path).ok();
                (path, bytes)
            })
            .collect()
        }
    }

    #[test]
    fn selected_track_intent_uses_current_project_revision_and_tip_without_writes() {
        let f = Fixture::new();
        let before = f.durable();
        let id = selected_track(&f.state).unwrap();
        let intent = capture(&f.state, id).unwrap();
        let model = f.state.library.basis.as_ref().unwrap();
        assert_eq!(intent.project_root, f.root);
        assert_eq!(intent.project_id, model.project.project_id);
        assert_eq!(intent.expected_revision, model.model_revision.0);
        assert_eq!(
            intent.expected_tip,
            eda_engine::revision::transaction_tip(&model.journal)
        );
        assert!(matches!(intent.action,NativeEditAction::DeleteBoardTrack{track} if track==id));
        assert!(!intent.action.is_move());
        assert_eq!(f.durable(), before);
    }

    #[test]
    fn stale_locked_and_multiple_selection_cannot_become_delete_intent() {
        let mut f = Fixture::new();
        let before = f.durable();
        let id = f.ids[0].clone();
        let valid = f.state.clone();
        if let SelectionTarget::Native(view) = &mut f.state.selection {
            view.stale = true;
        }
        assert!(capture(&f.state, id.id).is_err());
        f.state = valid.clone();
        if let SelectionTarget::Native(view) = &mut f.state.selection {
            view.locked.insert(id.clone());
        }
        assert!(selected_track(&f.state).is_err());
        assert!(capture(&f.state, id.id).is_err());
        f.state = valid;
        assert!(f.ids.len() >= 2);
        f.select(SelectionSubject::Compound(
            CompoundSelection::new(f.ids.iter().take(2).cloned().collect::<Vec<_>>(), None)
                .unwrap(),
        ));
        assert!(selected_track(&f.state).is_err());
        assert!(capture(&f.state, id.id).is_err());
        assert_eq!(f.durable(), before);
    }

    #[test]
    fn stale_scene_imported_backing_and_missing_capability_refuse() {
        let mut f = Fixture::new();
        let id = f.ids[0].id;
        let before = f.durable();
        let valid = f.state.clone();
        f.state.scene.source_revision.push_str("-stale");
        assert!(capture(&f.state, id).is_err());
        f.state = valid.clone();
        f.state.backing.as_mut().unwrap().request.board_file =
            Some(f.root.join("foreign.kicad_pcb"));
        assert!(capture(&f.state, id).is_err());
        f.state = valid;
        f.state.backing.as_mut().unwrap().native_write_verbs.clear();
        assert!(capture(&f.state, id).is_err());
        assert_eq!(f.durable(), before);
    }
}
