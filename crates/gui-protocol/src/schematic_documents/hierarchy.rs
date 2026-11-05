//! Source-backed sheet bodies and interface memberships over qualified documents.
//! A membership path names the child occurrence; its prefix names the parent
//! source occurrence containing the actual Port. Display keys never mint IDs.
use super::{SchematicDocumentKey, SchematicDocuments};
use crate::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, PointNm};
use eda_engine::{schematic::Schematic, substrate::ModelRevision};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyTarget {
    pub identity: Id,
    pub parent: SchematicDocumentKey,
    pub child: SchematicDocumentKey,
    pub instance: Uuid,
    pub definition: Uuid,
    pub instance_name: String,
    pub source_name: String,
    pub position: PointNm,
    pub port: Option<Uuid>,
    pub port_name: Option<String>,
    pub port_direction: Option<Value>,
    pub child_label: Option<Uuid>,
    pub unavailable: Option<String>,
}
impl HierarchyTarget {
    pub fn graphic_key(&self) -> String {
        match self.port {
            Some(port) => interface_key(self.instance, port),
            None => format!("schematic-sheet-instance:{}", self.instance),
        }
    }
    pub fn text_key(&self) -> String {
        format!("{}:text", self.graphic_key())
    }

    pub fn display_name(&self) -> String {
        match &self.port_name {
            Some(name) => format!("{name} · {}", self.instance_name),
            None => self.instance_name.clone(),
        }
    }
    pub fn values(&self) -> BTreeMap<String, Value> {
        let mut values = BTreeMap::from([
            ("name".into(), json!(self.display_name())),
            ("sheet_instance_name".into(), json!(self.instance_name)),
            ("sheet_source_name".into(), json!(self.source_name)),
            ("sheet_instance_uuid".into(), json!(self.instance)),
            ("sheet_definition_uuid".into(), json!(self.definition)),
            ("sheet_source_uuid".into(), json!(self.child.source_sheet)),
            ("parent_sheet_uuid".into(), json!(self.parent.source_sheet)),
            (
                "sheet_occurrence_path".into(),
                json!(self.child.instance_path),
            ),
            (
                "parent_occurrence_path".into(),
                json!(self.parent.instance_path),
            ),
            (
                "position_nm".into(),
                json!({"x":self.position.x,"y":self.position.y}),
            ),
            (
                "hierarchy_status".into(),
                json!(self.unavailable.as_deref().unwrap_or("Available")),
            ),
        ]);
        if let Some(port) = self.port {
            values.insert("parent_port_uuid".into(), json!(port));
            values.insert("port_name".into(), json!(self.port_name));
            values.insert(
                "port_direction".into(),
                self.port_direction.clone().unwrap_or(Value::Null),
            );
            if let Some(label) = self.child_label {
                values.insert("child_label_uuid".into(), json!(label));
            }
        }
        values
    }
}

pub fn interface_key(instance: Uuid, port: Uuid) -> String {
    format!("schematic-sheet-interface:{instance}:{port}")
}

/// Uses the existing document occurrence traversal and engine hierarchy links.
/// No name matching, fabricated source object or alternate hierarchy resolver.
pub fn capture(
    project: Uuid,
    revision: ModelRevision,
    source: &Schematic,
) -> Result<BTreeMap<Id, HierarchyTarget>, String> {
    let documents =
        SchematicDocuments::from_source(project, revision, String::new(), source.clone())
            .map_err(|error| error.to_string())?;
    let mut targets = BTreeMap::new();
    for row in &documents.rows {
        let Some(parent) = &row.parent else {
            continue;
        };
        let Some(instance_id) = row.key.instance_path.last() else {
            continue;
        };
        let instance = source
            .sheet_instances
            .get(instance_id)
            .ok_or("Qualified sheet instance is unavailable")?;
        let parent_sheet = source
            .sheets
            .get(&parent.source_sheet)
            .ok_or("Parent source sheet is unavailable")?;
        if instance.parent_sheet != Some(parent.source_sheet) {
            return Err("Sheet instance parent occurrence changed".into());
        }
        let body = HierarchyTarget {
            identity: Id {
                class: Class::SheetInstance,
                id: *instance_id,
                instance_path: row.key.instance_path.clone(),
            },
            parent: parent.clone(),
            child: row.key.clone(),
            instance: *instance_id,
            definition: instance.definition,
            instance_name: instance.name.clone(),
            source_name: row.source_name.clone(),
            position: PointNm {
                x: instance.position.x,
                y: instance.position.y,
            },
            port: None,
            port_name: None,
            port_direction: None,
            child_label: None,
            unavailable: None,
        };
        let bindings = eda_engine::connectivity::occurrence_partition::sheet_interface_bindings(
            source,
            *instance_id,
        );
        let mut body_status = bindings.as_ref().err().map(|error| {
            format!("Interface unavailable: {error}; repair this instance's source bindings")
        });
        let mut seen = BTreeSet::new();
        for port_id in &instance.ports {
            if !seen.insert(*port_id) {
                body_status =
                    Some("Repeated interface membership; repair the source binding".into());
                continue;
            }
            let Some(port) = parent_sheet.ports.get(port_id) else {
                body_status = Some(format!(
                    "Parent port {port_id} is unavailable; repair the source binding"
                ));
                continue;
            };
            let child_label = bindings
                .as_ref()
                .ok()
                .and_then(|bindings| bindings.get(port_id))
                .copied();
            let target = HierarchyTarget {
                identity: Id {
                    class: Class::SheetInterface,
                    id: *port_id,
                    instance_path: row.key.instance_path.clone(),
                },
                position: PointNm {
                    x: port.position.x,
                    y: port.position.y,
                },
                port: Some(*port_id),
                port_name: Some(port.name.clone()),
                port_direction: Some(json!(port.direction)),
                child_label,
                unavailable: body_status.clone(),
                ..body.clone()
            };
            if target.unavailable.is_some() {
                body_status = target.unavailable.clone();
            }
            targets.insert(target.identity.clone(), target);
        }
        targets.insert(
            body.identity.clone(),
            HierarchyTarget {
                unavailable: body_status,
                ..body
            },
        );
    }
    Ok(targets)
}
