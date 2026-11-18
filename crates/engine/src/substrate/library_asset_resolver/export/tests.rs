//! Exchange integrity and filesystem negative controls, not CAD/content approval.
use super::*;
use crate::library_assets::{
    Actor, AssetDigest, AssetKind, AssetProvenance, AssetRevision, LocalCounters, PROFILE,
    VerifiedAsset, encode_metadata,
    manifest::{BlobEntry, LibraryDiscovery, LibraryReleaseManifest, ObjectEntry, ObjectKind},
};
use crate::substrate::library_asset_resolver::{LibraryAssetLimits, configure_server_policy};
use std::{fs, os::unix::fs::symlink, path::PathBuf};
use uuid::Uuid;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-offline-export-{}", Uuid::new_v4()));
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            for (relative, bytes) in inventory(&path) {
                files.insert(
                    PathBuf::from(path.file_name().unwrap()).join(relative),
                    bytes,
                );
            }
        } else {
            files.insert(path.file_name().unwrap().into(), fs::read(&path).unwrap());
        }
    }
    files
}

fn managed_source(root: &Path) -> PathBuf {
    let project = root.join("project");
    let library = project.join("library");
    fs::create_dir_all(library.join("objects/sha256")).unwrap();
    fs::create_dir_all(library.join("blobs/sha256")).unwrap();
    let payload = br#"{ "schema_version":1, "uuid":"00000000-0000-0000-0000-000000000001", "name":"Native unit", "manufacturer":"", "pins":{}, "tags":[] }
"#;
    let provenance: AssetProvenance = serde_json::from_value(json!({
        "schema_version":1,"authors":[],"created_at":null,"change_actor":null,
        "changed_at":null,"change_summary":null,"origin_kind":"native",
        "source_assets":[],"sources":[],"evidence":[],
        "rights":{"license_expression":null,"attribution":[],"restrictions":[],"evidence":[]}
    }))
    .unwrap();
    let revision = AssetRevision {
        schema_version: 1,
        asset_id: Uuid::from_u128(1),
        kind: AssetKind::Unit,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(payload),
        parents: vec![],
        dependencies: vec![],
        provenance_ref: provenance.fingerprint().unwrap(),
        extensions: None,
    };
    let reference = revision.reference().unwrap();
    let revision_bytes = encode_metadata(&revision).unwrap();
    let provenance_bytes = encode_metadata(&provenance).unwrap();
    let verified = VerifiedAsset::read(
        &reference,
        &revision_bytes,
        &provenance_bytes,
        payload,
        1_000_000,
        LocalCounters::default(),
    )
    .unwrap();
    let assets = BTreeMap::from([(reference.clone(), verified)]);
    let closure = inspect_closure(
        std::slice::from_ref(&reference),
        &assets,
        ClosureLimits {
            max_nodes: 100,
            max_depth: 100,
        },
    )
    .unwrap();
    let extra = b"Retained inventory bytes outside engineering payload";
    let extra_digest = AssetDigest::blob(extra);
    let mut manifest = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: Uuid::from_u128(20),
        release_id: Uuid::from_u128(21),
        publisher: Actor {
            recorded_id: None,
            display_name: None,
            verification_evidence: vec![],
        },
        profile: PROFILE.into(),
        roots: vec![reference.clone()],
        closure_digest: closure.digest().clone(),
        objects: vec![
            ObjectEntry {
                digest: reference.revision_id.clone(),
                kind: ObjectKind::AssetRevision,
            },
            ObjectEntry {
                digest: revision.provenance_ref.clone(),
                kind: ObjectKind::Provenance,
            },
        ],
        blobs: vec![
            BlobEntry {
                digest: revision.payload_ref.clone(),
                byte_length: payload.len() as u64,
                media_type: "application/json".into(),
            },
            BlobEntry {
                digest: extra_digest.clone(),
                byte_length: extra.len() as u64,
                media_type: "text/plain".into(),
            },
        ],
        publication_record_refs: vec![],
        extensions: None,
    };
    manifest.objects.sort_by(|a, b| a.digest.cmp(&b.digest));
    manifest.blobs.sort_by(|a, b| a.digest.cmp(&b.digest));
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: manifest.collection_id,
        profile: PROFILE.into(),
        manifest_ref: manifest.fingerprint().unwrap(),
    };
    for (digest, bytes) in [
        (reference.revision_id, revision_bytes),
        (revision.provenance_ref, provenance_bytes),
        (
            discovery.manifest_ref.clone(),
            encode_metadata(&manifest).unwrap(),
        ),
    ] {
        fs::write(
            library.join(format!("objects/sha256/{}.json", digest.hex())),
            bytes,
        )
        .unwrap();
    }
    for (digest, bytes) in [
        (revision.payload_ref, payload.as_slice()),
        (extra_digest, extra.as_slice()),
    ] {
        fs::write(
            library.join(format!("blobs/sha256/{}", digest.hex())),
            bytes,
        )
        .unwrap();
    }
    fs::write(
        library.join("library.json"),
        encode_metadata(&discovery).unwrap(),
    )
    .unwrap();
    fs::write(
        library.join("lock.json"),
        b"source lock must not be read or changed",
    )
    .unwrap();
    fs::create_dir_all(project.join(".datum/journal")).unwrap();
    fs::write(
        project.join(".datum/journal/transactions.jsonl"),
        b"source journal sentinel\n",
    )
    .unwrap();
    project
}

