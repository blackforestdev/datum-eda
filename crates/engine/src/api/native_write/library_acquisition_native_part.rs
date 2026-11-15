//! Identity-preserving acquisition of one complete, immutable native Part snapshot.
use super::{LibraryAcquisitionRequest, PreparedWrite, WriteProvenance};
use crate::{
    api::native_write::{
        context::BatchComposer,
        imports::{import_map_shard_operation, source_shard_id_for_relative_path},
        library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
        registry::NativeWriteContext,
    },
    error::EngineError,
    substrate::{DesignModel, ImportMapEntry, ObjectRevision, Operation, ProjectResolver},
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use uuid::Uuid;
#[path = "library_acquisition_native_part/source.rs"]
mod source;

#[derive(Debug, Clone, PartialEq)]
pub struct NativePartNode {
    pub id: Uuid,
    pub kind: String,
    pub object: Value,
    pub revision: u64,
    pub source_hash: String,
    pub source_path: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativePartPreview {
    pub source_path: PathBuf,
    pub source_project: Uuid,
    pub part: Uuid,
    pub name: String,
    pub digest: String,
    pub nodes: Vec<NativePartNode>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NativePartChoice {
    pub id: Uuid,
    pub label: String,
    /// Declared descriptive metadata; never physical qualification or identity.
    pub search_text: String,
    pub refusal: Option<String>,
}
fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}
fn resolve(path: &Path) -> Result<DesignModel, EngineError> {
    if path.as_os_str().is_empty() {
        return Err(invalid(
            "Enter a saved native source project folder before refreshing",
        ));
    }
    if !path.join("project.json").is_file() {
        return Err(invalid(
            "Choose a saved native Datum project containing the reusable Part",
        ));
    }
    ProjectResolver::new(path).resolve()
}
/// Source reads never write the source project or attach it as a live pool.
pub fn catalog(path: &Path) -> Result<Vec<NativePartChoice>, EngineError> {
    let model = resolve(path)?;
    let index = source::Index::new(&model);
    let mut rows = Vec::new();
    for object in model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "parts")
    {
        let result = index.node(object.object_id, "parts").and_then(|raw| {
            serde_json::from_value::<crate::pool::Part>(raw.object).map_err(EngineError::from)
        });
        let part = match result {
            Ok(part) => part,
            Err(error) => {
                rows.push(NativePartChoice {
                    id: object.object_id,
                    label: "Native Part source unavailable".into(),
                    search_text: String::new(),
                    refusal: Some(error.to_string()),
                });
                continue;
            }
        };
        let package_name = index
            .node(part.package, "packages")
            .ok()
            .and_then(|node| {
                node.object
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let mut tags: Vec<_> = part.tags.iter().cloned().collect();
        tags.sort();
        let search_text = std::iter::once(part.description.as_str())
            .chain(std::iter::once(package_name.as_str()))
            .chain(tags.iter().map(String::as_str))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        rows.push(NativePartChoice {
            id: part.uuid,
            label: format!(
                "{} {} · {} · {}",
                part.manufacturer, part.mpn, part.value, package_name
            ),
            search_text,
            refusal: source::capture(path, &model, &index, part.uuid)
                .err()
                .map(|e| e.to_string()),
        });
    }
    rows.sort_by(|a, b| a.label.cmp(&b.label).then(a.id.cmp(&b.id)));
    Ok(rows)
}
/// An explicit content-qualified snapshot, not a distributed latest-source lock.
pub fn preview(path: &Path, part: Uuid) -> Result<NativePartPreview, EngineError> {
    let model = resolve(path)?;
    let snapshot = source::capture(path, &model, &source::Index::new(&model), part)?;
    let checked_model = resolve(path)?;
    let checked = source::capture(
        path,
        &checked_model,
        &source::Index::new(&checked_model),
        part,
    )?;
    if snapshot.digest != checked.digest {
        return Err(invalid(
            "Native Part source changed during capture; refresh it",
        ));
    }
    Ok(snapshot)
}
impl NativePartPreview {
    pub fn object(&self, kind: &str, id: Uuid) -> Option<&Value> {
        self.nodes
            .iter()
            .find(|n| n.kind == kind && n.id == id)
            .map(|n| &n.object)
    }
    /// Same native identity and every raw field must agree; no name-based merging.
    pub fn reused(&self, target: &DesignModel) -> Result<Vec<Uuid>, EngineError> {
        let index = source::Index::new(target);
        let mut reused = Vec::new();
        for node in &self.nodes {
            if target.objects.contains_key(&node.id) {
                let saved = index.node(node.id, &node.kind)?;
                if saved.object != node.object {
                    return Err(invalid(format!(
                        "Native {} {} conflicts with this Project's saved definition",
                        node.kind, node.id
                    )));
                }
                reused.push(node.id);
            } else if index.has_identity(node.id)
                || target.source_shards.iter().any(|s| {
                    s.relative_path
                        == PoolLibraryObjectTarget::new("pool", &node.kind, node.id).relative_path
                })
            {
                return Err(invalid(format!(
                    "Native source identity {} is unavailable or ambiguous",
                    node.id
                )));
            }
        }
        Ok(reused)
    }
}
pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryAcquisitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let id = request
        .symbol_name
        .parse()
        .map_err(|_| invalid("Choose a native Part identity"))?;
    let snapshot = preview(&request.source_path, id)?;
    if snapshot.digest != request.expected_source_hash {
        return Err(invalid(
            "Native Part source changed since preview; refresh it",
        ));
    }
    let reused = snapshot.reused(context.model)?;
    if reused.len() == snapshot.nodes.len() {
        return Err(invalid(
            "This complete Part is already in the Project; select its saved definition",
        ));
    }
    let map_path = format!(
        ".datum/import_map/native-part-{}-{}.json",
        snapshot.source_project, id
    );
    if context
        .model
        .source_shards
        .iter()
        .any(|s| s.relative_path == map_path)
    {
        return Err(invalid(
            "This native Part acquisition trace already exists; saved definitions are not overwritten",
        ));
    }
    let specs = snapshot
        .nodes
        .iter()
        .filter(|n| !reused.contains(&n.id))
        .map(|n| PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", &n.kind, n.id),
            object: n.object.clone(),
        })
        .collect();
    let prepared =
        build_pool_library_write(context.model, provenance.clone(), Some("pool"), specs)?;
    let entries = snapshot
        .nodes
        .iter()
        .map(|n| {
            let target = PoolLibraryObjectTarget::new("pool", &n.kind, n.id);
            ImportMapEntry {
                import_key: format!(
                    "datum:native-part:{}:{}:{}",
                    snapshot.source_project, id, n.id
                ),
                object_id: n.id,
                source_shard_id: context.model.objects.get(&n.id).map_or_else(
                    || source_shard_id_for_relative_path(&target.relative_path),
                    |o| o.source_shard_id,
                ),
                status: Default::default(),
                source_tool: "datum_native_part".into(),
                source_path: snapshot
                    .source_path
                    .join(&n.source_path)
                    .to_string_lossy()
                    .into_owned(),
                source_object_ref: format!(
                    "{}:{}:{}:revision:{}:snapshot:{}",
                    snapshot.source_project, id, n.id, n.revision, snapshot.digest
                ),
                source_hash: n.source_hash.clone(),
            }
        })
        .collect::<Vec<_>>();
    if entries
        .iter()
        .any(|entry| context.model.import_map.contains_key(&entry.import_key))
    {
        return Err(invalid(
            "Native acquisition trace identity already exists; refresh or use the saved Part",
        ));
    }
    let guards = reused.into_iter().map(|id| Operation::GuardObjectRevision {
        object_id: id,
        expected_object_revision: ObjectRevision(context.model.objects[&id].object_revision.0),
    });
    BatchComposer::compose(context.model, provenance)
        .push_ops(guards)
        .push_ops(prepared.batch.operations)
        .push_ops([import_map_shard_operation(map_path, entries)?])
        .finish()
}

#[cfg(test)]
#[path = "library_acquisition_native_part/tests.rs"]
mod tests;
