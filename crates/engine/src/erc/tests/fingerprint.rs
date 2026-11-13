use crate::erc::{ErcConfig, run_prechecks_with_nets};
use crate::schematic::{
    CheckDomain, CheckWaiver, NetPinRef, PinElectricalType, Schematic, SchematicNetInfo,
    WaiverTarget,
};
use uuid::Uuid;

#[test]
fn output_conflict_rename_keeps_identity_but_electrical_change_reopens() {
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: Default::default(),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: vec![],
    };
    let mut net = SchematicNetInfo {
        uuid: Uuid::new_v4(),
        occurrences: Vec::new(),
        name: "DATA".into(),
        class: None,
        pins: (0..2)
            .map(|i| NetPinRef {
                uuid: Uuid::new_v4(),
                instance_path: Vec::new(),
                component: format!("U{i}"),
                pin: "1".into(),
                electrical_type: PinElectricalType::Output,
            })
            .collect(),
        labels: 1,
        ports: 0,
        port_uuids: vec![],
        sheets: vec![],
        semantic_class: None,
    };
    let evaluate = |net, waivers: &[CheckWaiver]| {
        run_prechecks_with_nets(&schematic, &ErcConfig::default(), waivers, vec![net])
    };
    let original = evaluate(net.clone(), &[]).remove(0);
    let waiver = CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::ERC,
        target: WaiverTarget::Fingerprint(original.fingerprint.clone().unwrap()),
        rationale: "Reviewed conflict".into(),
        created_by: None,
    };
    net.name = "RENAMED".into();
    net.pins[0].component = "U7".into();
    // Caller ordering is presentation, too.
    net.pins.reverse();
    let renamed = evaluate(net.clone(), std::slice::from_ref(&waiver)).remove(0);
    assert_ne!(renamed.message, original.message);
    assert_eq!(renamed.id, original.id);
    assert_eq!(renamed.fingerprint, original.fingerprint);
    assert!(renamed.waived);
    net.pins[0].electrical_type = PinElectricalType::PowerOut;
    let changed = evaluate(net, &[waiver]).remove(0);
    assert_eq!(changed.code, original.code);
    assert_eq!(changed.id, original.id);
    assert_ne!(changed.fingerprint, original.fingerprint);
    assert!(!changed.waived);
}
