use super::*;

fn eagle_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle")
        .join(name)
}

fn kicad_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad")
        .join(name)
}

#[path = "main_tests_import_plan.rs"]
mod main_tests_import_plan;
#[path = "main_tests_plan_apply.rs"]
mod main_tests_plan_apply;
#[path = "main_tests_query_surface.rs"]
mod main_tests_query_surface;
#[path = "main_tests_check.rs"]
mod main_tests_check;
#[path = "main_tests_modify_basic.rs"]
mod main_tests_modify_basic;
#[path = "main_tests_modify_advanced.rs"]
mod main_tests_modify_advanced;
