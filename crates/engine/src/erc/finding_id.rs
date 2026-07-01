use uuid::Uuid;

use super::ErcObjectRef;

pub(super) fn stable_finding_id(
    code: &str,
    net_name: Option<&str>,
    component: Option<&str>,
    pin: Option<&str>,
    objects: &[ErcObjectRef],
) -> Uuid {
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
