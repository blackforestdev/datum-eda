use super::*;

pub(crate) fn execute_revision_command(
    format: &OutputFormat,
    command: RevisionCommands,
) -> Result<(String, i32)> {
    match command {
        RevisionCommands::Catalog => Ok((
            match format {
                OutputFormat::Json => render_output(
                    format,
                    &eda_engine::api::native_write::revision::revision_public_catalog(),
                ),
                OutputFormat::Text => {
                    let catalog =
                        eda_engine::api::native_write::revision::revision_public_catalog();
                    format!(
                        "revision operations: {}\nrevision queries: {}\nrevision refusals: {}\nproposal twins: {}\nauthority record families: {}",
                        catalog.operations.len(),
                        catalog.queries.len(),
                        catalog.refusals.len(),
                        catalog.proposal_twins.len(),
                        catalog.authority_record_families.len(),
                    )
                }
            },
            0,
        )),
        RevisionCommands::Query(args) => {
            let model = eda_engine::substrate::ProjectResolver::new(&args.path).resolve()?;
            let request = eda_engine::api::native_write::revision::RevisionQueryRequest {
                project_id: model.project.project_id,
                query: args.query,
                as_of_sequence: args.as_of_sequence,
                expected_model_revision: args.expected_model_revision,
            };
            let response = eda_engine::api::native_write::revision::query_revision_authority(
                &args.path, &request,
            )?;
            let refused = response.refusal.is_some();
            let output = match format {
                OutputFormat::Json => render_output(format, &response),
                OutputFormat::Text => {
                    eda_engine::api::native_write::revision::render_revision_query_human(&response)
                }
            };
            Ok((output, if refused { 1 } else { 0 }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};

    #[test]
    fn cli_json_is_the_engine_payload_and_text_is_human_facing() {
        let (json_output, code) =
            execute_revision_command(&OutputFormat::Json, RevisionCommands::Catalog).unwrap();
        assert_eq!(code, 0);
        let from_cli: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(
            from_cli,
            serde_json::to_value(
                eda_engine::api::native_write::revision::revision_public_catalog()
            )
            .unwrap()
        );

        let root =
            std::env::temp_dir().join(format!("datum_cli_revision_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Revision CLI fixture".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let (text_output, code) = execute_revision_command(
            &OutputFormat::Text,
            RevisionCommands::Query(RevisionQueryArgs {
                path: root.clone(),
                query: "configuration_current".into(),
                as_of_sequence: None,
                expected_model_revision: None,
            }),
        )
        .unwrap();
        assert_eq!(code, 0);
        assert!(text_output.contains("revision query: configuration_current"));
        assert!(text_output.contains("records: 0"));
        let _ = std::fs::remove_dir_all(root);
    }
}
