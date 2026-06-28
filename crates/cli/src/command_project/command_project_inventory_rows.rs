use std::collections::BTreeMap;

use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{
    ComponentInstanceAuthority, ComponentInstanceRoleMetadata, DesignModel,
};
use uuid::Uuid;

use super::{NativeBomRow, NativePnpRow};

pub(super) fn component_instances_by_package(
    model: &DesignModel,
) -> BTreeMap<Uuid, (Uuid, Option<ComponentInstanceRoleMetadata>)> {
    let mut by_package = BTreeMap::new();
    for (component_instance_id, component_instance) in &model.component_instances {
        if component_instance.authority != ComponentInstanceAuthority::Authored {
            continue;
        }
        for package_ref in &component_instance.placed_package_refs {
            by_package.insert(
                *package_ref,
                (
                    *component_instance_id,
                    component_instance
                        .placed_package_roles
                        .get(package_ref)
                        .cloned(),
                ),
            );
        }
    }
    by_package
}

pub(super) fn component_to_bom_row(
    component: PlacedPackage,
    component_instance_uuid: Option<Uuid>,
    role: Option<&ComponentInstanceRoleMetadata>,
) -> NativeBomRow {
    NativeBomRow {
        component_instance_uuid,
        component_instance_role: role.map(|metadata| metadata.role.clone()),
        component_instance_label: role.and_then(|metadata| metadata.label.clone()),
        reference: component.reference,
        value: component.value,
        part_uuid: component.part.to_string(),
        package_uuid: component.package.to_string(),
        layer: component.layer,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        locked: component.locked,
    }
}

pub(super) fn component_to_pnp_row(
    component: PlacedPackage,
    component_instance_uuid: Option<Uuid>,
    role: Option<&ComponentInstanceRoleMetadata>,
) -> NativePnpRow {
    NativePnpRow {
        component_instance_uuid,
        component_instance_role: role.map(|metadata| metadata.role.clone()),
        component_instance_label: role.and_then(|metadata| metadata.label.clone()),
        reference: component.reference,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        layer: component.layer,
        side: if component.layer <= 16 {
            "top"
        } else {
            "bottom"
        }
        .to_string(),
        package_uuid: component.package.to_string(),
        part_uuid: component.part.to_string(),
        value: component.value,
        locked: component.locked,
    }
}
