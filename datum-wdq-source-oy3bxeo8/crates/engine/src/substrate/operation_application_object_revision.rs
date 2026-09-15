use std::collections::BTreeMap;

use super::{CommitDiff, DomainObject, EngineError, ObjectId, ObjectRevision};

pub(super) fn bump_existing_object(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    object_id: ObjectId,
    diff: Option<&mut CommitDiff>,
) -> Result<(), EngineError> {
    let object = objects.get_mut(&object_id).ok_or(EngineError::NotFound {
        object_type: "domain_object",
        uuid: object_id,
    })?;
    object.object_revision = ObjectRevision(object.object_revision.0 + 1);
    if let Some(diff) = diff {
        diff.modified.push(object_id);
    }
    Ok(())
}
