use super::genesis::{GenesisSpec, bootstrap_native_project};
use super::project::project_units_seed_evidence;
use super::test_support::temp_project_root;
use crate::ir::units::{PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units};
use crate::substrate::ProjectResolver;

#[test]
fn v1_units_receipt_projects_remain_readable_without_receipt_rewrite() {
    let root = temp_project_root("project_units_v1_receipt");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "V1 Units Receipt Fixture".to_owned(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let PreFeatureProjectUnitsMigration::Create { receipt, .. } =
        migrate_pre_feature_project_units(None).unwrap()
    else {
        unreachable!()
    };
    let exact_v1 = serde_json::to_value(&receipt).unwrap();
    model.project.project_units_seed_receipt = Some(exact_v1.clone());

    let first = project_units_seed_evidence(&model).unwrap();
    let second = project_units_seed_evidence(&model).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.copied_values, receipt.copied_values);
    assert!(first.source_summary.contains("FactoryMigrationV1"));
    assert_eq!(model.project.project_units_seed_receipt, Some(exact_v1));
    let _ = std::fs::remove_dir_all(&root);
}
