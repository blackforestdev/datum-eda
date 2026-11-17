use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use super::invalid;
use crate::error::EngineError;

/// A content identity, never an authenticity, approval or rights assertion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct AssetDigest(String);

impl AssetDigest {
    pub(crate) fn parse(value: String) -> Result<Self, EngineError> {
        let hex = value
            .strip_prefix("sha256:")
            .ok_or_else(|| invalid("unsupported fingerprint algorithm"))?;
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid(
                "fingerprint requires 64 lowercase hexadecimal digits",
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn blob(bytes: &[u8]) -> Self {
        Self(format!("sha256:{:x}", Sha256::digest(bytes)))
    }

    pub(crate) fn record(domain: RecordDomain, bytes: &[u8]) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"datum.library.");
        hash.update(domain.name().as_bytes());
        hash.update(b".v1\n");
        hash.update(bytes);
        Self(format!("sha256:{:x}", hash.finalize()))
    }

    pub(crate) fn verify_blob(&self, bytes: &[u8]) -> Result<(), EngineError> {
        if *self != Self::blob(bytes) {
            return Err(invalid("payload fingerprint differs from retained bytes"));
        }
        Ok(())
    }

    pub(crate) fn hex(&self) -> &str {
        &self.0[7..]
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AssetDigest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordDomain {
    AssetRevision,
    Provenance,
    Review,
    ReleaseManifest,
    Closure,
}

impl RecordDomain {
    fn name(self) -> &'static str {
        match self {
            Self::AssetRevision => "asset-revision",
            Self::Provenance => "provenance",
            Self::Review => "review",
            Self::ReleaseManifest => "release-manifest",
            Self::Closure => "closure",
        }
    }
}
