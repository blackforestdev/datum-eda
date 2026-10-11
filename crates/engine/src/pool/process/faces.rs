//! Physical face declarations, validated against sequence rather than numeric IDs.
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::board::{Stackup, StackupLayerType};
use crate::error::EngineError;
use crate::ir::geometry::LayerId;

use super::{EffectivePadstackProcess, EffectiveProcessAperture, error};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessFace {
    pub copper_layer: LayerId,
    pub mask_layer: Option<LayerId>,
    pub paste_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessSourceCapability {
    pub version: u32,
    pub front: ProcessFace,
    pub back: ProcessFace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessMembership {
    pub mask_layers: Vec<LayerId>,
    pub paste_layers: Vec<LayerId>,
}

impl ProcessSourceCapability {
    pub fn validate(&self, stackup: &Stackup) -> Result<(), EngineError> {
        if self.version != 1 {
            return Err(error(
                "unsupported BoardRoot process-source capability version",
            ));
        }
        let mut ids = BTreeSet::new();
        if stackup.layers.iter().any(|layer| !ids.insert(layer.id)) {
            return Err(error("ambiguous duplicate stackup layer IDs"));
        }
        let copper: Vec<_> = stackup
            .layers
            .iter()
            .filter(|layer| layer.layer_type == StackupLayerType::Copper)
            .map(|layer| layer.id)
            .collect();
        if copper.first() != Some(&self.front.copper_layer)
            || copper.last() != Some(&self.back.copper_layer)
        {
            return Err(error(
                "front/back copper must match physical stackup exterior sequence",
            ));
        }
        let mut processes = BTreeSet::new();
        for face in [&self.front, &self.back] {
            for (id, kind) in [
                (face.mask_layer, StackupLayerType::SolderMask),
                (face.paste_layer, StackupLayerType::Paste),
            ] {
                if let Some(id) = id {
                    if !processes.insert(id) {
                        return Err(error("ambiguous process layer face association"));
                    }
                    if !stackup
                        .layers
                        .iter()
                        .any(|layer| layer.id == id && layer.layer_type == kind)
                    {
                        return Err(error(
                            "process face association has missing or incorrectly typed layer",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Copper reach is already transformed by the canonical placed-Pad projection.
    /// In particular reflection and blind endpoints are not inferred here.
    pub fn membership(
        &self,
        stackup: &Stackup,
        reached_copper: &[LayerId],
        process: &EffectivePadstackProcess,
    ) -> Result<ProcessMembership, EngineError> {
        self.validate(stackup)?;
        if reached_copper.iter().any(|id| {
            !stackup
                .layers
                .iter()
                .any(|layer| layer.id == *id && layer.layer_type == StackupLayerType::Copper)
        }) {
            return Err(error(
                "process membership references unknown reached copper",
            ));
        }
        let mut result = ProcessMembership {
            mask_layers: Vec::new(),
            paste_layers: Vec::new(),
        };
        for face in [&self.front, &self.back] {
            if !reached_copper.contains(&face.copper_layer) {
                continue;
            }
            if process.mask != EffectiveProcessAperture::Suppressed {
                result.mask_layers.push(face.mask_layer.ok_or_else(|| {
                    error("enabled mask on reached face has no declared mask layer")
                })?);
            }
            if process.paste != EffectiveProcessAperture::Suppressed {
                result.paste_layers.push(face.paste_layer.ok_or_else(|| {
                    error("enabled paste on reached face has no declared paste layer")
                })?);
            }
        }
        Ok(result)
    }
}
