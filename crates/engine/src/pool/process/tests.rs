use super::*;
use serde_json::json;

fn pad(mask: Option<Value>, paste: Option<Value>, diameter: i64) -> Value {
    let mut value = json!({"uuid": Uuid::new_v4(), "name": "process pad",
        "aperture": {"circle": {"diameter_nm": diameter}}});
    if let Some(mask) = mask {
        value["mask_policy"] = mask;
    }
    if let Some(paste) = paste {
        value["paste_policy"] = paste;
    }
    value
}

fn basis() -> Value {
    json!({"process_basis": {"kind": "inline", "basis": {
        "mask": {"expansion_nm": 10}, "paste": {"expansion_nm": -2},
        "paste_ratio_ppm": 100000, "declared_basis": "authored",
        "provenance": {"actor": "designer", "reason": "declared process"}
    }}})
}

#[test]
fn process_explicit_policies_keep_selected_profile_identity_revision_and_hash_binding() {
    let id = Uuid::new_v4();
    let profile = ProcessProfile {
        uuid: id,
        name: "Authored".into(),
        basis: serde_json::from_value(basis()["process_basis"]["basis"].clone()).unwrap(),
    };
    let selector = json!({"process_basis": {"kind": "project_profile", "profile_id": id,
        "expected_object_revision": 7, "expected_value_hash": "current-hash"}});
    let raw = pad(Some(json!("exposed")), Some(json!("aperture")), 101);
    let pin = |revision, hash| PinnedProcessProfile {
        profile: &profile,
        object_revision: revision,
        value_hash: hash,
    };
    let result = resolve_padstack_process(&raw, &selector, Some(pin(7, "current-hash"))).unwrap();
    assert!(result.resolved_basis.is_none());
    assert_eq!(
        result.paste,
        EffectiveProcessAperture::Circle {
            diameter_nm: Quadratic3::rational(Rational::integer(101))
        }
    );
    for selected in [Some(pin(8, "current-hash")), Some(pin(7, "old-hash")), None] {
        assert!(resolve_padstack_process(&raw, &selector, selected).is_err());
    }
    let other = ProcessProfile {
        uuid: Uuid::new_v4(),
        ..profile.clone()
    };
    assert!(
        resolve_padstack_process(
            &raw,
            &selector,
            Some(PinnedProcessProfile {
                profile: &other,
                object_revision: 7,
                value_hash: "current-hash"
            })
        )
        .is_err()
    );
    assert!(resolve_padstack_process(&raw, &json!({}), None).is_ok());
}

#[test]
fn process_exact_ratio_presence_override_and_suppression() {
    let absent = resolve_padstack_process(&pad(None, None, 101), &basis(), None).unwrap();
    let EffectiveProcessAperture::Circle { diameter_nm } = absent.paste else {
        panic!()
    };
    assert_eq!(diameter_nm.a, Rational::new(1071, 10).unwrap());
    assert!(diameter_nm.integer_nm().is_err());
    assert!(absent.raw_mask_policy.is_none());
    let explicit = resolve_padstack_process(
        &pad(Some(json!("inherit")), Some(json!("aperture")), 101),
        &basis(),
        None,
    )
    .unwrap();
    assert_eq!(explicit.raw_mask_policy, Some(json!("inherit")));
    let EffectiveProcessAperture::Circle { diameter_nm } = explicit.paste else {
        panic!()
    };
    assert_eq!(diameter_nm.integer_nm().unwrap(), 101);
    let suppressed = resolve_padstack_process(
        &pad(Some(json!("tented")), Some(json!("none")), 101),
        &json!({}),
        None,
    )
    .unwrap();
    assert_eq!(suppressed.mask, EffectiveProcessAperture::Suppressed);
    assert_eq!(suppressed.paste, EffectiveProcessAperture::Suppressed);
}

#[test]
fn process_no_metadata_fallback_collapse_or_overflow_repair() {
    assert!(
        resolve_padstack_process(
            &pad(None, None, 101),
            &json!({"process_aperture_policy": "ipc_derived"}),
            None
        )
        .is_err()
    );
    assert!(
        resolve_padstack_process(
            &pad(Some(json!({"expansion_nm": -51})), Some(json!("none")), 101),
            &json!({}),
            None
        )
        .is_err()
    );
    assert!(
        resolve_padstack_process(
            &pad(
                Some(json!({"expansion_nm": 1})),
                Some(json!("none")),
                i64::MAX
            ),
            &json!({}),
            None
        )
        .is_err()
    );
}

#[test]
fn process_faces_use_physical_sequence_and_distinct_single_copper_faces() {
    use crate::board::{Stackup, StackupLayer, StackupLayerType as T};
    let stackup = Stackup {
        layers: vec![
            StackupLayer::new(30, "front copper", T::Copper, 1),
            StackupLayer::new(2, "front mask", T::SolderMask, 1),
            StackupLayer::new(8, "back mask", T::SolderMask, 1),
        ],
    };
    let capability = ProcessSourceCapability {
        version: 1,
        front: ProcessFace {
            copper_layer: 30,
            mask_layer: Some(2),
            paste_layer: None,
        },
        back: ProcessFace {
            copper_layer: 30,
            mask_layer: Some(8),
            paste_layer: None,
        },
    };
    let process = resolve_padstack_process(
        &pad(Some(json!("exposed")), Some(json!("none")), 101),
        &json!({}),
        None,
    )
    .unwrap();
    let membership = capability.membership(&stackup, &[30], &process).unwrap();
    assert_eq!(membership.mask_layers, vec![2, 8]);
    assert!(membership.paste_layers.is_empty());
    let mut ambiguous = capability;
    ambiguous.back.mask_layer = Some(2);
    assert!(ambiguous.validate(&stackup).is_err());
}
