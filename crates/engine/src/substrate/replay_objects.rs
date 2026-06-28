use std::collections::BTreeMap;
use std::path::Path;

use super::{
    DomainObject, EngineError, ObjectId, SourceShardRef, collect_uuid_objects,
    domain_for_shard_kind, read_json_value,
};

pub(super) fn refresh_materialized_shard_objects(
    project_root: &Path,
    shards: &[SourceShardRef],
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) -> Result<(), EngineError> {
    let mut import_map = BTreeMap::new();
    for shard in shards {
        let path = if shard.path.is_absolute() {
            shard.path.clone()
        } else {
            project_root.join(&shard.relative_path)
        };
        if !path.exists() {
            continue;
        }
        let Ok(value) = read_json_value(&path) else {
            continue;
        };
        collect_uuid_objects(
            &value,
            shard,
            domain_for_shard_kind(&shard.kind),
            objects,
            &mut import_map,
        );
    }
    Ok(())
}
