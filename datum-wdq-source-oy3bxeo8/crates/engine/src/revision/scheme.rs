use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    AuthorityDiagnostic, AuthorityRecord, AuthoritySnapshot, ConfigurationItemId, RevisionSchemeId,
};

pub const REVISION_SCHEME_KINDS: &[RevisionSchemeKind] = &[
    RevisionSchemeKind::LinearAlphabetic,
    RevisionSchemeKind::LinearNumeric,
    RevisionSchemeKind::IssueRevision,
    RevisionSchemeKind::Iso19650InformationContainer,
    RevisionSchemeKind::OrganizationCustom,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionSchemeKind {
    LinearAlphabetic,
    LinearNumeric,
    IssueRevision,
    Iso19650InformationContainer,
    OrganizationCustom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionSchemeEntry {
    pub ordinal: u64,
    pub revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suitability_or_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionNamespaceRule {
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionSchemeData {
    pub scheme_version: u64,
    pub kind: RevisionSchemeKind,
    pub entries: Vec<RevisionSchemeEntry>,
    pub namespaces: BTreeMap<ConfigurationItemId, RevisionNamespaceRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RevisionSchemeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionSchemeResolution {
    Resolved(RevisionSchemeData),
    Refused(AuthorityDiagnostic),
}

pub fn resolve_revision_scheme(
    snapshot: &AuthoritySnapshot,
    scheme_id: RevisionSchemeId,
    configuration_item_id: ConfigurationItemId,
) -> RevisionSchemeResolution {
    let Some(body) = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::RevisionScheme(body) if body.id == scheme_id => Some(body),
        _ => None,
    }) else {
        return refused("revision_scheme_missing", "revision scheme does not exist");
    };
    let diagnostics = validate_scheme_data(&body.semantics);
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return RevisionSchemeResolution::Refused(diagnostic);
    }
    if !body
        .semantics
        .namespaces
        .contains_key(&configuration_item_id)
    {
        return refused(
            "revision_scheme_namespace_missing",
            "ConfigurationItem has no scheme namespace",
        );
    }
    RevisionSchemeResolution::Resolved(body.semantics.clone())
}

pub(crate) fn validate_scheme_data(data: &RevisionSchemeData) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    if data.scheme_version == 0 {
        diagnostics.push(diag(
            "revision_scheme_invalid_version",
            "scheme version must be positive",
        ));
    }
    if data.entries.is_empty() {
        diagnostics.push(diag(
            "revision_scheme_exhausted",
            "scheme contains no declared sequence entries",
        ));
    }
    let ordinals: BTreeSet<_> = data.entries.iter().map(|entry| entry.ordinal).collect();
    let labels: BTreeSet<_> = data
        .entries
        .iter()
        .map(|entry| (&entry.revision, &entry.suitability_or_status))
        .collect();
    if ordinals.len() != data.entries.len() || labels.len() != data.entries.len() {
        diagnostics.push(diag(
            "revision_scheme_ambiguous_sequence",
            "scheme ordinals and revision/status pairs must be unique",
        ));
    }
    if data.kind != RevisionSchemeKind::Iso19650InformationContainer
        && data
            .entries
            .iter()
            .any(|entry| entry.suitability_or_status.is_some())
    {
        diagnostics.push(diag(
            "revision_scheme_status_axis_ineligible",
            "only the ISO 19650 scheme carries a separate suitability/status axis",
        ));
    }
    diagnostics
}

fn refused(code: &str, message: &str) -> RevisionSchemeResolution {
    RevisionSchemeResolution::Refused(diag(code, message))
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
