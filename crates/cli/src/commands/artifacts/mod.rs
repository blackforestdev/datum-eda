// commands/artifacts — artifact generation, runs, validation, previews,
// and evidence commit paths.
//
// checks.rs is deliberately NOT declared here: it remains a child module of
// command_project_native_inspect.rs (via #[path]), whose check-view helpers
// it depends on; only the file lives in this family directory.

#[allow(unused_imports)]
use super::*;

pub(crate) mod artifacts;
mod runs;
mod validation;

pub(crate) use self::artifacts::{
    compare_native_project_artifacts, generate_native_project_artifacts,
    preview_native_project_artifact_file, query_native_project_artifact,
    query_native_project_artifact_files, query_native_project_artifacts,
};
pub(crate) use self::validation::validate_native_project_artifact;
