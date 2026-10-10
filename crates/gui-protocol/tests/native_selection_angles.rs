#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    SelectionSubject,
    selection_resolution::NativeSelectionResolution,
    selection_view::{InspectionValue, build_selection_view},
};
use eda_engine::{
    api::native_write::{commit_prepared, project},
    ir::units::{
        DecimalDegreePrecision, PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units,
    },
    substrate::Operation,
};
use fixture::{Fixture, id, provenance};

#[test]
fn native_text_angles_use_project_precision_without_changing_canonical_values() {
    let mut f = Fixture::new("text-angle-context");
    for (n, rotation) in [(1, 45), (3, 90)] {
        let text: eda_engine::board::BoardText = serde_json::from_value(serde_json::json!({
            "uuid":id(n),"text":"B","position":{"x":10,"y":20},
            "rotation":rotation,"layer":3,"height_nm":1234567,"stroke_width_nm":4567
        }))
        .unwrap();
        f.commit([Operation::CreateBoardText {
            text_id: id(n),
            text: serde_json::to_value(text).unwrap(),
        }]);
    }
    f.commit([Operation::CreateSchematicText {
        sheet_id: f.sheet,
        text_id: id(2),
        text: serde_json::json!({"uuid":id(2),"text":"S","position":{"x":30,"y":40},"rotation":45}),
    }]);
    let scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let target = |class, n| Id {
        class,
        id: id(n),
        instance_path: vec![],
    };
    let read = |f: &Fixture, subject| {
        let authority = NativeSelectionResolution::capture(&f.model).unwrap();
        build_selection_view(&f.model, &authority, subject, &scene, None, |_| true).unwrap()
    };
    let before = f.source_bytes();
    for class in [Class::BoardText, Class::SchematicText] {
        let view = read(
            &f,
            SelectionSubject::Object(target(class, if class == Class::BoardText { 1 } else { 2 })),
        );
        assert_eq!(
            view.field("text_rotation"),
            InspectionValue::Common(serde_json::json!(45))
        );
        assert!(
            view.field_display_preview("text_rotation", 200)
                .starts_with("Unavailable — Project quantity context unavailable:")
        );
    }
    Fixture::assert_bytes(&before);
    let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("fixture migration")
    };
    let initialize =
        project::build_initialize_project_display_units(&f.model, provenance(), profile, &receipt)
            .unwrap();
    commit_prepared(&mut f.model, &f.root, initialize).unwrap();
    for (precision, expected) in [
        (DecimalDegreePrecision::Decimal0, "45deg"),
        (DecimalDegreePrecision::Decimal3, "45.000deg"),
    ] {
        let mut profile = project::project_display_units(&f.model).unwrap();
        profile.angle_precision = precision;
        let set =
            project::build_set_project_display_units(&f.model, provenance(), profile).unwrap();
        commit_prepared(&mut f.model, &f.root, set).unwrap();
        let before = f.source_bytes();
        for class in [Class::BoardText, Class::SchematicText] {
            let view = read(
                &f,
                SelectionSubject::Object(target(
                    class,
                    if class == Class::BoardText { 1 } else { 2 },
                )),
            );
            assert_eq!(
                view.field("text_rotation"),
                InspectionValue::Common(serde_json::json!(45))
            );
            assert_eq!(view.field_display_preview("text_rotation", 200), expected);
        }
        let mut view = read(
            &f,
            SelectionSubject::Compound(
                CompoundSelection::new(
                    [target(Class::BoardText, 1), target(Class::BoardText, 3)],
                    None,
                )
                .unwrap(),
            ),
        );
        view.scope = Some(Class::BoardText);
        assert_eq!(view.field("text_rotation"), InspectionValue::Mixed);
        assert_eq!(view.field_display_preview("text_rotation", 200), "Mixed");
        Fixture::assert_bytes(&before);
    }
}
