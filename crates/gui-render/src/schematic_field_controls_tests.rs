//! Shared form labels and hit regions distinguish metadata from canonical fields.
use super::*;
use datum_gui_protocol::schematic_authoring::{CustomFieldDraft, selected::SymbolEditDraft};
use datum_gui_protocol::{
    AuthoredSelectionClass, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent, PaneId,
};
use eda_engine::{
    ir::geometry::Point,
    schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode, SymbolField},
};
use uuid::Uuid;

#[test]
fn custom_metadata_shared_form_reports_display_limits_and_disables_removed_text() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.backing = Some(datum_gui_protocol::WorkspaceBacking {
        request: datum_gui_protocol::LiveReviewRequest {
            project_root: "/unused".into(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        board_path: "/unused/board.json".into(),
        native_write_verbs: ["datum.schematic.set_symbol_field".into()]
            .into_iter()
            .collect(),
    });
    let id = Uuid::new_v4();
    let symbol = SymbolEditDraft {
        target: AuthoredSelectionIdentity {
            class: AuthoredSelectionClass::Symbol,
            id,
            instance_path: vec![],
        },
        basis: AuthoringViewBasis {
            project: Uuid::new_v4(),
            revision: "test".into(),
            pane: PaneId(1),
            content: PaneContent::Schematic,
            source: Uuid::new_v4(),
            document: None,
        },
        tip: None,
        original_reference: "R1".into(),
        original_value: "22k".into(),
    };
    let original = PlacedSymbol {
        uuid: id,
        reference: "R1".into(),
        value: "22k".into(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        pins: vec![],
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
        fields: vec![SymbolField {
            uuid: Uuid::new_v4(),
            key: "Reference".into(),
            value: "metadata".into(),
            visible: true,
            position: Some(Point::new(123, 456)),
        }],
    };
    let e = &mut state.schematic_authoring;
    e.picker = true;
    e.editing = Some(FieldsEditDraft::Symbol(symbol.clone()));
    let layout = ShellLayout::for_window(1280, 800, None);
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    super::super::render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::SchematicCustomField(C::Open))
    );
    let mut draft = CustomFieldDraft {
        symbol,
        field_id: None,
        original,
        remove: false,
        visibility: None,
        placing: false,
    };
    for existing in [false, true] {
        draft.field_id = existing.then(|| draft.original.fields[0].uuid);
        state.schematic_authoring.editing = Some(FieldsEditDraft::Custom(draft.clone()));
        text.clear();
        hits.clear();
        super::super::render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        for label in [
            "Custom metadata",
            "Key",
            "Value",
            "New",
            "Apply",
            "Cancel",
            "Symbol fields…",
        ] {
            assert!(text.iter().any(|t| t.text == label), "missing {label}");
        }
        assert!(text.iter().any(|t| t.text
            == if existing {
                "Visible · manually positioned"
            } else {
                "Hidden / not drawn · no position"
            }));
        let controls: Vec<_> = hits
            .iter()
            .filter(|h| matches!(h.target, HitTarget::SchematicCustomField(_)))
            .collect();
        assert!(controls.iter().all(|h| h.rect.height > 0.));
        assert_eq!(
            hits.iter()
                .any(|h| h.target == HitTarget::SchematicCustomField(C::Hide)),
            existing
        );
        assert_eq!(
            hits.iter()
                .any(|h| h.target == HitTarget::SchematicCustomField(C::Place)),
            existing
        );
        if existing {
            assert!(text.iter().any(|t| t.text == "Move text…"));
        }
    }
    state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
    state.ui.layout.focused = PaneId(1);
    draft.placing = true;
    state.schematic_authoring.picker = false;
    state.schematic_authoring.editing = Some(FieldsEditDraft::Custom(draft.clone()));
    assert_eq!(
        datum_gui_protocol::EditorTool::active(&state),
        datum_gui_protocol::EditorTool::Field
    );
    text.clear();
    hits.clear();
    super::super::render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
    assert!(
        text.iter()
            .any(|t| t.text == "Move field text · click to place")
    );
    assert!(hits.iter().any(|h| h.target == HitTarget::SchematicCancel));
    assert!(!hits.iter().any(|h| matches!(
        h.target,
        HitTarget::SchematicReference | HitTarget::SchematicValue | HitTarget::SchematicEditApply
    )));
    draft.placing = false;
    state.schematic_authoring.picker = true;
    draft.remove = true;
    state.schematic_authoring.editing = Some(FieldsEditDraft::Custom(draft));
    text.clear();
    hits.clear();
    super::super::render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
    assert!(text.iter().any(|t| t.text.contains("Removal pending")));
    assert!(
        !state
            .schematic_authoring
            .editing
            .as_ref()
            .unwrap()
            .text_editable()
    );
    assert!(
        !hits
            .iter()
            .any(|h| h.target == HitTarget::SchematicCustomField(C::Remove))
    );
    state.backing.as_mut().unwrap().native_write_verbs.clear();
    text.clear();
    hits.clear();
    super::super::render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
    assert!(
        !hits
            .iter()
            .any(|h| matches!(h.target, HitTarget::SchematicCustomField(_)))
    );
}
