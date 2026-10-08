use super::*;
use crate::revision::ProjectWriteLease;
use std::{
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    process::{Command, Stdio},
};

fn rename(model: &DesignModel, name: &str) -> OperationBatch {
    OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "writer-test".into(),
            source: CommitSource::Test,
            reason: name.into(),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: name.into(),
        }],
    }
}

fn child(root: &Path, mode: &str) -> std::process::Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "substrate::tests::writer_ownership::child_writer_process",
            "--ignored",
            "--nocapture",
        ])
        .env("DATUM_WRITER_TEST_ROOT", root)
        .env("DATUM_WRITER_TEST_MODE", mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
#[ignore = "subprocess entry, invoked by the ownership tests"]
fn child_writer_process() {
    let Some(root) = std::env::var_os("DATUM_WRITER_TEST_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    if std::env::var("DATUM_WRITER_TEST_MODE").unwrap() == "commit" {
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        model
            .commit_journaled(&root, rename(&model, "other-process"))
            .unwrap();
    } else {
        let _guard = ProjectWriteLease::acquire(&root).unwrap();
        std::fs::write(root.join("child-ready"), b"ready").unwrap();
        std::io::stdout().flush().unwrap();
        let mut byte = [0];
        let _ = std::io::stdin().read(&mut byte);
    }
}

#[test]
fn stale_prepared_intent_cannot_overwrite_another_process() {
    let root = temp_project_root("writer_stale_process");
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let mut old = ProjectResolver::new(&root).resolve().unwrap();
    let batch = rename(&old, "stale-overwrite");
    let old_revision = old.model_revision.clone();
    let mut writer = child(&root, "commit");
    assert!(writer.wait().unwrap().success());
    let before = std::fs::read(root.join("project.json")).unwrap();
    let journal = std::fs::read(transaction_journal_path(&root)).unwrap();
    let error = old.commit_journaled(&root, batch).unwrap_err();
    assert!(error.to_string().contains("persisted Project changed"));
    assert_eq!(old.model_revision, old_revision);
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), before);
    assert_eq!(
        std::fs::read(transaction_journal_path(&root)).unwrap(),
        journal
    );
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().project.name,
        "other-process"
    );
}

#[test]
fn process_death_releases_guard_without_changing_coordination_inode() {
    let root = temp_project_root("writer_process_death");
    let mut writer = child(&root, "hold");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !root.join("child-ready").exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "child did not acquire"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(ProjectWriteLease::acquire(&root).is_err());
    let inode = std::fs::metadata(root.join(".datum/revision/writer.lock"))
        .unwrap()
        .ino();
    writer.kill().unwrap();
    writer.wait().unwrap();
    let guard = ProjectWriteLease::acquire(&root).unwrap();
    assert_eq!(
        std::fs::metadata(root.join(".datum/revision/writer.lock"))
            .unwrap()
            .ino(),
        inode
    );
    drop(guard);
    assert!(root.join(".datum/revision/writer.lock").exists());
}

#[test]
fn aliases_contend_and_unsafe_or_legacy_boundaries_refuse_without_reaping() {
    let root = temp_project_root("writer_boundary");
    let alias = root.with_extension("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    let guard = ProjectWriteLease::acquire(&root).unwrap();
    assert!(ProjectWriteLease::acquire(&alias).is_err());
    drop(guard);
    let legacy = root.join(".datum/revision/write.lock");
    std::fs::write(&legacy, b"malformed legacy evidence").unwrap();
    assert!(ProjectWriteLease::acquire(&root).is_err());
    assert_eq!(
        std::fs::read(&legacy).unwrap(),
        b"malformed legacy evidence"
    );
    std::fs::remove_file(&legacy).unwrap(); // explicit test cleanup only
    let other = temp_project_root("writer_symlink");
    std::fs::create_dir_all(other.join(".datum/revision")).unwrap();
    std::os::unix::fs::symlink(
        root.join(".datum/revision/writer.lock"),
        other.join(".datum/revision/writer.lock"),
    )
    .unwrap();
    assert!(ProjectWriteLease::acquire(&other).is_err());
}
