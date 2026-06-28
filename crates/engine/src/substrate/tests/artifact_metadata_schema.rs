use super::*;
use crate::substrate::artifact::persist_artifact_metadata;

const VALID_ARTIFACT_SHA256: &str =
    "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b";

fn minimal_artifact_metadata(
    project_id: Uuid,
    model_revision: ModelRevision,
    artifact_id: Uuid,
) -> ArtifactMetadata {
    ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision,
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from("fab/board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    }
}

#[test]
fn artifact_metadata_helper_rejects_unsupported_payload_schema_version() {
    let root = temp_project_root("artifact_metadata_helper_rejects_payload_schema");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let artifact_id = Uuid::new_v5(&project_id, b"unsupported-artifact-metadata-schema");
    let mut metadata = minimal_artifact_metadata(project_id, model.model_revision, artifact_id);
    metadata.schema_version = ARTIFACT_METADATA_SCHEMA_VERSION + 1;

    let error = persist_artifact_metadata(&root, &metadata)
        .expect_err("unsupported artifact metadata schema should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported artifact metadata schema_version 2")
    );
    assert!(
        !root
            .join(format!(".datum/artifacts/{artifact_id}.json"))
            .exists()
    );
}

#[test]
fn resolver_defaults_legacy_artifact_metadata_payload_schema_version() {
    let root = temp_project_root("legacy_artifact_metadata_payload_schema");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let artifact_id = Uuid::new_v5(&project_id, b"legacy-artifact-metadata-schema");

    write_json(
        &root.join(format!(".datum/artifacts/{artifact_id}.json")),
        serde_json::json!({
            "artifact_id": artifact_id,
            "kind": "gerber_set",
            "project_id": project_id,
            "model_revision": model.model_revision.0,
            "output_job": null,
            "variant": null,
            "generator_version": "unit-test",
            "output_dir": "fab",
            "files": [{
                "path": "fab/board-F_Cu.gbr",
                "sha256": VALID_ARTIFACT_SHA256
            }],
            "production_projections": [],
            "validation_state": "not_validated"
        }),
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve legacy artifact metadata");

    assert_eq!(
        resolved.artifact_metadata[&artifact_id].schema_version,
        ARTIFACT_METADATA_SCHEMA_VERSION
    );
}
