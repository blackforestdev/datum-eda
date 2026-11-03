use super::*;
use eda_engine::substrate::{ComponentSelectionBinding, ComponentSelectionRelation};
use uuid::Uuid;

fn identity(class: Class) -> Id {
    Id {
        class,
        id: Uuid::new_v4(),
        instance_path: vec![],
    }
}
fn occurrence(id: &Id) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: if id.class == Class::Symbol {
            "symbols"
        } else {
            "packages"
        }
        .into(),
        source_id: id.id,
        instance_path: id.instance_path.clone(),
    }
}
fn view(ids: &[Id]) -> SelectionView {
    let mut view = SelectionView {
        project: Uuid::new_v4(),
        revision: eda_engine::substrate::ModelRevision("binding-inspector-basis".into()),
        subject: SelectionSubject::Compound(
            crate::CompoundSelection::new(ids.iter().cloned(), None).unwrap(),
        ),
        members: ids.iter().cloned().collect(),
        related: BTreeSet::new(),
        scope: None,
        values: BTreeMap::new(),
        authored_anchors: BTreeMap::new(),
        inspection: vec![],
        zone_regions: BTreeMap::new(),
        board_keys: BTreeSet::new(),
        schematic_keys: BTreeSet::new(),
        schematic_documents: vec![],
        board_presentations: BTreeMap::new(),
        schematic_presentations: BTreeMap::new(),
        related_board_keys: BTreeSet::new(),
        related_schematic_keys: BTreeSet::new(),
        hidden: BTreeSet::new(),
        unprojected: BTreeSet::new(),
        locked: BTreeSet::new(),
        notice: None,
        dropped: BTreeSet::new(),
        stale: false,
    };
    view.inspection = inspection::compile(&view);
    view
}
fn field<'a>(view: &'a SelectionView, scope: Option<Class>, name: &str) -> &'a InspectionValue {
    &view
        .inspection
        .iter()
        .find(|s| s.scope == scope)
        .unwrap()
        .fields[name]
}

#[test]
fn schematic_binding_inspector_distinguishes_qualified_owner_from_missing_counterpart() {
    let symbol = identity(Class::Symbol);
    let source = occurrence(&symbol);
    let owner = Uuid::new_v4();
    let mut context = ComponentSelectionContext::default();
    context.bindings.insert(
        source.clone(),
        ComponentSelectionBinding {
            component_instance: owner,
            counterpart_status: ComponentCounterpartStatus::NotDeclared,
        },
    );
    let mut selected = view(&[symbol.clone()]);
    append(&mut selected, &context);
    for scope in [None, Some(Class::Symbol)] {
        assert_eq!(
            field(&selected, scope, "Related component").display_preview(100),
            "Schematic binding — no board counterpart"
        );
        let InspectionValue::Common(details) = field(&selected, scope, "Component bindings") else {
            panic!()
        };
        assert_eq!(
            details["bindings"][0]["binding"]["component_instance"],
            owner.to_string()
        );
        assert_eq!(details["relations"].as_array().unwrap().len(), 0);
        assert_eq!(details["unavailable"].as_array().unwrap().len(), 0);
        assert!(
            !field(&selected, scope, "Related component")
                .display_preview(100)
                .contains(&owner.to_string())
        );
    }
    assert_eq!(selected.members, [symbol.clone()].into());
    assert!(selected.related.is_empty());

    // A declared but unavailable counterpart does not inherit the healthy label.
    context
        .bindings
        .get_mut(&source)
        .unwrap()
        .counterpart_status = ComponentCounterpartStatus::Unavailable;
    context
        .unavailable
        .insert(source.clone(), "Declared counterpart is stale".into());
    append(&mut selected, &context);
    assert_eq!(
        field(&selected, None, "Related component"),
        &InspectionValue::Unavailable("Declared counterpart is stale".into())
    );

    // A qualified owner of the same source in another occurrence is not ours.
    context.bindings.clear();
    context.unavailable.clear();
    let mut other = source.clone();
    other.instance_path.push(Uuid::new_v4());
    context.bindings.insert(
        other,
        ComponentSelectionBinding {
            component_instance: owner,
            counterpart_status: ComponentCounterpartStatus::NotDeclared,
        },
    );
    context
        .unavailable
        .insert(source, "No current authored component binding".into());
    append(&mut selected, &context);
    assert_eq!(
        field(&selected, None, "Related component"),
        &InspectionValue::Unavailable("No current authored component binding".into())
    );
    let InspectionValue::Common(details) = field(&selected, None, "Component bindings") else {
        panic!()
    };
    assert!(details["bindings"].as_array().unwrap().is_empty());
}

#[test]
fn schematic_binding_inspector_preserves_physical_context_and_complete_scope() {
    let symbol = identity(Class::Symbol);
    let package = identity(Class::Footprint);
    let schematic_only = identity(Class::Symbol);
    let mut context = ComponentSelectionContext::default();
    let owner = Uuid::new_v4();
    for (from, to, name) in [(&symbol, &package, "R1"), (&package, &symbol, "R1")] {
        context.bindings.insert(
            occurrence(from),
            ComponentSelectionBinding {
                component_instance: owner,
                counterpart_status: ComponentCounterpartStatus::Resolved,
            },
        );
        context.relations.insert(ComponentSelectionRelation {
            component_instance: owner,
            source: occurrence(from),
            counterpart: occurrence(to),
            counterpart_reference: Some(name.into()),
        });
    }
    for (source, expected) in [
        (&symbol, "Footprint R1 — component binding; related only"),
        (&package, "Symbol R1 — component binding; related only"),
    ] {
        let mut selected = view(&[source.clone()]);
        selected.related.insert(if source == &symbol {
            package.clone()
        } else {
            symbol.clone()
        });
        let original = selected.related.clone();
        append(&mut selected, &context);
        assert_eq!(
            field(&selected, None, "Related component").display_preview(100),
            expected
        );
        assert_eq!(selected.related, original);
    }
    context.bindings.insert(
        occurrence(&schematic_only),
        ComponentSelectionBinding {
            component_instance: Uuid::new_v4(),
            counterpart_status: ComponentCounterpartStatus::NotDeclared,
        },
    );
    let mut selected = view(&[symbol.clone(), schematic_only, package]);
    append(&mut selected, &context);
    assert_eq!(
        field(&selected, Some(Class::Symbol), "Related component").display_preview(120),
        "2 qualified bindings; 1 related objects; 0 unavailable; related only"
    );
    assert_eq!(
        field(&selected, Some(Class::Footprint), "Related component").display_preview(120),
        "Symbol R1 — component binding; related only"
    );
    assert!(
        selected.related.is_empty(),
        "Summary never expands authored or related selection"
    );
}
