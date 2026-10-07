use super::*;
pub(super) fn validate_schema_version(
    issues: &mut Vec<NativeProjectValidationIssueView>,
    subject: String,
    schema_version: u32,
) {
    if schema_version != SUPPORTED_NATIVE_SCHEMA_VERSION {
        push_issue(
            issues,
            "error",
            "invalid_schema_version",
            subject,
            format!(
                "unsupported schema_version {}; expected {}",
                schema_version, SUPPORTED_NATIVE_SCHEMA_VERSION
            ),
        );
    }
}
