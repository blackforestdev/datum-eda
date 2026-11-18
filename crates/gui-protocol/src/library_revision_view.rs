//! Read-only common records qualified against the Library's resolved basis.
use super::{LibraryBrowser, LibraryEntry};
use eda_engine::substrate::DesignModel;
use serde_json::{Value, json};

#[path = "library_contribution_view.rs"]
mod contribution;

pub(super) fn matches_model(description: &Value, model: &DesignModel) -> bool {
    let library = &description["library"];
    description.get("project_id") == Some(&json!(model.project.project_id))
        && description.get("model_revision") == Some(&json!(model.model_revision))
        && description.get("transaction_tip")
            == Some(&json!(eda_engine::revision::transaction_tip(
                &model.journal
            )))
        && library.get("project_id") == description.get("project_id")
        && library.get("model_revision") == description.get("model_revision")
        && library.get("managed") == Some(&json!(model.project.schema_version == Some(2)))
        && library.get("available_assets").is_some_and(Value::is_array)
}

impl LibraryBrowser {
    pub(crate) fn current_revision_inspection(&self) -> Option<&Value> {
        self.revision_inspection.as_ref().filter(|description| {
            self.basis
                .as_ref()
                .is_some_and(|model| matches_model(description, model))
        })
    }

    pub(crate) fn common_revision_summary(&self, entry: &LibraryEntry) -> Vec<String> {
        let Some(description) = self.current_revision_inspection() else {
            return vec!["Common revision records: unavailable for this snapshot".into()];
        };
        let library = &description["library"];
        if library["managed"] == false {
            return vec!["Common revision history: not adopted; native metadata retained".into()];
        }
        let Some(kind) = record_kind(entry) else {
            return Vec::new();
        };
        let records: Vec<_> = library["available_assets"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| {
                record["revision"]["asset_id"] == json!(entry.id) && record["kind"] == kind
            })
            .collect();
        if records.iter().any(|record| {
            record["active"] == true && record["active_source_path"] != json!(entry.source_path)
        }) {
            return vec!["Common revision records: source owner differs; refresh required".into()];
        }
        let mut lines = Vec::new();
        // These are explicit active and catalog-root facts, never a guessed
        // newest revision or a change in asset search/ranking/adoption policy.
        for (label, active) in [("Active source", true), ("Saved candidate", false)] {
            for record in records.iter().filter(|record| {
                if active {
                    record["active"] == true
                } else {
                    record["active"] != true && record["catalog_root"] == true
                }
            }) {
                lines.push(format!("{label}: {}", entry.name));
                if entry.kind == "parts" && record["kind"] == "part" {
                    lines.extend(contribution::overview(record));
                } else {
                    lines.push("Common records: recorded; full details below".into());
                }
                lines.push(format!(
                    "{label} exact revision: {}",
                    record["revision"]["revision_id"]
                        .as_str()
                        .unwrap_or("unavailable")
                ));
            }
        }
        if lines.is_empty() {
            lines.push(
                if records.is_empty() {
                    "Common revision history: not recorded for this definition"
                } else {
                    "Common revision history: retained records; no active or catalog-root claim"
                }
                .into(),
            );
        }
        lines
    }

    /// Reject stale input without replacing a newer, still-qualified view.
    pub fn set_revision_inspection(&mut self, description: Value) -> Result<(), String> {
        let model = self.basis.as_ref().ok_or("Library source unavailable")?;
        if !matches_model(&description, model) {
            if self
                .revision_inspection
                .as_ref()
                .is_some_and(|d| !matches_model(d, model))
            {
                self.revision_inspection = None;
                self.revision_inspection_id = uuid::Uuid::new_v4();
            }
            return Err(
                "Project changed during Library inspection; refresh before reviewing revisions"
                    .into(),
            );
        }
        if self.revision_inspection_id.is_nil()
            || self.revision_inspection.as_ref() != Some(&description)
        {
            self.revision_inspection_id = uuid::Uuid::new_v4();
            self.revision_inspection = Some(description);
        }
        Ok(())
    }

    pub(super) fn common_revision_lines(&self, entry: &LibraryEntry) -> Vec<String> {
        let Some(description) = self.current_revision_inspection() else {
            return vec!["Common revision records: unavailable for this snapshot".into()];
        };
        let library = &description["library"];
        if library["managed"] == false {
            return vec!["Common revision history: not adopted; native metadata retained".into()];
        }
        let Some(kind) = record_kind(entry) else {
            return Vec::new();
        };
        let records: Vec<_> = library["available_assets"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["revision"]["asset_id"] == json!(entry.id) && r["kind"] == kind)
            .collect();
        if records.is_empty() {
            return vec!["Common revision history: not recorded for this definition".into()];
        }
        let mut lines = vec![
            format!(
                "Library catalog: {}",
                library["catalog_manifest"]
                    .as_str()
                    .unwrap_or("unavailable")
            ),
            "Publication and approval: not inferred from local records".into(),
        ];
        for record in records {
            let active = record["active"] == true;
            if active && record["active_source_path"] != json!(entry.source_path) {
                return vec![
                    "Common revision records: source owner differs; refresh required".into(),
                ];
            }
            lines.push(format!(
                "{} revision: {}",
                if active { "Active" } else { "Available local" },
                record["revision"]["revision_id"]
                    .as_str()
                    .unwrap_or("unavailable")
            ));
            lines.push(format!("Asset identity: {}", entry.id));
            for (label, field) in [
                ("Payload digest", "payload_ref"),
                ("Parent revisions", "parents"),
                ("Exact dependencies", "dependencies"),
                ("Common provenance", "provenance"),
            ] {
                flatten_detail(label, &record[field], &mut lines, "unknown");
            }
            if kind == "part" {
                lines.extend(contribution::lines(record));
            }
        }
        lines
    }
}

