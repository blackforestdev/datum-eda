//! Captured map pins share the accepted and sealed pending source owners.
use super::*;
impl ElectricalBasis<'_> {
    /// Read the declared map from this captured frame through the shared
    /// authenticated history reader. Held snapshots never reread live files.
    pub(in crate::substrate) fn pinned_map(
        &self,
        model: &DesignModel,
        reference: &crate::substrate::RevisionedRef,
    ) -> Result<Value, EngineError> {
        if let Some(frame) = &self.pending_frame {
            if frame.candidate().project.project_id != model.project.project_id
                || frame.candidate().model_revision != model.model_revision
            {
                return Err(invalid("pending map frame differs from evaluated model"));
            }
            return frame.pinned_definition(
                reference.object_id,
                "pin_pad_maps",
                reference.object_revision,
            );
        }
        if self.pending_map_changes.contains(&reference.object_id) {
            return Err(invalid(
                "declared Map revision is unavailable in a pending source-change frame",
            ));
        }
        let basis = if let Some(accepted) = self.accepted_library_basis {
            if !self
                .part_statuses
                .as_ref()
                .is_some_and(|(project, revision, _)| {
                    *project == model.project.project_id && *revision == model.model_revision
                })
                || model.objects.get(&reference.object_id)
                    != accepted.objects.get(&reference.object_id)
            {
                return Err(invalid(
                    "pinned Map differs from the qualified prospective source basis",
                ));
            }
            accepted
        } else {
            model
        };
        basis.pool_object_revision_with_reader(
            reference.object_id,
            "pin_pad_maps",
            reference.object_revision,
            &|shard| {
                if !model.source_shards.iter().any(|view| view == shard) {
                    return Err(invalid(
                        "pinned Map source shard differs from the captured basis",
                    ));
                }
                self.values
                    .get(&shard.shard_id)
                    .cloned()
                    .ok_or_else(|| invalid("pinned Map source is absent from the captured basis"))
            },
        )
    }
}
