//! Local discovery updates preserve active lock bytes and retained history.
use super::*;

pub(super) fn candidate_state(
    snapshot: &LibraryAssetSnapshot,
    previous: &LibraryStateValues,
    provenance: &WriteProvenance,
    draft: &capture::Draft,
) -> Result<(LibraryStateValues, Vec<LibraryContent>), EngineError> {
    candidate_state_from_manifest(snapshot.manifest(), previous, provenance, draft)
}

pub(super) fn candidate_state_from_manifest(
    previous_manifest: &LibraryReleaseManifest,
    previous: &LibraryStateValues,
    provenance: &WriteProvenance,
    draft: &capture::Draft,
) -> Result<(LibraryStateValues, Vec<LibraryContent>), EngineError> {
    let mut manifest = previous_manifest.clone();
    manifest.release_id = Uuid::new_v4();
    manifest.publisher = actor(provenance);
    // Subject-bound claims belong to the retained prior release, not this new
    // local candidate. Old immutable manifests and evidence remain untouched.
    manifest.publication_record_refs.clear();
    manifest.roots = draft.roots.clone();
    manifest.closure_digest =
        AssetDigest::record(RecordDomain::Closure, &encode_metadata(&draft.references)?);
    let mut content = draft.content.clone();
    for item in &content {
        let digest = item.fingerprint()?;
        match item {
            LibraryContent::Blob { bytes, .. } => {
                if let Some(old) = manifest.blobs.iter().find(|b| b.digest == digest) {
                    if old.byte_length != bytes.len() as u64 {
                        return Err(refusal("retained blob size collision"));
                    }
                } else {
                    manifest.blobs.push(BlobEntry {
                        digest,
                        byte_length: bytes.len() as u64,
                        media_type: "application/json".into(),
                    });
                }
            }
            LibraryContent::Record { record_type, .. } => {
                let kind = match record_type {
                    LibraryRecordType::AssetRevision => ObjectKind::AssetRevision,
                    LibraryRecordType::Provenance => ObjectKind::Provenance,
                    LibraryRecordType::Review => ObjectKind::Review,
                    LibraryRecordType::ReleaseManifest => {
                        return Err(refusal("candidate draft contains a nested release"));
                    }
                };
                if let Some(old) = manifest.objects.iter().find(|o| o.digest == digest) {
                    if old.kind != kind {
                        return Err(refusal("retained record type collision"));
                    }
                } else {
                    manifest.objects.push(ObjectEntry { digest, kind });
                }
            }
        }
    }
    manifest.objects.sort_by(|a, b| a.digest.cmp(&b.digest));
    manifest.blobs.sort_by(|a, b| a.digest.cmp(&b.digest));
    let digest = manifest.fingerprint()?;
    content.push(LibraryContent::Record {
        record_type: LibraryRecordType::ReleaseManifest,
        fingerprint: digest.as_str().into(),
        bytes: encode_metadata(&manifest)?,
    });
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: manifest.collection_id,
        profile: PROFILE.into(),
        manifest_ref: digest,
    };
    Ok((
        LibraryStateValues {
            lock: previous.lock.clone(),
            catalog: encode_metadata(&discovery)?,
        },
        content,
    ))
}