fn record_kind(entry: &LibraryEntry) -> Option<&'static str> {
    Some(match entry.kind.as_str() {
        "units" => "unit",
        "symbols" => "symbol",
        "entities" => "entity",
        "parts" => "part",
        "packages" => "package",
        "footprints" => "footprint",
        "padstacks" => "padstack",
        "pin_pad_maps" => "pin_pad_map",
        _ => return None,
    })
}

pub(super) fn flatten_detail(path: &str, value: &Value, lines: &mut Vec<String>, null_text: &str) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let name = value.get("name").and_then(Value::as_str).unwrap_or(key);
                let path = if path.is_empty() {
                    name.into()
                } else {
                    format!("{path}.{name}")
                };
                flatten_detail(&path, value, lines, null_text);
            }
        }
        Value::Array(values) => {
            if values.is_empty() {
                lines.push(format!("{path}: empty"));
            } else {
                for (i, value) in values.iter().enumerate() {
                    flatten_detail(&format!("{path}[{i}]"), value, lines, null_text);
                }
            }
        }
        _ => lines.push(format!(
            "{path}: {}",
            if value.is_null() {
                null_text.into()
            } else {
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string())
            }
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    use eda_engine::substrate::ProjectResolver;
    use std::sync::Arc;
    use uuid::Uuid;

    fn basis() -> (std::path::PathBuf, DesignModel) {
        let root = std::env::temp_dir().join(format!("datum-library-view-{}", Uuid::new_v4()));
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Library view control".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        (root, model)
    }

    fn description(model: &DesignModel, managed: bool) -> Value {
        json!({"project_id":model.project.project_id,"model_revision":model.model_revision,
            "transaction_tip":eda_engine::revision::transaction_tip(&model.journal),
            "library":{"project_id":model.project.project_id,"model_revision":model.model_revision,
                "managed":managed,"available_assets":[]}})
    }

    #[test]
    fn stale_inspection_clears_records_even_when_project_identity_is_unchanged() {
        let (root, model) = basis();
        let valid = description(&model, false);
        let mut browser = LibraryBrowser::capture(Arc::new(model));
        for (path, replacement) in [
            (vec!["project_id"], json!(Uuid::new_v4())),
            (vec!["model_revision"], json!("old")),
            (vec!["transaction_tip"], json!(Uuid::new_v4())),
            (vec!["library", "model_revision"], json!("old")),
            (vec!["library", "managed"], json!(true)),
        ] {
            browser.set_revision_inspection(valid.clone()).unwrap();
            let mut changed = valid.clone();
            let mut field = &mut changed;
            for key in path {
                field = &mut field[key];
            }
            *field = replacement;
            assert!(browser.set_revision_inspection(changed).is_err());
            assert_eq!(browser.revision_inspection.as_ref(), Some(&valid));
        }
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("transaction_tip");
        assert!(browser.set_revision_inspection(missing).is_err());
        let mut changed_basis = (**browser.basis.as_ref().unwrap()).clone();
        changed_basis.model_revision.0 = "new basis".into();
        browser.basis = Some(Arc::new(changed_basis));
        assert!(browser.set_revision_inspection(valid).is_err());
        assert!(browser.revision_inspection.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn revision_details_keep_candidate_and_unknown_provenance_distinct_from_active() {
        let (root, mut model) = basis();
        // Projection-only input. This does not adopt a profile or establish
        // canonical persistence, materialization, history or GUI proof.
        model.project.schema_version = Some(2);
        let mut view = description(&model, true);
        let id = Uuid::new_v4();
        let entry = LibraryEntry {
            id,
            kind: "units".into(),
            name: "View fixture".into(),
            source_path: format!("pool/units/{id}.json"),
            revision: Some(1),
            raw: json!({"name":"View fixture","nullable":null,"tags":[]}),
            warning: None,
        };
        let first = format!("sha256:{}", "1".repeat(64));
        let second = format!("sha256:{}", "2".repeat(64));
        view["library"]["catalog_manifest"] = json!(first);
        view["library"]["available_assets"] = json!([
            {"revision":{"asset_id":id,"revision_id":first},"kind":"unit","active":true,
                "active_source_path":entry.source_path,"payload_ref":first,"parents":[],"dependencies":[],
                "provenance":{"authors":[],"created_at":null,"rights":{"license_expression":null}}},
            {"revision":{"asset_id":id,"revision_id":second},"kind":"unit","active":false,
                "active_source_path":null,"payload_ref":second,"parents":[first],"dependencies":[],
                "provenance":{"authors":[],"created_at":null,"change_summary":"Reviewed candidate"}}
        ]);
        let mut browser = LibraryBrowser::capture(Arc::new(model));
        browser.set_revision_inspection(view.clone()).unwrap();
        let lines = browser.detail_lines(&entry);
        assert!(
            lines
                .iter()
                .any(|s| s == &format!("Active revision: {first}"))
        );
        assert!(
            lines
                .iter()
                .any(|s| s == &format!("Available local revision: {second}"))
        );
        assert!(
            lines
                .iter()
                .any(|s| s == "Common provenance.created_at: unknown")
        );
        assert!(
            lines
                .iter()
                .any(|s| s == "Common provenance.rights.license_expression: unknown")
        );
        assert!(lines.iter().any(|s| s == "nullable: null"));
        assert!(lines.iter().any(|s| s == "tags: empty"));
        view["library"]["available_assets"][0]["active_source_path"] = json!("other/owner.json");
        browser.set_revision_inspection(view).unwrap();
        assert!(
            browser
                .detail_lines(&entry)
                .iter()
                .any(|s| s.contains("source owner differs"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
