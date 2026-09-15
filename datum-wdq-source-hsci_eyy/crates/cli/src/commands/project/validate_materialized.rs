use super::{NativeProjectValidationIssueView, push_issue, relative_subject};
use eda_engine::substrate::{DesignModel, SourceShardKind};
use serde::de::DeserializeOwned;
use std::path::Path;

pub(super) fn load_materialized_kind_document<T: DeserializeOwned>(
    model: &DesignModel,
    kind: SourceShardKind,
    root: &Path,
    path: &Path,
    missing_code: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<T> {
    match model.materialized_source_shard_value(kind) {
        Ok(value) => parse_materialized_document(root, path, value, issues),
        Err(err) => {
            push_issue(
                issues,
                "error",
                missing_code,
                relative_subject(root, path),
                format!("required native project file is missing from resolver model: {err}"),
            );
            None
        }
    }
}

pub(super) fn load_materialized_relative_path_document<T: DeserializeOwned>(
    model: &DesignModel,
    root: &Path,
    path: &Path,
    missing_code: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<T> {
    let relative_path = relative_subject(root, path);
    match model.materialized_source_shard_value_by_relative_path(&relative_path) {
        Ok(value) => parse_materialized_document(root, path, value, issues),
        Err(err) => {
            push_issue(
                issues,
                "error",
                missing_code,
                relative_path,
                format!("referenced native project file is missing from resolver model: {err}"),
            );
            None
        }
    }
}

fn parse_materialized_document<T: DeserializeOwned>(
    root: &Path,
    path: &Path,
    value: serde_json::Value,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<T> {
    match serde_json::from_value::<T>(value) {
        Ok(document) => Some(document),
        Err(err) => {
            push_issue(
                issues,
                "error",
                "invalid_json",
                relative_subject(root, path),
                format!("failed to parse materialized JSON: {err}"),
            );
            None
        }
    }
}