#[test]
fn verified_inventory_roundtrip_preserves_exact_bytes_and_refuses_invalid_source() {
    // Server policy is process-global; run this case in its own test process.
    const KEY: &str = "DATUM_OFFLINE_EXPORT_TEST_CHILD";
    if std::env::var_os(KEY).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "substrate::library_asset_resolver::export::tests::verified_inventory_roundtrip_preserves_exact_bytes_and_refuses_invalid_source", "--nocapture"])
            .env(KEY,"1").output().unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
        return;
    }
    configure_server_policy(LibraryAssetLimits {
        metadata_bytes: 1_000_000,
        blob_bytes: 1_000_000,
        files: 100,
        depth: 100,
        total_bytes: 10_000_000,
    })
    .unwrap();
    let fixture = Fixture::new();
    let source = managed_source(&fixture.0);
    let before = inventory(&source);
    let destination = fixture.0.join("exported");
    let report = export_candidate(&source, &destination).unwrap();
    assert_eq!(report["status"], "exported");
    let mut expected = inventory(&source.join("library"));
    expected.remove(Path::new("lock.json"));
    assert_eq!(inventory(&destination), expected);
    assert_eq!(
        crate::api::Engine::inspect_library_candidate(&destination).unwrap()["inspection_status"],
        "available"
    );
    assert_eq!(inventory(&source), before);
    assert!(export_candidate(&source, &destination).is_err());
    assert_eq!(inventory(&destination), expected);
    let blob = fs::read_dir(source.join("library/blobs/sha256"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(blob, b"corrupt input").unwrap();
    let invalid_before = inventory(&source);
    assert!(export_candidate(&source, &fixture.0.join("invalid")).is_err());
    assert!(!fixture.0.join("invalid").exists());
    assert_eq!(inventory(&source), invalid_before);
    assert!(!fs::read_dir(&fixture.0).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".datum-library-export-")
    }));
}

