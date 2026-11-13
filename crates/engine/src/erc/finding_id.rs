use uuid::Uuid;

use super::ErcObjectRef;

pub(super) fn stable_finding_id(
    code: &str,
    net_name: Option<&str>,
    component: Option<&str>,
    pin: Option<&str>,
    objects: &[ErcObjectRef],
    object_uuids: &[Uuid],
) -> Uuid {
    // Interface diagnostics retain their existing occurrence-sensitive identity.
    if code != "hierarchical_connectivity_mismatch" && !object_uuids.is_empty() {
        let mut ids = object_uuids.to_vec();
        ids.sort();
        ids.dedup();
        return Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("erc:{code}:{ids:?}").as_bytes(),
        );
    }
    let mut material = vec![format!("code={code}")];
    if let Some(net_name) = net_name {
        material.push(format!("net={net_name}"));
    }
    if let Some(component) = component {
        material.push(format!("component={component}"));
    }
    if let Some(pin) = pin {
        material.push(format!("pin={pin}"));
    }
    for object in objects {
        material.push(format!("obj:{}={}", object.kind, object.key));
    }
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, material.join("|").as_bytes())
}

/// Qualify ordinary findings from the native partition actually evaluated.
/// Legacy and interface diagnostics keep their existing identity behavior.
pub(super) fn qualify(
    findings: &mut [super::ErcFinding],
    net: &crate::schematic::SchematicNetInfo,
) {
    if net.occurrences.is_empty() {
        return;
    }
    for finding in findings {
        finding.net_uuid = Some(net.uuid);
        finding.occurrences = net
            .occurrences
            .iter()
            .filter(|occurrence| {
                finding.object_uuids.contains(&occurrence.source_id)
                    || (finding.object_uuids == [net.uuid]
                        && matches!(occurrence.class.as_str(), "labels" | "net_declarations"))
            })
            .cloned()
            .collect();
        finding.occurrences.sort();
        finding.occurrences.dedup();
        let material = serde_json::to_vec(&(finding.code, net.uuid, &finding.occurrences))
            .expect("qualified ERC identity");
        finding.id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, &material);
    }
}
