use crate::development_workspace::{checked_external, resolve, scratch_at};
use std::path::Path;

#[test]
fn standalone_replay_aliases_bind_historical_paths_without_local_maps() {
    use crate::development_workspace::replay_relative;
    let checkout = Path::new("/clean/export");
    for path in [
        "target/user-edit-selection/journal-read-project",
        "/home/bfadmin/Documents/datum-eda/target/user-edit-selection/journal-read-project",
        "/clean/export/target/user-edit-selection/journal-read-project",
    ] {
        assert_eq!(
            replay_relative(Path::new(path), checkout).unwrap().unwrap(),
            Path::new("user-edit-selection/journal-read-project")
        );
    }
    assert!(
        replay_relative(Path::new("target/user-edit-selection-other"), checkout)
            .unwrap()
            .is_none()
    );
    assert!(replay_relative(Path::new("target/user-edit-selection/../escape"), checkout).is_err());
}

#[test]
fn replay_completion_and_symlink_controls_protect_inputs() {
    use crate::development_workspace::replay_at;
    let tmp = std::env::temp_dir().join(format!("datum-replay-paths-{}", uuid::Uuid::new_v4()));
    let root = tmp.join("private");
    let checkout = tmp.join("checkout");
    std::fs::create_dir_all(&checkout).unwrap();
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("index.json"), b"{}").unwrap();
    assert!(replay_at(&root, Path::new("fixture"), &checkout).is_err());
    std::fs::write(
        root.join("index.json"),
        include_bytes!("../testdata/selection/s5a_v1/replay-inputs/index.json"),
    )
    .unwrap();
    std::fs::create_dir(root.join("fixture")).unwrap();
    assert!(replay_at(&root, Path::new("fixture"), &checkout).is_ok());
    assert!(replay_at(&root, Path::new("../checkout"), &checkout).is_err());
    assert!(replay_at(&checkout, Path::new("fixture"), &checkout).is_err());
    std::os::unix::fs::symlink(root.join("fixture"), root.join("alias")).unwrap();
    assert!(replay_at(&root, Path::new("alias"), &checkout).is_err());
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn relocated_pinned_paths_preserve_suffix_and_reject_retired_or_partial_inputs() {
    let checkout = Path::new("/original/checkout");
    let mut map = serde_json::json!({"schema_version":1,"collections":[{
        "source_relative":"target/replay", "destination":"/external/retained/replay",
        "state":"preserved", "disposition":"retained-fixture"
    }]});
    assert_eq!(
        resolve(Path::new("target/replay/project.json"), checkout, &map).unwrap(),
        Path::new("/external/retained/replay/project.json")
    );
    assert_eq!(
        resolve(
            Path::new("/original/checkout/target/replay/project.json"),
            checkout,
            &map
        )
        .unwrap(),
        Path::new("/external/retained/replay/project.json")
    );
    for path in [
        "target/replay-other/project.json",
        "target/replay/../escape",
        "/unrelated/target/replay",
    ] {
        assert!(resolve(Path::new(path), checkout, &map).is_err());
    }
    map["collections"][0]["state"] = "prepared".into();
    assert!(resolve(Path::new("target/replay"), checkout, &map).is_err());
    map["collections"][0]["state"] = "preserved".into();
    map["collections"][0]["disposition"] = "retired-released-test-output".into();
    assert!(resolve(Path::new("target/replay"), checkout, &map).is_err());
}

#[test]
fn external_workspace_and_relocation_cannot_alias_checkout_or_traverse() {
    let tmp = std::env::temp_dir().join(format!("datum-workspace-paths-{}", uuid::Uuid::new_v4()));
    let checkout = tmp.join("checkout");
    std::fs::create_dir_all(&checkout).unwrap();
    std::os::unix::fs::symlink(&checkout, tmp.join("alias")).unwrap();
    for path in [
        checkout.join("fixtures"),
        tmp.join("alias/fixtures"),
        tmp.join("outside/../checkout"),
    ] {
        assert!(checked_external(&path, &checkout).is_err());
    }
    assert!(checked_external(Path::new("relative"), &checkout).is_err());
    assert_eq!(
        checked_external(&tmp.join("outside/new"), &checkout).unwrap(),
        tmp.join("outside/new")
    );
    let mut map = serde_json::json!({"schema_version":1,"collections":[{
        "source_relative":"target/replay","destination":checkout.join("forbidden"),"state":"preserved"
    }]});
    assert!(resolve(Path::new("target/replay"), &checkout, &map).is_err());
    map["collections"][0]["source_relative"] = "target/../replay".into();
    assert!(resolve(Path::new("target/replay"), &checkout, &map).is_err());
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn scratch_child_aliases_cannot_redirect_new_output_into_checkout() {
    let tmp = std::env::temp_dir().join(format!("datum-scratch-alias-{}", uuid::Uuid::new_v4()));
    let checkout = tmp.join("checkout");
    let base = tmp.join("external");
    std::fs::create_dir_all(&checkout).unwrap();
    std::fs::create_dir(&base).unwrap();
    std::os::unix::fs::symlink(&checkout, base.join("scratch")).unwrap();
    assert!(scratch_at(&base, "family", &checkout).is_err());
    std::fs::remove_file(base.join("scratch")).unwrap();
    std::fs::create_dir(base.join("scratch")).unwrap();
    std::os::unix::fs::symlink(&checkout, base.join("scratch/family")).unwrap();
    assert!(scratch_at(&base, "family", &checkout).is_err());
    assert!(!checkout.join("family").exists());
    std::fs::remove_dir_all(tmp).unwrap();
}
