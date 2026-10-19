use super::*;
use std::time::{Duration, Instant};

pub(super) struct Scratch(pub(super) PathBuf);
impl Scratch {
    pub(super) fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qa/host-filesystem-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
    pub(super) fn request(&self, command: Command) -> Request {
        Request {
            scope: 7,
            parent: self.0.clone(),
            parent_identity: Identity::of(&fs::metadata(&self.0).unwrap()),
            command,
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn receipt(service: &mut Service) -> Completion {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(receipt) = service.receive() {
            return receipt;
        }
        assert!(
            Instant::now() < deadline,
            "actual folder worker must complete"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn invoke(service: &mut Service, request: Request) -> Outcome {
    service.submit(request).unwrap();
    receipt(service).outcome
}
fn assert_refused(outcome: Outcome) -> String {
    match outcome {
        Outcome::Refused(reason) => reason,
        other => panic!("expected refusal, got {other:?}"),
    }
}
fn rename(root: &Scratch, old: &str, new: &str) -> Request {
    root.request(Command::Rename {
        old_leaf: old.into(),
        new_leaf: new.into(),
        source: Identity::of(&fs::symlink_metadata(root.0.join(old)).unwrap()),
    })
}
#[test]
fn host_filesystem_actual_worker_creates_and_renames_file_folder_without_overwrite() {
    let root = Scratch::new();
    let mut service = Service::new(|| {}).unwrap();
    let request = root.request(Command::CreateDirectory {
        leaf: " exact λ ".into(),
    });
    let ticket = service.submit(request.clone()).unwrap();
    assert!(
        service.submit(request).is_err(),
        "duplicate submission blocked until receipt consumed"
    );
    let completed = receipt(&mut service);
    assert_eq!(completed.ticket, ticket);
    assert_eq!(completed.scope, 7);
    assert!(matches!(completed.outcome, Outcome::Committed(_)));
    assert!(root.0.join(" exact λ ").is_dir());
    assert!(matches!(
        invoke(&mut service, rename(&root, " exact λ ", "renamed folder")),
        Outcome::Committed(_)
    ));
    fs::write(root.0.join("note.txt"), b"unaltered bytes").unwrap();
    let outcome = invoke(&mut service, rename(&root, "note.txt", " note λ.txt "));
    assert!(matches!(outcome, Outcome::Committed(_)));
    assert_eq!(
        fs::read(root.0.join(" note λ.txt ")).unwrap(),
        b"unaltered bytes"
    );
    fs::create_dir(root.0.join("occupied")).unwrap();
    assert_refused(invoke(
        &mut service,
        rename(&root, "renamed folder", "occupied"),
    ));
    assert!(root.0.join("renamed folder").is_dir() && root.0.join("occupied").is_dir());
    assert_refused(invoke(
        &mut service,
        root.request(Command::CreateDirectory {
            leaf: "occupied".into(),
        }),
    ));
    assert!(matches!(
        invoke(&mut service, rename(&root, " note λ.txt ", " note λ.txt ")),
        Outcome::Unchanged(_)
    ));
    fs::write(root.0.join("collision.txt"), b"other").unwrap();
    assert_refused(invoke(
        &mut service,
        rename(&root, " note λ.txt ", "collision.txt"),
    ));
    assert_eq!(fs::read(root.0.join("collision.txt")).unwrap(), b"other");
}
#[test]
fn host_filesystem_refuses_stale_parent_source_links_and_invalid_names() {
    let root = Scratch::new();
    let mut service = Service::new(|| {}).unwrap();
    for leaf in ["", ".", "..", "a/b", "a\0b", ".datum", "project.json"] {
        assert_refused(invoke(
            &mut service,
            root.request(Command::CreateDirectory { leaf: leaf.into() }),
        ));
    }
    fs::write(root.0.join("file"), b"original").unwrap();
    let request = rename(&root, "file", "new");
    fs::rename(root.0.join("file"), root.0.join("retired")).unwrap();
    fs::write(root.0.join("file"), b"replacement").unwrap();
    assert_refused(invoke(&mut service, request));
    assert!(!root.0.join("new").exists());
    std::os::unix::fs::symlink(root.0.join("file"), root.0.join("link")).unwrap();
    assert_refused(invoke(&mut service, rename(&root, "link", "link-new")));
    let mut request = root.request(Command::CreateDirectory { leaf: "new".into() });
    request.parent_identity.inode ^= 1;
    assert_refused(invoke(&mut service, request));
    assert_eq!(fs::read(root.0.join("file")).unwrap(), b"replacement");
}
#[test]
fn host_filesystem_protects_projects_namespaces_and_session_ancestors_but_allows_cwd_mkdir() {
    let root = Scratch::new();
    let mut service = Service::new(|| {}).unwrap();
    service.set_protection(Protection {
        managed: vec![],
        anchors: vec![root.0.clone()],
    });
    assert!(matches!(
        invoke(
            &mut service,
            root.request(Command::CreateDirectory {
                leaf: "host child".into()
            })
        ),
        Outcome::Committed(_)
    ));
    let inner = root.0.join("container");
    fs::create_dir_all(inner.join("nested Project")).unwrap();
    fs::write(
        inner.join("nested Project/project.json"),
        b"even malformed is a safety marker",
    )
    .unwrap();
    assert!(
        assert_refused(invoke(
            &mut service,
            rename(&root, "container", "relocated")
        ))
        .contains("Project")
    );
    let managed = root.0.join("managed");
    fs::create_dir(&managed).unwrap();
    service.set_protection(Protection {
        managed: vec![managed.clone()],
        anchors: vec![root.0.join("host child")],
    });
    assert_refused(invoke(
        &mut service,
        rename(&root, "host child", "moved cwd"),
    ));
    let request = Request {
        scope: 1,
        parent: managed.clone(),
        parent_identity: Identity::of(&fs::metadata(&managed).unwrap()),
        command: Command::CreateDirectory {
            leaf: "not allowed".into(),
        },
    };
    assert_refused(invoke(&mut service, request));
    assert!(!managed.join("not allowed").exists());
    fs::write(root.0.join("project.json"), b"marker").unwrap();
    assert_refused(invoke(
        &mut service,
        root.request(Command::CreateDirectory {
            leaf: "project internal".into(),
        }),
    ));
}
#[test]
fn host_filesystem_retains_completion_after_callers_scope_retires() {
    let root = Scratch::new();
    let mut service = Service::new(|| {}).unwrap();
    let ticket = service
        .submit(root.request(Command::CreateDirectory {
            leaf: "persistent".into(),
        }))
        .unwrap();
    // No browser state/lifetime is retained by the worker or its completion.
    let completed = receipt(&mut service);
    assert_eq!((completed.ticket, completed.scope), (ticket, 7));
    assert!(matches!(completed.outcome, Outcome::Committed(_)));
    assert!(root.0.join("persistent").is_dir());
}
#[test]
fn host_filesystem_subtree_change_unreadable_and_stale_protection_refuse() {
    use std::os::unix::fs::PermissionsExt;
    let root = Scratch::new();
    fs::create_dir(root.0.join("folder")).unwrap();
    let request = rename(&root, "folder", "changed");
    let parent = fs::File::open(&root.0).unwrap();
    let inspection =
        safety::inspect(&parent, &root.0, &request.command, &Protection::default()).unwrap();
    fs::write(root.0.join("folder/new file"), b"changed subtree").unwrap();
    assert!(inspection.recheck().is_err());
    fs::set_permissions(root.0.join("folder"), fs::Permissions::from_mode(0)).unwrap();
    if unsafe { libc::geteuid() } != 0 {
        assert!(
            safety::inspect(&parent, &root.0, &request.command, &Protection::default()).is_err()
        );
    }
    fs::set_permissions(root.0.join("folder"), fs::Permissions::from_mode(0o700)).unwrap();
    let epoch = Arc::new(AtomicU64::new(2));
    let job = Job {
        ticket: 1,
        request,
        protection: Protection::default(),
        epoch,
        generation: 1,
    };
    assert!(assert_refused(operations::execute(job)).contains("safety context changed"));
    assert!(!root.0.join("changed").exists());
}

#[test]
fn host_filesystem_allows_ordinary_project_notes_but_guards_actual_storage_namespaces() {
    let root = Scratch::new();
    let project = Scratch(root.0.join("Project"));
    for directory in ["board", "schematic", "rules"] {
        fs::create_dir_all(project.0.join(directory)).unwrap();
        fs::write(
            project.0.join(directory).join(format!("{directory}.json")),
            b"{}",
        )
        .unwrap();
    }
    fs::write(project.0.join("project.json"), br#"{"board":"board/board.json","schematic":"schematic/schematic.json","rules":"rules/rules.json","pools":[]}"#).unwrap();
    fs::write(project.0.join("notes.txt"), b"ordinary host bytes").unwrap();
    let mut service = Service::new(|| {}).unwrap();
    service.set_protection(Protection {
        managed: vec![root.0.join("missing unrelated backing")],
        anchors: vec![project.0.clone()],
    });
    assert!(matches!(
        invoke(
            &mut service,
            rename(&project, "notes.txt", "renamed notes.txt")
        ),
        Outcome::Committed(_)
    ));
    assert_eq!(
        fs::read(project.0.join("renamed notes.txt")).unwrap(),
        b"ordinary host bytes"
    );
    assert!(matches!(
        invoke(
            &mut service,
            project.request(Command::CreateDirectory {
                leaf: "ordinary folder".into()
            })
        ),
        Outcome::Committed(_)
    ));
    assert_refused(invoke(
        &mut service,
        rename(&project, "board", "moved board"),
    ));
    let board = Scratch(project.0.join("board"));
    assert_refused(invoke(
        &mut service,
        board.request(Command::CreateDirectory {
            leaf: "new host folder".into(),
        }),
    ));
    assert!(project.0.join("board/board.json").is_file());
}

#[test]
fn host_filesystem_missing_protected_destinations_are_not_claimed_by_host_actions() {
    let root = Scratch::new();
    let mut service = Service::new(|| {}).unwrap();
    fs::write(root.0.join("file"), b"preserve").unwrap();
    service.set_protection(Protection {
        managed: vec![root.0.join("pending backing")],
        anchors: vec![root.0.join("pending cwd/child")],
    });
    assert_refused(invoke(
        &mut service,
        rename(&root, "file", "pending backing"),
    ));
    assert_refused(invoke(&mut service, rename(&root, "file", "pending cwd")));
    assert_refused(invoke(
        &mut service,
        root.request(Command::CreateDirectory {
            leaf: "pending backing".into(),
        }),
    ));
    assert!(matches!(
        invoke(
            &mut service,
            root.request(Command::CreateDirectory {
                leaf: "ordinary".into()
            })
        ),
        Outcome::Committed(_)
    ));
    assert!(
        matches!(
            invoke(
                &mut service,
                root.request(Command::CreateDirectory {
                    leaf: "pending cwd".into()
                })
            ),
            Outcome::Committed(_)
        ),
        "cwd relocation guard is Rename-only"
    );
    assert_eq!(fs::read(root.0.join("file")).unwrap(), b"preserve");
}

#[test]
fn host_filesystem_global_revision_marker_does_not_turn_host_ancestor_into_project() {
    let root = Scratch::new();
    fs::create_dir_all(root.0.join(".datum/revision")).unwrap();
    fs::write(
        root.0.join(".datum/revision/writer.lock"),
        b"unchanged coordination marker",
    )
    .unwrap();
    let ordinary = Scratch(root.0.join("ordinary"));
    fs::create_dir(&ordinary.0).unwrap();
    fs::write(ordinary.0.join("note.txt"), b"unchanged note bytes").unwrap();
    let mut service = Service::new(|| {}).unwrap();
    assert!(matches!(
        invoke(
            &mut service,
            ordinary.request(Command::CreateDirectory {
                leaf: "new folder".into()
            })
        ),
        Outcome::Committed(_)
    ));
    assert!(matches!(
        invoke(&mut service, rename(&ordinary, "note.txt", "renamed.txt")),
        Outcome::Committed(_)
    ));
    assert_refused(invoke(
        &mut service,
        rename(&root, ".datum", "moved metadata"),
    ));
    let protected = Scratch(root.0.join(".datum/revision"));
    assert_refused(invoke(
        &mut service,
        protected.request(Command::CreateDirectory { leaf: "bad".into() }),
    ));
    assert_eq!(
        fs::read(protected.0.join("writer.lock")).unwrap(),
        b"unchanged coordination marker"
    );
}

#[test]
fn host_filesystem_project_settings_and_identity_bridge_are_protected_not_global_names() {
    let root = Scratch::new();
    for directory in ["board", "schematic", "rules", "settings", ".ids"] {
        fs::create_dir(root.0.join(directory)).unwrap();
        fs::write(root.0.join(directory).join("data.json"), b"retained bytes").unwrap();
    }
    fs::write(root.0.join("project.json"), br#"{"board":"board/data.json","schematic":"schematic/data.json","rules":"rules/data.json","pools":[],"settings":{"checking":"settings/data.json","output":"settings/future-output.json"}}"#).unwrap();
    let mut service = Service::new(|| {}).unwrap();
    let parents = [
        Scratch(root.0.join("settings")),
        Scratch(root.0.join(".ids")),
    ];
    for parent in &parents {
        assert_refused(invoke(
            &mut service,
            rename(&parent, "data.json", "changed.json"),
        ));
        assert_refused(invoke(
            &mut service,
            parent.request(Command::CreateDirectory {
                leaf: "new child".into(),
            }),
        ));
        assert_eq!(
            fs::read(parent.0.join("data.json")).unwrap(),
            b"retained bytes"
        );
    }
    fs::write(root.0.join("ordinary.txt"), b"outside protected stores").unwrap();
    assert!(matches!(
        invoke(&mut service, rename(&root, "ordinary.txt", "renamed.txt")),
        Outcome::Committed(_)
    ));
    let ordinary = Scratch::new();
    for directory in ["settings", ".ids", "pool", "schema", "revision"] {
        assert!(matches!(
            invoke(
                &mut service,
                ordinary.request(Command::CreateDirectory {
                    leaf: directory.into()
                })
            ),
            Outcome::Committed(_)
        ));
    }
}

#[test]
fn host_filesystem_readonly_availability_uses_real_safety_without_publishing() {
    let root = Scratch::new();
    fs::create_dir(root.0.join("ordinary")).unwrap();
    fs::create_dir(root.0.join("container")).unwrap();
    fs::create_dir(root.0.join("container/nested")).unwrap();
    fs::write(
        root.0.join("container/nested/project.json"),
        b"classification marker",
    )
    .unwrap();
    fs::write(root.0.join("note.txt"), b"retained exact bytes").unwrap();
    let protection = Protection::default();
    for name in ["ordinary", "note.txt"] {
        assert!(operations::availability(&rename(&root, name, name), &protection).is_ok());
    }
    assert!(
        operations::availability(&rename(&root, "container", "container"), &protection).is_err()
    );
    let parent = root.request(Command::CreateDirectory {
        leaf: String::new(),
    });
    assert!(operations::availability(&parent, &protection).is_ok());
    assert!(
        operations::availability(
            &parent,
            &Protection {
                managed: vec![root.0.clone()],
                anchors: vec![]
            }
        )
        .is_err()
    );
    let inspector = Inspector::new(|| {}).unwrap();
    let old = inspector
        .request(
            parent.clone(),
            Some(rename(&root, "note.txt", "note.txt").command),
            protection.clone(),
        )
        .unwrap();
    let current = inspector
        .request(
            parent,
            Some(rename(&root, "container", "container").command),
            protection,
        )
        .unwrap();
    assert_ne!(old, current);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(answer) = inspector.receive()
            && answer.ticket == current
        {
            assert!(answer.create.is_ok());
            assert!(answer.rename.unwrap().is_err());
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        fs::read(root.0.join("note.txt")).unwrap(),
        b"retained exact bytes"
    );
    assert!(
        root.0.join("ordinary").is_dir() && root.0.join("container/nested/project.json").is_file()
    );
    assert_eq!(
        fs::read_dir(&root.0).unwrap().count(),
        3,
        "availability writes nothing"
    );
}
