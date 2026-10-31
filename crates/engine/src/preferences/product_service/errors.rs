//! Typed-error projection for the Global Preferences product service.
//!
//! Extracted from `product_service.rs` at the PM022 ceiling crossing that the
//! refusal-precedence fix required: typed-error construction (`error`) and the
//! service-refusal-to-wire-error mapping (`map_refusal`) are one coherent
//! responsibility owned here, while the mutation path, actor/receipt ordering and
//! `context()` stay in the parent. No forwarding layer and no public-API change --
//! `error` keeps its module-wide reach for the sibling callers (product_proposal,
//! product_dispatch, product_query, product_idempotency, project_genesis) and
//! `map_refusal` stays reachable to the parent's mutation path only.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::preferences::{
    PreferenceErrorCodeV1, PreferenceErrorV1, PreferenceServiceRefusal,
    PreferenceServiceRefusalKind,
};

use super::GlobalPreferencesProductService;

impl GlobalPreferencesProductService {
    pub(in crate::preferences) fn error(
        &self,
        code: PreferenceErrorCodeV1,
        message: &str,
        details: BTreeMap<String, Value>,
        preserved_draft: Option<Value>,
    ) -> PreferenceErrorV1 {
        PreferenceErrorV1 {
            code,
            message: message.to_owned(),
            details: Box::new(details),
            current_context: Box::new(self.context()),
            preserved_draft: preserved_draft.map(Box::new),
            preserved_proposal: None,
        }
    }

    pub(super) fn map_refusal(&self, refusal: PreferenceServiceRefusal) -> PreferenceErrorV1 {
        let code = match refusal.kind {
            PreferenceServiceRefusalKind::StaleGeneration => PreferenceErrorCodeV1::StaleGeneration,
            PreferenceServiceRefusalKind::WriterConflict => PreferenceErrorCodeV1::WriterConflict,
            PreferenceServiceRefusalKind::InvalidValue => {
                PreferenceErrorCodeV1::InvalidPreferenceValue
            }
            PreferenceServiceRefusalKind::IneligibleSource => {
                PreferenceErrorCodeV1::IneligibleSource
            }
            PreferenceServiceRefusalKind::UnreadableRepository => {
                PreferenceErrorCodeV1::RepositoryUnreadable
            }
            PreferenceServiceRefusalKind::MigrationRequired => {
                PreferenceErrorCodeV1::MigrationRequired
            }
            PreferenceServiceRefusalKind::Repository => PreferenceErrorCodeV1::RepositoryIo,
        };
        self.error(
            code,
            &refusal.message,
            BTreeMap::new(),
            refusal.preserved_draft,
        )
    }
}
