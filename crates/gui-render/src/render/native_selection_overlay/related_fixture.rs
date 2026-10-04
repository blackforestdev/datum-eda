//! Canonical electrical subject with an engine-related parent and foreign geometry.
use super::*;
use eda_engine::{
    api::native_write::{self, WriteProvenance},
    schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode, SymbolPin},
    substrate::{CommitSource, ElectricalOccurrence},
};
pub(crate) fn selected_related_state() -> (Fixture, ReviewWorkspaceState) {
    let (mut fixture, mut state) = selected_schematic_text_state();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let sheet = authority
        .snapshot()
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap()
        .uuid;
    let pin = uuid::Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: uuid::Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "U1".into(),
        value: "Context".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: pin,
            library_pin: None,
            number: "1".into(),
            name: "P".into(),
            electrical_type: eda_engine::schematic::PinElectricalType::Passive,
            position: Point::new(0, 0),
        }],
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let write = native_write::schematic_symbols::build_place_schematic_symbol(
        &fixture.model,
        WriteProvenance::new(
            "selection-related-fixture",
            CommitSource::Test,
            "source-related parent baseline",
        ),
        sheet,
        &symbol,
        None,
    )
    .unwrap();
    let root = fixture.root().to_path_buf();
    native_write::commit_prepared(&mut fixture.model, &root, write).unwrap();
    let wire = eda_engine::schematic::SchematicWire {
        uuid: uuid::Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(5_000_000, 0),
    };
    let write = native_write::schematic_connectivity::build_create_schematic_wire(
        &fixture.model,
        WriteProvenance::new(
            "selection-related-fixture",
            CommitSource::Test,
            "explicit conductive pin connection",
        ),
        sheet,
        &wire,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &root, write).unwrap();
    let write = native_write::electrical::build_adopt_schematic_net_identities(
        &fixture.model,
        WriteProvenance::new(
            "selection-related-fixture",
            CommitSource::Test,
            "explicit canonical Net adoption",
        ),
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &root, write).unwrap();
    let nets: Vec<_> = fixture
        .model
        .electrical_identities
        .values()
        .filter_map(|r| {
            matches!(
                r.identity,
                eda_engine::substrate::ElectricalIdentity::Net { retired: false, .. }
            )
            .then_some(r.id)
        })
        .collect();
    assert_eq!(nets.len(), 1, "one adopted connected circuit");
    let record = eda_engine::substrate::ElectricalIdentityRecord {
        id: uuid::Uuid::new_v4(),
        object_revision: eda_engine::substrate::ObjectRevision(0),
        identity: eda_engine::substrate::ElectricalIdentity::NetRelationship {
            logical_net: nets[0],
            board_net: None,
            intent: eda_engine::substrate::NetRelationshipIntent::SchematicOnly,
            evidence: vec![],
        },
    };
    let write = native_write::electrical::build_create_electrical_identity(
        &fixture.model,
        WriteProvenance::new(
            "selection-related-fixture",
            CommitSource::Test,
            "explicit schematic-only correspondence",
        ),
        record,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &root, write).unwrap();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let net = authority
        .snapshot()
        .net_for_origin(
            &fixture.model.model_revision,
            &ElectricalOccurrence {
                class: "pins".into(),
                source_id: pin,
                instance_path: vec![],
            },
        )
        .unwrap();
    let scene = datum_gui_protocol::project_native_schematic_scene(&fixture.model, fixture.root())
        .unwrap()
        .unwrap();
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let view = build_selection_view(
        &fixture.model,
        &authority,
        SelectionSubject::GlobalNet(net.subject_id),
        &state.scene,
        Some(&scene),
        |_| true,
    )
    .unwrap();
    assert!(view.members.contains(&Id {
        class: Class::Pin,
        id: pin,
        instance_path: vec![]
    }));
    let parent = Id {
        class: Class::Symbol,
        id: symbol.uuid,
        instance_path: vec![],
    };
    assert!(view.related.contains(&parent));
    assert!(!view.members.contains(&parent));
    assert!(!view.related_schematic_keys.is_empty());
    state.schematic_scene = Some(scene);
    state.selection = SelectionTarget::Native(Box::new(view));
    (fixture, state)
}
