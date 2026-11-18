//! The selected native Library source uses the existing Inspector rows and basis.
use super::*;
use crate::library_browser::LibraryEntry;
use eda_engine::substrate::SourceShardKind;

/// Disposable presentation identity, built only from the actual current source
/// and inspection epoch. It is neither asset history nor engine write authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryInspectorBasis {
    pub key: String,
    pub asset: Uuid,
    pub kind: String,
    pub source_path: String,
    pub source_shard: Uuid,
    pub source_hash: String,
    pub object_revision: u64,
    pub inspection_id: Uuid,
    pub common_inspection_current: bool,
}

impl LibraryInspectorBasis {
    pub(super) fn capture(state: &ReviewWorkspaceState) -> Option<Self> {
        let entry = selected_library_entry(state)?;
        let model = state.library.basis.as_ref()?;
        let object = model.objects.get(&entry.id)?;
        let source = model
            .source_shards
            .iter()
            .find(|source| source.shard_id == object.source_shard_id)?;
        Some(Self {
            key: entry.key(),
            asset: entry.id,
            kind: entry.kind.clone(),
            source_path: entry.source_path.clone(),
            source_shard: source.shard_id,
            source_hash: source.content_hash.clone(),
            object_revision: object.object_revision.0,
            inspection_id: state.library.revision_inspection_id,
            common_inspection_current: state.library.current_revision_inspection().is_some(),
        })
    }
}

/// One admission rule for projection, renderer and metadata navigation. Drafts,
/// previews and Navigator keep their existing owners and custom presentation.
/// Missing/stale common records never hide a qualified native source's metadata.
pub fn selected_library_entry(state: &ReviewWorkspaceState) -> Option<&LibraryEntry> {
    let browser = &state.library;
    if state.inspector_context.owner != InspectorOwner::Selection
        || state.ui.layout.focused_content() != PaneContent::Library
        || browser.draft.is_some()
        || browser.configuration.is_some()
        || browser.resolution.is_some()
        || browser.acquisition.is_some()
        || browser.revision_workflow.is_some()
        || !crate::CheckProducerRequest::workspace_available(state)
    {
        return None;
    }
    let entry = browser.selected(&state.selection)?;
    if entry.warning.is_some()
        || browser
            .entries
            .iter()
            .filter(|other| other.source_path == entry.source_path)
            .count()
            != 1
        || entry
            .raw
            .get("uuid")
            .and_then(serde_json::Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            != Some(entry.id)
    {
        return None;
    }
    let model = browser.basis.as_ref()?;
    let object = model.objects.get(&entry.id)?;
    let mut sources = model.source_shards.iter().filter(|source| {
        source.relative_path == entry.source_path && source.kind == SourceShardKind::Pool
    });
    let source = sources.next()?;
    (sources.next().is_none()
        && object.source_shard_id == source.shard_id
        && object.domain == "pool"
        && object.kind == entry.kind
        && entry.revision == Some(object.object_revision.0))
    .then_some(entry)
}

pub(super) fn append<'a>(
    context: &mut InspectorContext<'a>,
    state: &'a ReviewWorkspaceState,
    entry: &'a LibraryEntry,
) {
    context.title = format!("Library · {}", entry.name);
    for line in state.library.common_revision_summary(entry) {
        push_line(context, InspectorSectionId::LibraryInfo, line);
    }
    context.push(
        InspectorSectionId::LibraryInfo,
        InspectorField::text(
            "Inspection authority",
            "Declared metadata does not establish engineering approval or redistribution rights",
        ),
    );
    context.push(
        InspectorSectionId::LibraryInfo,
        InspectorField::text(
            "Check currency",
            "This inspection does not establish authoritative check freshness",
        ),
    );
    for (label, value) in [
        ("Name", entry.name.clone()),
        ("Kind", entry.label().to_owned()),
        ("Source", entry.source_path.clone()),
        ("Asset identity", entry.id.to_string()),
        (
            "Native source revision",
            entry
                .revision
                .map_or_else(|| "Unavailable".into(), |value| value.to_string()),
        ),
    ] {
        context.push(
            InspectorSectionId::Source,
            InspectorField::text(label, value),
        );
    }
    // Keep complete common/provenance/review records and all native fields.
    // Existing Inspector section collapse, wrapping and virtual rows own access.
    for line in state.library.detail_lines(entry) {
        push_line(context, InspectorSectionId::Properties, line);
    }
}

