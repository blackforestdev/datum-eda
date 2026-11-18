//! Capture one explicit edit and repin its catalog dependents without adoption.
use super::*;
use std::io::Read;

pub(super) struct Draft {
    pub(super) selected: AssetRevisionRef,
    pub(super) roots: Vec<AssetRevisionRef>,
    pub(super) references: Vec<AssetRevisionRef>,
    pub(super) content: Vec<LibraryContent>,
    pub(super) changed: Vec<AssetRevisionRef>,
}

struct Builder<'a> {
    context: &'a NativeWriteContext<'a>,
    assets: &'a BTreeMap<AssetRevisionRef, VerifiedAsset>,
    provenance: &'a WriteProvenance,
    changed_at: Option<String>,
    metadata: Option<(Uuid, AssetProvenance)>,
    bases: BTreeMap<Uuid, AssetRevisionRef>,
    edit: Option<(Uuid, Value)>,
    built: BTreeMap<Uuid, (AssetRevisionRef, AssetKind)>,
    visiting: BTreeSet<Uuid>,
    content: BTreeMap<String, LibraryContent>,
    changed: Vec<AssetRevisionRef>,
    limits: LibraryAssetLimits,
    raw_used: usize,
    source_ids: Option<BTreeMap<Uuid, usize>>,
}

pub(super) fn capture(
    context: &NativeWriteContext<'_>,
    snapshot: &LibraryAssetSnapshot,
    provenance: &WriteProvenance,
    request: CandidateRequest,
    limits: LibraryAssetLimits,
) -> Result<Draft, EngineError> {
    capture_inventory(
        context,
        &snapshot.manifest().roots,
        snapshot.assets(),
        provenance,
        request,
        limits,
    )
}