#[test]
fn no_replace_publication_preserves_existing_and_racing_destinations() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    fs::create_dir(&source).unwrap();
    for name in ["empty", "nonempty", "file", "link"] {
        let dest = fixture.0.join(name);
        match name {
            "empty" => fs::create_dir(&dest).unwrap(),
            "nonempty" => {
                fs::create_dir(&dest).unwrap();
                fs::write(dest.join("keep"), b"unchanged").unwrap();
            }
            "file" => fs::write(&dest, b"unchanged").unwrap(),
            _ => symlink(&source, &dest).unwrap(),
        }
        assert!(writer::Stage::new(&source, &dest).is_err());
    }
    let racing = fixture.0.join("racing");
    let mut stage = writer::Stage::new(&source, &racing).unwrap();
    stage
        .write_files(&BTreeMap::from([(
            "library.json".into(),
            b"stage".to_vec(),
        )]))
        .unwrap();
    fs::create_dir(&racing).unwrap();
    fs::write(racing.join("keep"), b"racer").unwrap();
    assert!(stage.publish().is_err());
    assert_eq!(fs::read(racing.join("keep")).unwrap(), b"racer");
    assert_eq!(
        fs::read(fixture.0.join("nonempty/keep")).unwrap(),
        b"unchanged"
    );
    assert_eq!(fs::read(fixture.0.join("file")).unwrap(), b"unchanged");
    assert!(
        fs::symlink_metadata(fixture.0.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!fs::read_dir(&fixture.0).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".datum-library-export-")
    }));
}

#[test]
fn destination_aliases_into_source_and_missing_parents_refuse_without_writes() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    fs::create_dir(&source).unwrap();
    let alias = fixture.0.join("alias");
    symlink(&source, &alias).unwrap();
    for dest in [
        source.join("nested"),
        alias.join("nested"),
        source.join("../source/nested"),
        fixture.0.join("missing/nested"),
    ] {
        assert!(writer::Stage::new(&source, &dest).is_err());
    }
    assert!(inventory(&source).is_empty());
    assert!(!fixture.0.join("missing").exists());
    let mut stage = writer::Stage::new(&source, &fixture.0.join("bad-path")).unwrap();
    assert!(
        stage
            .write_files(&BTreeMap::from([("../escaped".into(), vec![])]))
            .is_err()
    );
    drop(stage);
    assert!(!fixture.0.join("escaped").exists());
    assert!(!fs::read_dir(&fixture.0).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".datum-library-export-")
    }));
}

#[test]
fn cleanup_never_removes_a_replacement_stage() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    fs::create_dir(&source).unwrap();
    let stage = writer::Stage::new(&source, &fixture.0.join("destination")).unwrap();
    let owned = fs::read_dir(&fixture.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".datum-library-export-")
        })
        .unwrap();
    fs::rename(&owned, fixture.0.join("moved-owned-stage")).unwrap();
    fs::create_dir(&owned).unwrap();
    fs::write(owned.join("replacement"), b"do not remove").unwrap();
    assert!(stage.publish().is_err());
    assert_eq!(
        fs::read(owned.join("replacement")).unwrap(),
        b"do not remove"
    );
}

#[test]
fn writable_parent_and_non_utf8_resolved_destinations_refuse_before_staging() {
    use std::os::unix::{ffi::OsStringExt, fs::PermissionsExt};
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    let parent = fixture.0.join("output-parent");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&parent).unwrap();
    for mode in [0o720, 0o702] {
        fs::set_permissions(&parent, fs::Permissions::from_mode(mode)).unwrap();
        assert!(writer::Stage::new(&source, &parent.join("destination")).is_err());
        assert!(inventory(&parent).is_empty());
    }
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    let invalid = std::ffi::OsString::from_vec(vec![b'x', 0xff]);
    assert!(writer::Stage::new(&source, &parent.join(&invalid)).is_err());
    let invalid_parent = fixture.0.join(&invalid);
    fs::create_dir(&invalid_parent).unwrap();
    let alias = fixture.0.join("utf8-alias");
    symlink(&invalid_parent, &alias).unwrap();
    assert!(writer::Stage::new(&source, &alias.join("destination")).is_err());
    assert!(inventory(&invalid_parent).is_empty());
    assert!(inventory(&parent).is_empty());
    assert!(inventory(&source).is_empty());
}
