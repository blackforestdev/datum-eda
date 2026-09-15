//! Public Revision Engine surface projected through the shared native API.
//!
//! This is a read/query facade, not a Design-operation builder. Revision
//! authority mutations remain their explicit typed APIs, while every Design
//! commit continues through the one revision preflight in the canonical
//! commit coordinator.

pub use crate::revision::{
    REVISION_PROPOSAL_TWINS, REVISION_PUBLIC_OPERATIONS, REVISION_PUBLIC_QUERIES,
    REVISION_PUBLIC_REFUSALS, RevisionPublicCatalog, RevisionQueryRequest, RevisionQueryResponse,
    RevisionRefusalPayload, query_revision_authority, render_revision_query_human,
    revision_public_catalog,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::registry::native_write_verbs;

    #[test]
    fn public_catalog_is_the_native_api_inventory_without_a_second_design_writer() {
        let catalog = revision_public_catalog();
        assert_eq!(catalog.operations, REVISION_PUBLIC_OPERATIONS);
        assert_eq!(catalog.queries, REVISION_PUBLIC_QUERIES);
        assert_eq!(catalog.refusals, REVISION_PUBLIC_REFUSALS);
        assert!(
            native_write_verbs()
                .iter()
                .all(|verb| !verb.id.starts_with("datum.revision.")),
            "revision authority must not masquerade as a second Design writer"
        );
    }
}
