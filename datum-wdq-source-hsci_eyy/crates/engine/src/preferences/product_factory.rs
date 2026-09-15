//! Preference-independent factory-mode construction for Project genesis.

use super::{GlobalPreferencesProductService, GlobalPreferencesService, PreferenceErrorV1};

impl GlobalPreferencesProductService {
    /// Construct a resolver restricted to the built-in factory seed. It owns
    /// no configuration path and performs no preference read, creation,
    /// migration, or recovery action.
    pub fn factory_only(writer_instance: impl Into<String>) -> Result<Self, PreferenceErrorV1> {
        let service =
            GlobalPreferencesService::factory_only(writer_instance, "Factory Units · built in")
                .map_err(|error| super::product_service::bootstrap_error(error.to_string()))?;
        let active_catalog_digest = super::product_service::catalog_digest(&service)?;
        Ok(Self {
            service,
            active_catalog_digest,
            factory_only: true,
        })
    }
}
