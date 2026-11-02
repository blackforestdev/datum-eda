//! One immutable source and occurrence census shared by selected-object writers.
use super::registry::NativeWriteContext;
use crate::{
    error::EngineError,
    schematic::Schematic,
    substrate::{ElectricalSelectionSnapshot, SelectionSourceDomain},
};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) struct SelectedSchematicSource {
    pub source: Schematic,
    paths: BTreeMap<(String, Uuid), Vec<Vec<Uuid>>>,
    owners: BTreeMap<(String, Uuid), Vec<Uuid>>,
}
impl SelectedSchematicSource {
    pub fn capture(context: &NativeWriteContext<'_>) -> Result<Self, EngineError> {
        let snapshot = ElectricalSelectionSnapshot::capture(context.model)
            .map_err(|e| invalid(format!("Selected source unavailable: {e:?}")))?;
        let authored = snapshot
            .authored_sources()
            .map_err(|e| invalid(format!("Selected occurrence unavailable: {e:?}")))?;
        let source = snapshot
            .schematic_source()
            .ok_or_else(|| invalid("Native schematic source unavailable"))?
            .clone();
        let mut paths: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for item in authored {
            if item.domain == SelectionSourceDomain::Schematic {
                paths
                    .entry((item.occurrence.class, item.occurrence.source_id))
                    .or_default()
                    .push(item.occurrence.instance_path);
            }
        }
        let mut owners: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for sheet in source.sheets.values() {
            macro_rules! collect {
                ($name:ident) => {
                    for (id, value) in &sheet.$name {
                        if *id == value.uuid {
                            owners
                                .entry((stringify!($name).to_string(), *id))
                                .or_default()
                                .push(sheet.uuid);
                        }
                    }
                };
            }
            collect!(symbols);
            collect!(wires);
            collect!(labels);
            collect!(ports);
            collect!(buses);
            collect!(bus_entries);
            collect!(junctions);
            collect!(noconnects);
        }
        Ok(Self {
            source,
            paths,
            owners,
        })
    }
    pub fn sheet(&self, class: &str, id: Uuid, path: &[Uuid]) -> Result<Uuid, EngineError> {
        let key = (class.to_string(), id);
        let paths = self.paths.get(&key).map(Vec::as_slice).unwrap_or_default();
        if paths.len() != 1 || paths[0] != path {
            return Err(invalid(
                "Edit requires exactly one complete matching schematic occurrence",
            ));
        }
        let owners = self.owners.get(&key).map(Vec::as_slice).unwrap_or_default();
        if owners.len() != 1 {
            return Err(invalid("Selected source ownership is ambiguous"));
        }
        Ok(owners[0])
    }
}
pub(super) fn selected_source(
    context: &NativeWriteContext<'_>,
    class: &str,
    id: Uuid,
    path: &[Uuid],
) -> Result<(Schematic, Uuid), EngineError> {
    let source = SelectedSchematicSource::capture(context)?;
    let sheet = source.sheet(class, id, path)?;
    Ok((source.source, sheet))
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