fn push_line(context: &mut InspectorContext<'_>, section: InspectorSectionId, line: String) {
    let (label, value) = line.split_once(": ").map_or_else(
        || ("Summary".to_owned(), line.clone()),
        |(label, value)| (label.to_owned(), value.to_owned()),
    );
    context.push(
        section,
        InspectorField {
            label: Cow::Owned(label),
            value: InspectorValue::Text(Cow::Owned(value)),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::{
        api::native_write::{
            genesis::{GenesisSpec, bootstrap_native_project},
            library_assets::inspect_local_library,
            registry::NativeWriteContext,
        },
        substrate::ProjectResolver,
    };
    use serde_json::json;
    use std::{path::PathBuf, sync::Arc};

    fn workspace() -> (PathBuf, ReviewWorkspaceState) {
        let root = std::env::temp_dir().join(format!("datum-library-inspector-{}", Uuid::new_v4()));
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Library Inspector controls".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let project = root.join("project.json");
        let mut header: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&project).unwrap()).unwrap();
        header["pools"] = json!([{"path":"pool","priority":1}]);
        std::fs::write(project, serde_json::to_vec(&header).unwrap()).unwrap();
        for index in 0..2 {
            let id = Uuid::from_u128(100 + index);
            let path = root.join(format!("pool/units/{id}.json"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            // Authored fixture input before initial resolution, never a model
            // stamped with an invented journal, revision or accepted frame.
            std::fs::write(
                path,
                serde_json::to_vec(&json!({"schema_version":1,"uuid":id,
                "name":format!("Unit {index}"),"manufacturer":"","pins":{},
                "tags":(0..250).map(|row|format!("retained-note-{row}")).collect::<Vec<_>>()}))
                .unwrap(),
            )
            .unwrap();
        }
        let model = Arc::new(ProjectResolver::new(&root).resolve().unwrap());
        let library = inspect_local_library(&NativeWriteContext {
            model: &model,
            project_root: &root,
        })
        .unwrap();
        let description = json!({"project_id":model.project.project_id,"model_revision":model.model_revision,
            "transaction_tip":eda_engine::revision::transaction_tip(&model.journal),"library":library});
        let mut state = crate::load_fixture_workspace_state();
        state.project_unopened = false;
        state.scene.project_uuid = model.project.project_id.to_string();
        state.scene.source_revision = model.model_revision.0.clone();
        state.schematic_documents.project = Some(model.project.project_id);
        state.ui.layout = crate::WorkspaceLayout::single();
        state.ui.layout.set_focused_content(PaneContent::Library);
        state.inspector_context = InspectorNavigation::default();
        state.library = crate::library_browser::LibraryBrowser::capture(model);
        state.library.set_revision_inspection(description).unwrap();
        state.selection = SelectionTarget::AuthoredObject(state.library.entries[0].key());
        state.backing = Some(crate::WorkspaceBacking {
            request: crate::LiveReviewRequest {
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
            native_write_verbs: Default::default(),
        });
        (root, state)
    }

    #[test]
    fn library_context_keeps_long_native_metadata_when_common_records_are_unavailable() {
        let (root, mut state) = workspace();
        state.library.revision_inspection = None;
        assert!(selected_library_entry(&state).is_some());
        let context = InspectorContext::capture(&state);
        assert!(context.title.starts_with("Library ·"));
        assert_eq!(context.sections[0].id, InspectorSectionId::LibraryInfo);
        assert!(context.action_basis.as_ref().unwrap().library.is_some());
        assert!(!context.rerun_available);
        assert!(context.visible_row_count(&state.inspector_context) > 250);
        let text = context
            .sections
            .iter()
            .flat_map(|section| &section.fields)
            .map(|field| context.field_text(field, usize::MAX))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("unavailable for this snapshot"));
        assert!(text.contains("retained-note-249"));
        assert!(text.contains("pool/units/"));
        assert!(text.contains("does not establish engineering approval or redistribution rights"));
        assert!(text.contains("does not establish authoritative check freshness"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn library_navigation_rejects_other_entry_epoch_source_and_stale_model_view() {
        let (root, mut state) = workspace();
        let context = InspectorContext::capture(&state);
        let basis = InspectorNavigationBasis::for_context(&state, &context, None);
        assert!(basis.is_current(&state));
        let selected = state.selection.clone();
        state.selection = SelectionTarget::AuthoredObject(state.library.entries[1].key());
        assert!(!basis.is_current(&state));
        state.selection = selected;
        let epoch = state.library.revision_inspection_id;
        state.library.revision_inspection_id = Uuid::new_v4();
        assert!(!basis.is_current(&state));
        state.library.revision_inspection_id = epoch;
        let inspection = state.library.revision_inspection.take();
        assert!(selected_library_entry(&state).is_some());
        assert!(!basis.is_current(&state));
        state.library.revision_inspection = inspection;
        assert!(basis.is_current(&state));
        let original = state.library.entries[0].clone();
        state.library.entries[0].revision = original.revision.map(|revision| revision + 1);
        assert!(selected_library_entry(&state).is_none());
        assert!(!basis.is_current(&state));
        state.library.entries[0] = original.clone();
        state.library.entries[0].source_path = "pool/units/wrong-owner.json".into();
        state.selection = SelectionTarget::AuthoredObject(state.library.entries[0].key());
        assert!(selected_library_entry(&state).is_none());
        state.library.entries[0] = original.clone();
        state.selection = SelectionTarget::AuthoredObject(original.key());
        state.library.entries.push(original);
        assert!(selected_library_entry(&state).is_none());
        state.library.entries.pop();
        state.scene.source_revision = "stale presentation, unchanged accepted model".into();
        assert!(selected_library_entry(&state).is_none());
        assert!(!basis.is_current(&state));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn library_inspector_admission_preserves_navigator_draft_preview_and_pane_owners() {
        let (root, mut state) = workspace();
        assert!(selected_library_entry(&state).is_some());
        state.inspector_context.owner = InspectorOwner::Navigator;
        assert!(selected_library_entry(&state).is_none());
        assert!(
            !InspectorContext::capture(&state)
                .title
                .starts_with("Library ·")
        );
        state.inspector_context.owner = InspectorOwner::Selection;
        state.ui.layout.set_focused_content(PaneContent::Board);
        assert!(selected_library_entry(&state).is_none());
        state.ui.layout.set_focused_content(PaneContent::Library);
        let model = state.library.basis.as_ref().unwrap();
        state.library.draft = Some(
            crate::library_authoring::LibraryDraft::begin(
                &state.library,
                None,
                "units",
                model.model_revision.0.clone(),
                eda_engine::revision::transaction_tip(&model.journal),
            )
            .unwrap(),
        );
        assert!(selected_library_entry(&state).is_none());
        state.library.draft.as_mut().unwrap().preview = Some(state.library.entries[0].raw.clone());
        assert!(selected_library_entry(&state).is_none());
        state.library.draft = None;
        assert!(selected_library_entry(&state).is_some());
        std::fs::remove_dir_all(root).unwrap();
    }
}
