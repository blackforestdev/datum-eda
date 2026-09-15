use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AuthorityDiagnostic, AuthoritySnapshot, IntegrityDiagnostic, IntegrityGeneration,
    RevisionAuthorityStore, canonical::digest_bytes,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum AuthorityResolution {
    Unconfigured,
    Resolved {
        snapshot: AuthoritySnapshot,
    },
    ReadOnlyDiagnostic {
        diagnostics: Vec<AuthorityDiagnostic>,
        preserved_snapshot_bytes: Vec<u8>,
    },
}

impl RevisionAuthorityStore {
    pub fn resolve_authority(&self, project_id: Uuid) -> AuthorityResolution {
        self.resolve_authority_inner(project_id)
            .unwrap_or_else(|error| AuthorityResolution::ReadOnlyDiagnostic {
                diagnostics: vec![AuthorityDiagnostic {
                    code: "authority_integrity_unavailable".to_string(),
                    message: error.to_string(),
                }],
                preserved_snapshot_bytes: Vec::new(),
            })
    }

    fn resolve_authority_inner(
        &self,
        project_id: Uuid,
    ) -> Result<AuthorityResolution, EngineError> {
        let Some(head) = self.read_head().transpose()? else {
            return Ok(AuthorityResolution::Unconfigured);
        };
        if head.project_id != project_id {
            return Err(EngineError::Validation(
                "authority head belongs to another Project".to_string(),
            ));
        }
        let generation: IntegrityGeneration =
            serde_json::from_slice(&std::fs::read(self.generation_path(&head.integrity_root))?)?;
        self.verify_generation(&generation)?;
        if generation.integrity_root != head.integrity_root
            || generation.transaction_id != head.transaction_id
        {
            return Err(EngineError::Validation(
                "authority head does not name its verified generation".to_string(),
            ));
        }
        let Some(digest) = generation.authority_snapshot_blob else {
            return Ok(AuthorityResolution::Unconfigured);
        };
        let bytes = std::fs::read(self.blob_path(&digest))?;
        if digest_bytes(&bytes) != digest {
            return Err(EngineError::Validation(
                "authority snapshot integrity mismatch".to_string(),
            ));
        }
        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(error) => {
                return Ok(AuthorityResolution::ReadOnlyDiagnostic {
                    diagnostics: vec![AuthorityDiagnostic {
                        code: "authority_malformed_payload".to_string(),
                        message: error.to_string(),
                    }],
                    preserved_snapshot_bytes: bytes,
                });
            }
        };
        let schema_version = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64);
        if schema_version != Some(super::AUTHORITY_SCHEMA_VERSION) {
            return Ok(AuthorityResolution::ReadOnlyDiagnostic {
                diagnostics: vec![AuthorityDiagnostic {
                    code: "authority_unsupported_schema".to_string(),
                    message: "authority snapshot schema is unknown and was preserved".to_string(),
                }],
                preserved_snapshot_bytes: bytes,
            });
        }
        let snapshot: AuthoritySnapshot = match serde_json::from_value(value) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return Ok(AuthorityResolution::ReadOnlyDiagnostic {
                    diagnostics: vec![AuthorityDiagnostic {
                        code: "authority_unknown_kind_or_payload".to_string(),
                        message: error.to_string(),
                    }],
                    preserved_snapshot_bytes: bytes,
                });
            }
        };
        if snapshot.canonical_bytes()? != bytes {
            return Ok(AuthorityResolution::ReadOnlyDiagnostic {
                diagnostics: vec![AuthorityDiagnostic {
                    code: "authority_noncanonical_payload".to_string(),
                    message: "authority snapshot is valid JSON but not canonical".to_string(),
                }],
                preserved_snapshot_bytes: bytes,
            });
        }
        let diagnostics = snapshot.validate();
        if diagnostics.is_empty() {
            Ok(AuthorityResolution::Resolved { snapshot })
        } else {
            Ok(AuthorityResolution::ReadOnlyDiagnostic {
                diagnostics,
                preserved_snapshot_bytes: bytes,
            })
        }
    }
}

impl From<IntegrityDiagnostic> for AuthorityDiagnostic {
    fn from(value: IntegrityDiagnostic) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}
