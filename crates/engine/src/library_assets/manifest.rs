//! Release transport inventory, kept separate from publication/approval authority.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    Actor, AssetClosure, AssetDigest, AssetProvenance, AssetRevision, AssetRevisionRef, Evidence,
    RecordDomain, decode_metadata, encode_metadata, invalid, records::PROFILE, review::AssetReview,
    validation,
};
use crate::error::EngineError;

pub(crate) const DISCOVERY_PATH: &str = "library.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LibraryDiscovery {
    pub(crate) schema_version: u32,
    pub(crate) collection_id: Uuid,
    pub(crate) profile: String,
    pub(crate) manifest_ref: AssetDigest,
}

impl LibraryDiscovery {
    pub(crate) fn verify_manifest(
        &self,
        manifest: &LibraryReleaseManifest,
    ) -> Result<(), EngineError> {
        if self.schema_version != 1
            || self.profile != PROFILE
            || self.collection_id != manifest.collection_id
            || self.manifest_ref != manifest.fingerprint()?
        {
            return Err(invalid("discovery pointer and release manifest differ"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ObjectKind {
    AssetRevision,
    Provenance,
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObjectEntry {
    pub(crate) digest: AssetDigest,
    #[serde(rename = "type")]
    pub(crate) kind: ObjectKind,
}

impl ObjectEntry {
    pub(crate) fn relative_path(&self) -> String {
        format!("objects/sha256/{}.json", self.digest.hex())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlobEntry {
    pub(crate) digest: AssetDigest,
    pub(crate) byte_length: u64,
    pub(crate) media_type: String,
}

impl BlobEntry {
    pub(crate) fn relative_path(&self) -> String {
        format!("blobs/sha256/{}", self.digest.hex())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LibraryReleaseManifest {
    pub(crate) schema_version: u32,
    pub(crate) collection_id: Uuid,
    pub(crate) release_id: Uuid,
    pub(crate) publisher: Actor,
    pub(crate) profile: String,
    pub(crate) roots: Vec<AssetRevisionRef>,
    pub(crate) closure_digest: AssetDigest,
    pub(crate) objects: Vec<ObjectEntry>,
    pub(crate) blobs: Vec<BlobEntry>,
    pub(crate) publication_record_refs: Vec<AssetDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extensions: Option<BTreeMap<String, Value>>,
}

impl LibraryReleaseManifest {
    pub(crate) fn validate(&self) -> Result<(), EngineError> {
        if self.schema_version != 1 || self.profile != PROFILE {
            return Err(invalid("unsupported release manifest profile/schema"));
        }
        validation::extensions(self.extensions.as_ref())?;
        if self.roots.windows(2).any(|p| p[0] >= p[1])
            || self.objects.windows(2).any(|p| p[0].digest >= p[1].digest)
            || self.blobs.windows(2).any(|p| p[0].digest >= p[1].digest)
            || self
                .publication_record_refs
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != self.publication_record_refs.len()
        {
            return Err(invalid("manifest inventories must be unique and sorted"));
        }
        for blob in &self.blobs {
            if blob.media_type.is_empty() {
                return Err(invalid("blob media type is required"));
            }
        }
        Ok(())
    }

    pub(crate) fn fingerprint(&self) -> Result<AssetDigest, EngineError> {
        self.validate()?;
        Ok(AssetDigest::record(
            RecordDomain::ReleaseManifest,
            &encode_metadata(self)?,
        ))
    }

    /// Check the declared transport inventory; this does not approve publication
    /// or replace schema-derived closure/context/design checks during adoption.
    pub(crate) fn verify_inventory(
        &self,
        objects: &BTreeMap<AssetDigest, Vec<u8>>,
        blobs: &BTreeMap<AssetDigest, Vec<u8>>,
        metadata_byte_limit: usize,
    ) -> Result<(), EngineError> {
        let own_digest = self.fingerprint()?;
        let mut revisions = BTreeMap::new();
        let mut retained = BTreeSet::new();
        collect_evidence(&self.publisher.verification_evidence, &mut retained);
        for entry in &self.objects {
            if entry.digest == own_digest {
                return Err(invalid("release manifest may not inventory itself"));
            }
            let bytes = objects
                .get(&entry.digest)
                .ok_or_else(|| invalid("inventoried object missing"))?;
            let actual = match entry.kind {
                ObjectKind::AssetRevision => {
                    let revision: AssetRevision = decode_metadata(bytes, metadata_byte_limit)?;
                    let reference = revision.reference()?;
                    revisions.insert(reference, revision);
                    AssetDigest::record(RecordDomain::AssetRevision, bytes)
                }
                ObjectKind::Provenance => {
                    let record: AssetProvenance = decode_metadata(bytes, metadata_byte_limit)?;
                    for actor in &record.authors {
                        collect_evidence(&actor.verification_evidence, &mut retained);
                    }
                    if let Some(actor) = &record.change_actor {
                        collect_evidence(&actor.verification_evidence, &mut retained);
                    }
                    for evidence in [&record.sources, &record.rights.evidence, &record.evidence] {
                        collect_evidence(evidence, &mut retained);
                    }
                    record.fingerprint()?
                }
                ObjectKind::Review => {
                    let record: AssetReview = decode_metadata(bytes, metadata_byte_limit)?;
                    collect_evidence(&record.reviewer.verification_evidence, &mut retained);
                    collect_evidence(&record.evidence, &mut retained);
                    record.fingerprint()?
                }
            };
            if actual != entry.digest {
                return Err(invalid("inventory object type/fingerprint mismatch"));
            }
        }
        for entry in &self.blobs {
            let bytes = blobs
                .get(&entry.digest)
                .ok_or_else(|| invalid("inventoried blob missing"))?;
            if u64::try_from(bytes.len()).ok() != Some(entry.byte_length) {
                return Err(invalid("inventoried blob byte length differs"));
            }
            entry.digest.verify_blob(bytes)?;
        }
        let listed: BTreeMap<_, _> = self
            .objects
            .iter()
            .map(|o| (o.digest.clone(), o.kind))
            .collect();
        let listed_blobs: BTreeSet<_> = self.blobs.iter().map(|b| b.digest.clone()).collect();
        if !retained.is_subset(&listed_blobs) {
            return Err(invalid(
                "retained evidence blob absent from release inventory",
            ));
        }
        for root in &self.roots {
            if !revisions.contains_key(root) {
                return Err(invalid("manifest root revision unavailable"));
            }
        }
        for revision in revisions.values() {
            if listed.get(&revision.provenance_ref) != Some(&ObjectKind::Provenance)
                || !listed_blobs.contains(&revision.payload_ref)
                || revision
                    .dependencies
                    .iter()
                    .any(|d| !revisions.contains_key(&d.target))
            {
                return Err(invalid(
                    "revision dependency/provenance/payload absent from inventory",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn verify_closure(&self, closure: &AssetClosure<'_>) -> Result<(), EngineError> {
        self.validate()?;
        if &self.closure_digest != closure.digest() {
            return Err(invalid("manifest dependency closure fingerprint differs"));
        }
        let listed: BTreeMap<_, _> = self
            .objects
            .iter()
            .map(|o| (o.digest.clone(), o.kind))
            .collect();
        let mut reached = BTreeSet::new();
        let mut pending = self.roots.clone();
        while let Some(reference) = pending.pop() {
            if !reached.insert(reference.clone()) {
                continue;
            }
            let asset = closure
                .assets()
                .get(&reference)
                .ok_or_else(|| invalid("manifest root/dependency absent from verified closure"))?;
            if listed.get(&reference.revision_id) != Some(&ObjectKind::AssetRevision) {
                return Err(invalid(
                    "verified asset revision absent from manifest inventory",
                ));
            }
            pending.extend(
                asset
                    .revision()
                    .dependencies
                    .iter()
                    .map(|d| d.target.clone()),
            );
        }
        if reached.len() != closure.assets().len() {
            return Err(invalid(
                "manifest roots do not reach the entire provided closure",
            ));
        }
        Ok(())
    }
}

fn collect_evidence(evidence: &[Evidence], retained: &mut BTreeSet<AssetDigest>) {
    retained.extend(evidence.iter().filter_map(|e| e.retained_blob.clone()));
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod tests;