pub(super) fn capture_inventory(
    context: &NativeWriteContext<'_>,
    catalog_roots: &[AssetRevisionRef],
    assets: &BTreeMap<AssetRevisionRef, VerifiedAsset>,
    provenance: &WriteProvenance,
    request: CandidateRequest,
    limits: LibraryAssetLimits,
) -> Result<Draft, EngineError> {
    let closure = inspect_closure(
        catalog_roots,
        assets,
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    let bases: BTreeMap<_, _> = closure
        .assets()
        .keys()
        .map(|r| (r.asset_id, r.clone()))
        .collect();
    let (selected, edit) = match request.change {
        CandidateChange::CaptureNative { asset_id } => {
            if bases.contains_key(&asset_id) {
                return Err(refusal(
                    "native capture cannot overwrite an existing catalog revision",
                ));
            }
            (asset_id, None)
        }
        CandidateChange::ReviseProvenance { base } => {
            if request.provenance_override.is_none() {
                return Err(refusal(
                    "a declaration-only revision requires explicit provenance",
                ));
            }
            if bases.get(&base.asset_id) != Some(&base) {
                return Err(refusal("revision edit has a stale or foreign parent"));
            }
            (base.asset_id, None)
        }
        CandidateChange::Revise { base, payload } => {
            if bases.get(&base.asset_id) != Some(&base) {
                return Err(refusal("revision edit has a stale or foreign parent"));
            }
            let acquired = &assets[&base];
            let policy = counter_policy(
                acquired.revision().kind,
                acquired.revision().payload_schema_version,
            )?;
            acquired.payload().verify_materialization(acquired.payload().value(), policy)
                .map_err(|_| refusal("value-based editing requires a lossless acquired base; original bytes retained"))?;
            (base.asset_id, Some((base.asset_id, payload)))
        }
    };
    let mut builder = Builder {
        context,
        assets,
        provenance,
        changed_at: request.changed_at,
        metadata: request
            .provenance_override
            .map(|metadata| (selected, metadata)),
        bases,
        edit,
        built: BTreeMap::new(),
        visiting: BTreeSet::new(),
        content: BTreeMap::new(),
        changed: vec![],
        limits,
        raw_used: 0,
        source_ids: None,
    };
    let selected_kind = builder.kind(selected)?;
    let selected_ref = builder.build(selected, selected_kind)?;
    // Build all retained heads using the same memoized revisions: changing a
    // reusable symbol must repin dependent candidates, rather than retain two
    // conflicting revisions of that symbol in one release closure.
    let mut roots = Vec::new();
    for root in catalog_roots {
        roots.push(builder.build(root.asset_id, assets[root].revision().kind)?);
    }
    if !roots.iter().any(|r| r.asset_id == selected) {
        roots.push(selected_ref.clone());
    }
    roots.sort();
    roots.dedup();
    let mut references: Vec<_> = builder.built.values().map(|(r, _)| r.clone()).collect();
    references.sort();
    builder.changed.sort();
    Ok(Draft {
        selected: selected_ref,
        roots,
        references,
        content: builder.content.into_values().collect(),
        changed: builder.changed,
    })
}

impl Builder<'_> {
    fn kind(&self, id: Uuid) -> Result<AssetKind, EngineError> {
        if let Some(base) = self.bases.get(&id) {
            return Ok(self.assets[base].revision().kind);
        }
        let source = self
            .context
            .model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool")
            .ok_or_else(|| refusal("native capture needs an unambiguous pool object"))?;
        [
            AssetKind::Unit,
            AssetKind::Symbol,
            AssetKind::Entity,
            AssetKind::Package,
            AssetKind::Padstack,
            AssetKind::Footprint,
            AssetKind::Part,
            AssetKind::PinPadMap,
        ]
        .into_iter()
        .find(|k| k.pool_kind() == Some(source.kind.as_str()))
        .ok_or_else(|| refusal("unsupported native library kind"))
    }

    fn build(&mut self, id: Uuid, kind: AssetKind) -> Result<AssetRevisionRef, EngineError> {
        if kind == AssetKind::ModelAttachment {
            return Err(refusal(
                "owned model dependency transport is not yet supported",
            ));
        }
        if let Some((reference, actual_kind)) = self.built.get(&id) {
            if *actual_kind != kind {
                return Err(refusal("one asset is referenced with different kinds"));
            }
            return Ok(reference.clone());
        }
        if self.visiting.contains(&id) {
            return Err(refusal("cyclic candidate dependency"));
        }
        if self.visiting.len() >= self.limits.depth
            || self.built.len() + self.visiting.len() >= self.limits.files
        {
            return Err(refusal("candidate closure exceeds service policy"));
        }
        if self.kind(id)? != kind {
            return Err(refusal("candidate dependency kind differs"));
        }
        self.visiting.insert(id);
        let base_ref = self.bases.get(&id).cloned();
        let edited = self.edit.as_ref().is_some_and(|(edited, _)| *edited == id);
        let (mut value, raw_source) = if edited {
            (self.edit.as_ref().unwrap().1.clone(), None)
        } else if let Some(base) = &base_ref {
            (self.assets[base].payload().value().clone(), None)
        } else {
            let (value, raw) = self.native_source(id, kind)?;
            (value, Some(raw))
        };
        let policy = counter_policy(kind, 1)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| refusal("candidate payload is not an object"))?;
        if policy.object_revision {
            object.remove("object_revision");
        }
        if policy.model_revision {
            object.remove("model_revision");
        }
        if value.get("schema_version").and_then(Value::as_u64) != Some(1)
            || value
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|v| v.parse::<Uuid>().ok())
                != Some(id)
        {
            return Err(refusal("candidate payload changes its identity or schema"));
        }
        for field in ["models_3d", "behavioural_models"] {
            if value.get(field).is_some_and(|v| {
                !v.is_null()
                    && !v.as_array().is_some_and(Vec::is_empty)
                    && !v.as_object().is_some_and(serde_json::Map::is_empty)
            }) {
                return Err(refusal(
                    "owned model dependency transport is not yet supported",
                ));
            }
        }
        // Proposed registration seam: expose the existing schema-owned slot API;
        // this unregistered module does not duplicate its reference classification.
        let mut dependencies = Vec::new();
        for slot in crate::library_assets::payload_reference_slots(kind, &value)? {
            let target = self.build(slot.target_id, slot.kind)?;
            dependencies.push(Dependency {
                role: slot.path.last().unwrap().clone(),
                payload_path: slot.path,
                target,
            });
        }
        if let Some(base) = &base_ref {
            let asset = &self.assets[base];
            if !self
                .metadata
                .as_ref()
                .is_some_and(|(target, _)| *target == id)
                && asset.revision().dependencies == dependencies
                && (!edited
                    || asset
                        .payload()
                        .verify_materialization(&value, policy)
                        .is_ok())
            {
                self.visiting.remove(&id);
                self.built.insert(id, (base.clone(), kind));
                return Ok(base.clone());
            }
        }
        let payload = if let Some(base) = &base_ref
            && (!edited
                || self.assets[base]
                    .payload()
                    .verify_materialization(&value, policy)
                    .is_ok())
        {
            // The resolver already bounded and fingerprint-verified these bytes.
            self.assets[base].payload().raw_bytes().to_vec()
        } else {
            format!(
                "{}\n",
                crate::ir::serialization::to_json_deterministic(&value)?
            )
            .into_bytes()
        };
        let mut provenance = if let Some((_, metadata)) =
            self.metadata.as_ref().filter(|(target, _)| *target == id)
        {
            metadata.clone()
        } else if let Some(base) = &base_ref {
            let mut prior = self.assets[base].provenance().clone();
            prior.origin_kind = OriginKind::Derived;
            prior.source_assets.push(base.clone());
            prior.source_assets.sort();
            prior.source_assets.dedup();
            prior
        } else {
            AssetProvenance {
                schema_version: 1,
                authors: vec![],
                created_at: None,
                change_actor: None,
                changed_at: None,
                change_summary: None,
                origin_kind: OriginKind::Migration,
                source_assets: vec![],
                sources: vec![],
                rights: Rights {
                    license_expression: None,
                    attribution: vec![],
                    restrictions: vec![],
                    evidence: vec![],
                },
                evidence: vec![],
                extensions: None,
            }
        };
        if let Some(base) = &base_ref {
            provenance.source_assets.push(base.clone());
            provenance.source_assets.sort();
            provenance.source_assets.dedup();
        }
        if let Some(raw) = raw_source {
            let digest = AssetDigest::blob(&raw);
            self.retain(LibraryContent::Blob {
                fingerprint: digest.as_str().into(),
                bytes: raw,
            })?;
            provenance.sources.push(Evidence {
                title: "Exact native source snapshot".into(),
                issuer: None,
                document_revision: None,
                locator: Some(format!("pool/{}/{}.json", kind.pool_kind().unwrap(), id)),
                retained_blob: Some(digest),
                access_note: None,
            });
        }
        provenance.change_actor = Some(actor(self.provenance));
        provenance.changed_at = self.changed_at.clone();
        provenance.change_summary = Some(self.provenance.reason.clone());
        let provenance_ref = provenance.fingerprint()?;
        let extensions = base_ref
            .as_ref()
            .and_then(|base| self.assets[base].revision().extensions.clone());
        let revision = AssetRevision {
            schema_version: 1,
            asset_id: id,
            kind,
            payload_schema_version: 1,
            payload_ref: AssetDigest::blob(&payload),
            parents: base_ref.into_iter().collect(),
            dependencies,
            provenance_ref,
            extensions,
        };
        let reference = revision.reference()?;
        let revision_bytes = encode_metadata(&revision)?;
        let provenance_bytes = encode_metadata(&provenance)?;
        VerifiedAsset::read(
            &reference,
            &revision_bytes,
            &provenance_bytes,
            &payload,
            self.limits.metadata_bytes.max(self.limits.blob_bytes),
            policy,
        )?;
        self.retain(LibraryContent::Blob {
            fingerprint: revision.payload_ref.as_str().into(),
            bytes: payload,
        })?;
        self.retain(LibraryContent::Record {
            record_type: LibraryRecordType::Provenance,
            fingerprint: revision.provenance_ref.as_str().into(),
            bytes: provenance_bytes,
        })?;
        self.retain(LibraryContent::Record {
            record_type: LibraryRecordType::AssetRevision,
            fingerprint: reference.revision_id.as_str().into(),
            bytes: revision_bytes,
        })?;
        self.visiting.remove(&id);
        self.built.insert(id, (reference.clone(), kind));
        self.changed.push(reference.clone());
        Ok(reference)
    }

    fn retain(&mut self, content: LibraryContent) -> Result<(), EngineError> {
        content.validate(self.limits.metadata_bytes, self.limits.blob_bytes)?;
        let path = content.relative_path()?;
        if let Some(existing) = self.content.get(&path) {
            if existing != &content {
                return Err(refusal("candidate content collision"));
            }
            return Ok(());
        }
        self.raw_used = self
            .raw_used
            .checked_add(content.bytes().len())
            .ok_or_else(|| refusal("candidate size overflow"))?;
        if self.raw_used > self.limits.total_bytes || self.content.len() >= self.limits.files {
            return Err(refusal("candidate content exceeds server policy"));
        }
        self.content.insert(path, content);
        Ok(())
    }

    fn native_source(
        &mut self,
        id: Uuid,
        kind: AssetKind,
    ) -> Result<(Value, Vec<u8>), EngineError> {
        if self.source_ids.is_none() {
            let sources: Vec<_> = self
                .context
                .model
                .source_shards
                .iter()
                .filter(|s| s.kind == SourceShardKind::Pool)
                .collect();
            if sources.len() > self.limits.files {
                return Err(refusal("native source inventory exceeds capture policy"));
            }
            let mut total = 0usize;
            let mut ids = BTreeMap::new();
            for source in sources {
                let remaining = self
                    .limits
                    .total_bytes
                    .checked_sub(total)
                    .ok_or_else(|| refusal("native inventory read budget exhausted"))?;
                let bytes = read_owned(
                    self.context.project_root,
                    &source.relative_path,
                    self.limits.blob_bytes.min(remaining),
                )?;
                total = total
                    .checked_add(bytes.len())
                    .ok_or_else(|| refusal("native inventory size overflow"))?;
                let value: Value = serde_json::from_slice(&bytes)?;
                if let Some(id) = value
                    .get("uuid")
                    .and_then(Value::as_str)
                    .and_then(|v| v.parse::<Uuid>().ok())
                {
                    *ids.entry(id).or_insert(0) += 1;
                }
            }
            self.source_ids = Some(ids);
        }
        if self.source_ids.as_ref().and_then(|ids| ids.get(&id)) != Some(&1) {
            return Err(refusal(
                "native asset identity has missing or ambiguous sources",
            ));
        }
        let relative = format!(
            "pool/{}/{}.json",
            kind.pool_kind()
                .ok_or_else(|| refusal("owned detail is not a pool kind"))?,
            id
        );
        let shards: Vec<_> = self
            .context
            .model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::Pool && s.relative_path == relative)
            .collect();
        let [shard] = shards.as_slice() else {
            return Err(refusal("native source path is absent or ambiguous"));
        };
        if self.context.model.objects[&id].source_shard_id != shard.shard_id {
            return Err(refusal("native source registry differs"));
        }
        let raw = read_owned(self.context.project_root, &relative, self.limits.blob_bytes)?;
        use sha2::{Digest, Sha256};
        if format!("{:x}", Sha256::digest(&raw)) != shard.content_hash {
            return Err(refusal(
                "native source bytes changed or require materialization",
            ));
        }
        let value: Value = serde_json::from_slice(&raw)?;
        // Proposed shared exact-value comparison seam, required before removing
        // counters: refuse unknown numeric precision loss during native capture.
        crate::library_assets::verify_native_snapshot(&raw, &value)?;
        Ok((value, raw))
    }
}

pub(super) fn read_owned(
    root: &std::path::Path,
    relative: &str,
    limit: usize,
) -> Result<Vec<u8>, EngineError> {
    let path = crate::substrate::library_asset_resolver::owned_path(root, relative)?;
    let mut file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > limit as u64 {
        return Err(refusal("native source exceeds capture policy"));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0; 16 * 1024];
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if bytes.len().checked_add(n).is_none_or(|end| end > limit) {
            return Err(refusal("native source grew beyond capture policy"));
        }
        bytes
            .try_reserve_exact(n)
            .map_err(|_| refusal("native capture allocation refused"))?;
        if bytes.capacity() > limit {
            return Err(refusal("native capture capacity exceeds policy"));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok(bytes)
}
