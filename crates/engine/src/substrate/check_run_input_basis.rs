//! Check profile metadata and the native DRC zone-copper input qualifier.
//! Authored revision remains a separate required basis. This receipt covers
//! resolved ZoneFill inputs, never every generated artifact or manufacturing.
use super::{CheckRun, StandardsBasis};
use crate::error::EngineError;
use crate::substrate::DesignModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CheckRunProfileBasis {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis_detail: Option<StandardsBasis>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ZoneCopperBasis {
    version: u64,
    scope: String,
    fingerprint: String,
}

fn capture(model: &DesignModel) -> Result<ZoneCopperBasis, EngineError> {
    let mut bytes = b"datum:native-drc-zone-copper:v1\0".to_vec();
    // The resolver owns Unfilled, Stale, Unsupported and authenticated empty
    // Filled. Hash their actual ordered inputs, not an invented copper bound.
    bytes.extend(serde_json::to_vec(&model.zone_fills)?);
    Ok(ZoneCopperBasis {
        version: 1,
        scope: "resolved_zone_fills".into(),
        fingerprint: format!("sha256:{}", crate::substrate::sha256_hex(&bytes)),
    })
}

impl DesignModel {
    pub fn native_drc_zone_copper_basis(&self) -> Result<Value, EngineError> {
        Ok(serde_json::to_value(capture(self)?)?)
    }

    pub fn native_drc_zone_copper_fingerprint(&self) -> Result<String, EngineError> {
        Ok(capture(self)?.fingerprint)
    }

    /// Missing or unknown receipts are unqualified, rather than proof that
    /// copper changed. Recorded findings remain readable in either case.
    pub fn qualify_native_drc_zone_copper_basis(
        &self,
        receipt: Option<&Value>,
    ) -> Result<(), String> {
        let recorded = CheckRun::native_drc_zone_copper_basis_fingerprint(receipt)?;
        let current = self
            .native_drc_zone_copper_fingerprint()
            .map_err(|e| e.to_string())?;
        if recorded != current {
            return Err("Zone copper inputs changed; rerun DRC".into());
        }
        Ok(())
    }
}

impl CheckRun {
    pub const NATIVE_DRC_ZONE_COPPER_BASIS_KEY: &'static str = "native_drc_zone_copper_basis";

    pub fn validate_native_drc_zone_copper_fingerprint(value: &str) -> Result<(), String> {
        if super::is_sha256_fingerprint(value) {
            Ok(())
        } else {
            Err("Malformed native DRC zone-copper fingerprint".into())
        }
    }

    pub fn native_drc_zone_copper_basis_fingerprint(
        receipt: Option<&Value>,
    ) -> Result<String, String> {
        let value = receipt.ok_or("Native DRC zone-copper basis unavailable; rerun DRC")?;
        let basis: ZoneCopperBasis = serde_json::from_value(value.clone())
            .map_err(|_| "Malformed native DRC zone-copper basis; rerun DRC")?;
        if basis.version != 1 {
            return Err("Unsupported native DRC zone-copper basis version; rerun DRC".into());
        }
        if basis.scope != "resolved_zone_fills"
            || Self::validate_native_drc_zone_copper_fingerprint(&basis.fingerprint).is_err()
        {
            return Err("Unqualified native DRC zone-copper basis; rerun DRC".into());
        }
        Ok(basis.fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    use crate::substrate::{ObjectRevision, ProjectResolver, ZoneFill, ZoneFillState};
    use uuid::Uuid;

    #[test]
    fn receipt_rejects_legacy_unknown_or_malformed_without_claiming_currentness() {
        assert!(
            CheckRun::native_drc_zone_copper_basis_fingerprint(None)
                .unwrap_err()
                .contains("unavailable")
        );
        for receipt in [
            serde_json::json!({}),
            serde_json::json!({"version":2,"scope":"resolved_zone_fills","fingerprint":format!("sha256:{}","a".repeat(64))}),
            serde_json::json!({"version":1,"scope":"other_artifacts","fingerprint":format!("sha256:{}","a".repeat(64))}),
        ] {
            assert!(CheckRun::native_drc_zone_copper_basis_fingerprint(Some(&receipt)).is_err());
        }
    }

    #[test]
    fn resolved_empty_unsupported_and_restored_inputs_have_distinct_honest_basis() {
        // Pure input-qualification test. Canonical fill/history integration is
        // exercised separately; direct map changes here are not write proof.
        let root = std::env::temp_dir().join(format!("datum-drc-zone-basis-{}", Uuid::new_v4()));
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Zone basis".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let revision = model.model_revision.clone();
        let before = model.native_drc_zone_copper_basis().unwrap();
        let id = Uuid::new_v4();
        model.zone_fills.insert(
            id,
            ZoneFill {
                schema_version: 2,
                zone_id: id,
                state: ZoneFillState::Filled,
                source_zone_revision: ObjectRevision(0),
                model_revision: revision.clone(),
                islands: Vec::new(),
                provenance: Some("certified empty input".into()),
            },
        );
        let empty = model.native_drc_zone_copper_basis().unwrap();
        assert_ne!(before, empty);
        assert!(
            model
                .qualify_native_drc_zone_copper_basis(Some(&before))
                .unwrap_err()
                .contains("changed")
        );
        assert!(
            model
                .qualify_native_drc_zone_copper_basis(Some(&empty))
                .is_ok()
        );
        model.zone_fills.get_mut(&id).unwrap().state = ZoneFillState::Unsupported;
        assert_ne!(empty, model.native_drc_zone_copper_basis().unwrap());
        model.zone_fills.remove(&id);
        assert!(
            model
                .qualify_native_drc_zone_copper_basis(Some(&before))
                .is_ok()
        );
        assert_eq!(model.model_revision, revision);
        std::fs::remove_dir_all(root).unwrap();
    }
}
