use std::collections::BTreeMap;

use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{
    ComponentInstanceAuthority, ComponentInstanceRoleMetadata, DesignModel,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct NativeBomRow {
    pub(in super::super) placement_uuid: Option<Uuid>,
    pub(in super::super) component_instance_uuid: Option<Uuid>,
    pub(in super::super) component_instance_role: Option<String>,
    pub(in super::super) component_instance_label: Option<String>,
    pub(in super::super) reference: String,
    pub(in super::super) value: String,
    pub(in super::super) part_uuid: String,
    pub(in super::super) package_uuid: String,
    pub(in super::super) layer: i32,
    pub(in super::super) x_nm: i64,
    pub(in super::super) y_nm: i64,
    pub(in super::super) rotation_deg: i32,
    pub(in super::super) locked: bool,
}

impl NativeBomRow {
    fn identity_key(&self, placement_identity: bool) -> String {
        if placement_identity {
            return self
                .placement_uuid
                .expect("placement CSV requires occurrence identity")
                .to_string();
        }
        self.component_instance_uuid
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.package_uuid.clone())
    }

    pub(super) fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.component_instance_uuid != other.component_instance_uuid {
            fields.push("component_instance_uuid".to_string());
        }
        if self.reference != other.reference {
            fields.push("reference".to_string());
        }
        if self.component_instance_role != other.component_instance_role {
            fields.push("component_instance_role".to_string());
        }
        if self.component_instance_label != other.component_instance_label {
            fields.push("component_instance_label".to_string());
        }
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct NativePnpRow {
    pub(in super::super) placement_uuid: Option<Uuid>,
    pub(in super::super) component_instance_uuid: Option<Uuid>,
    pub(in super::super) component_instance_role: Option<String>,
    pub(in super::super) component_instance_label: Option<String>,
    pub(in super::super) reference: String,
    pub(in super::super) x_nm: i64,
    pub(in super::super) y_nm: i64,
    pub(in super::super) rotation_deg: i32,
    pub(in super::super) layer: i32,
    pub(in super::super) side: String,
    pub(in super::super) package_uuid: String,
    pub(in super::super) part_uuid: String,
    pub(in super::super) value: String,
    pub(in super::super) locked: bool,
}

impl NativePnpRow {
    fn identity_key(&self, placement_identity: bool) -> String {
        if placement_identity {
            return self
                .placement_uuid
                .expect("placement CSV requires occurrence identity")
                .to_string();
        }
        self.component_instance_uuid
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.package_uuid.clone())
    }

    pub(super) fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.component_instance_uuid != other.component_instance_uuid {
            fields.push("component_instance_uuid".to_string());
        }
        if self.reference != other.reference {
            fields.push("reference".to_string());
        }
        if self.component_instance_role != other.component_instance_role {
            fields.push("component_instance_role".to_string());
        }
        if self.component_instance_label != other.component_instance_label {
            fields.push("component_instance_label".to_string());
        }
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.side != other.side {
            fields.push("side".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

pub(super) fn bom_rows_by_identity(
    rows: &[NativeBomRow],
    placement_identity: bool,
) -> (BTreeMap<String, NativeBomRow>, Vec<String>) {
    let mut by_identity = BTreeMap::new();
    let mut duplicates = Vec::new();
    for row in rows {
        let identity = row.identity_key(placement_identity);
        match by_identity.entry(identity) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                duplicates.push(existing.key().clone());
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(row.clone());
            }
        }
    }
    (by_identity, duplicates)
}

pub(super) fn pnp_rows_by_identity(
    rows: &[NativePnpRow],
    placement_identity: bool,
) -> (BTreeMap<String, NativePnpRow>, Vec<String>) {
    let mut by_identity = BTreeMap::new();
    let mut duplicates = Vec::new();
    for row in rows {
        let identity = row.identity_key(placement_identity);
        match by_identity.entry(identity) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                duplicates.push(existing.key().clone());
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(row.clone());
            }
        }
    }
    (by_identity, duplicates)
}

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
        placement_uuid: Some(component.uuid),
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
        placement_uuid: Some(component.uuid),
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
