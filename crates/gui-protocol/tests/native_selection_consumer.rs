//! Native source-to-consumer binding, not screenshot-only selection evidence.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity, SelectionSubject,
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_scene::authored_candidate,
};
use eda_engine::{ir::geometry::Point, substrate::Operation};
use fixture::*;
fn wire(f: &mut Fixture, n: u128, from: i64, to: i64) {
    f.commit([Operation::CreateSchematicWire {sheet_id:f.sheet,wire_id:id(n),wire:serde_json::json!({"uuid":id(n),"from":{"x":from,"y":0},"to":{"x":to,"y":0},"style":{"width":0,"color":null,"dash":null}})}]);
}
#[test]
fn native_schematic_projects_current_source_and_click_identity_without_reimport_or_write() {
    let mut f = Fixture::new("current-native-schematic");
    wire(&mut f, 1, 0, 1_000_000);
    wire(&mut f, 2, 1_000_000, 2_000_000);
    let adoption = eda_engine::api::native_write::electrical::build_adopt_schematic_net_identities(
        &f.model,
        provenance(),
    )
    .unwrap();
    eda_engine::api::native_write::commit_prepared(&mut f.model, &f.root, adoption).unwrap();
    let files = f.source_bytes();
    let scene = datum_gui_protocol::project_native_schematic_scene(&f.model, &f.root)
        .unwrap()
        .unwrap();
    assert_eq!(scene.project_uuid, f.model.project.project_id.to_string());
    assert_eq!(scene.source_revision, f.model.model_revision.0);
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let hit = scene
        .board_graphics
        .iter()
        .find(|v| v.source_object_uuid == id(1).to_string())
        .unwrap();
    let origin = authored_candidate(&scene, &hit.object_id, &authority).unwrap();
    assert_eq!(
        origin,
        Identity {
            class: Class::Wire,
            id: id(1),
            instance_path: vec![]
        }
    );
    let subject = authority.acquire_run(origin, None).unwrap();
    let selected = authority.resolve(&subject).unwrap();
    assert_eq!(selected.members.len(), 2);
    let missing = SelectionSubject::Object(Identity {
        class: Class::Wire,
        id: id(999),
        instance_path: vec![],
    });
    assert!(authority.members(&missing).unwrap().is_empty());
    Fixture::assert_bytes(&files);
}
#[test]
fn native_binding_uses_declared_source_metadata_and_rejects_scene_only_identity() {
    let mut f = Fixture::new("source-binding");
    f.line(id(1), Point::new(0, 0), Point::new(10, 0));
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let mut track = scene.tracks[0].clone();
    track.object_id = "arbitrary-render-key".into();
    track.source_object_uuid = id(1).to_string();
    scene.tracks = vec![track];
    assert_eq!(
        authored_candidate(&scene, "arbitrary-render-key", &authority)
            .unwrap()
            .id,
        id(1)
    );
    scene.tracks[0].source_object_uuid = id(999).to_string();
    assert!(authored_candidate(&scene, "arbitrary-render-key", &authority).is_none());
}
