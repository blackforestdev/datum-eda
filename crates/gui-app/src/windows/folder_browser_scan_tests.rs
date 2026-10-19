use super::*;
use std::sync::atomic::AtomicUsize;

#[test]
fn scan_is_read_only_includes_dotfolders_links_and_collision_names() {
    let root = Scratch::new();
    std::fs::create_dir(root.path().join("z")).unwrap();
    std::fs::create_dir(root.path().join("Alpha")).unwrap();
    std::fs::create_dir(root.path().join(".hidden")).unwrap();
    std::fs::write(root.path().join("occupied-file"), b"unchanged").unwrap();
    std::os::unix::fs::symlink(root.path().join("Alpha"), root.path().join("link")).unwrap();
    std::os::unix::fs::symlink(root.path().join("missing"), root.path().join("dangling")).unwrap();
    use std::os::unix::ffi::OsStrExt;
    std::fs::create_dir(root.path().join(std::ffi::OsStr::from_bytes(b"bad-\xff"))).unwrap();
    let listing = scan(root.path(), || true).unwrap();
    assert_eq!(
        listing
            .entries
            .iter()
            .filter(|e| e.navigable())
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        [".hidden", "Alpha", "link", "z"]
    );
    assert!(listing.occupied.contains("occupied-file"));
    assert!(listing.occupied.contains("dangling"));
    assert!(listing.notice.is_some());
    assert_eq!(
        std::fs::read(root.path().join("occupied-file")).unwrap(),
        b"unchanged"
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 7);
    assert!(scan(&root.path().join("occupied-file"), || true).is_err());
    assert!(scan(&root.path().join("absent"), || true).is_err());
}

#[test]
fn scan_cancels_and_worker_delivers_latest_navigation() {
    let root = Scratch::new();
    for i in 0..1500 {
        std::fs::create_dir(root.path().join(format!("folder-{i}"))).unwrap();
    }
    let checked = AtomicUsize::new(0);
    assert!(scan(root.path(), || checked.fetch_add(1, Ordering::Relaxed) < 3).is_err());
    let scanner = Scanner::new(|| {}).unwrap();
    let old = scanner.request(root.path().into()).unwrap();
    scanner.cancel();
    let latest = scanner.request(root.path().join("folder-1499")).unwrap();
    assert_ne!(old, latest);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(answer) = scanner.receive() {
            if answer.ticket == latest {
                assert!(answer.result.unwrap().entries.is_empty());
                break;
            }
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn scan_reports_unreadable_and_disappearing_folder_without_writes() {
    use std::os::unix::fs::PermissionsExt;
    let root = Scratch::new();
    let private = root.path().join("private");
    std::fs::create_dir(&private).unwrap();
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0)).unwrap();
    let result = scan(&private, || true);
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o700)).unwrap();
    if unsafe { libc::geteuid() } != 0 {
        assert!(result.unwrap_err().contains("Permission denied"));
    }
    std::fs::remove_dir(&private).unwrap();
    assert!(scan(&private, || true).is_err());
    assert!(!private.exists());
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.datum/native-project-startup-repair/c09-tests")
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn path_assistance_worker_matches_literal_substrings_keeps_occupancy_and_distinguishes_explicit_errors()
 {
    let root = Scratch::new();
    for leaf in [
        "Documents",
        "Downloads",
        "kicad_projects",
        ".dot",
        "space λ ",
    ] {
        std::fs::create_dir(root.path().join(leaf)).unwrap();
    }
    std::fs::write(root.path().join("ordinary.txt"), b"original").unwrap();
    let listing = query::discover(&root.path().join("d"), None, true, || true).unwrap();
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["Documents", "Downloads", ".dot", "kicad_projects"]
    );
    assert!(listing.occupied.contains("ordinary.txt"));
    assert!(listing.parent_identity.is_some() && listing.suggestions.is_some());
    assert!(query::discover(&root.path().join("d"), None, false, || true).is_err());
    let no_match = query::discover(&root.path().join("not-found"), None, true, || true).unwrap();
    assert!(no_match.entries.is_empty() && no_match.suggestions.is_some());
    assert!(
        query::discover(&root.path().join("Documents"), None, true, || true)
            .unwrap()
            .suggestions
            .is_some()
    );
    for raw in ["/", "~", ".", "..", "folder/.", "folder/..", "folder/"] {
        assert!(
            query::discover(root.path(), Some(raw), true, || true)
                .unwrap()
                .suggestions
                .is_none(),
            "navigation anchor {raw}"
        );
    }
    let scanner = Scanner::new(|| {}).unwrap();
    let old = scanner
        .request_input(root.path().join("Doc"), Some("~/Doc".into()), true)
        .unwrap();
    let last = scanner
        .request_input(root.path().join("Down"), Some("~/Down".into()), true)
        .unwrap();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(answer) = scanner.receive() {
            if answer.ticket == last {
                assert_ne!(answer.ticket, old);
                assert_eq!(answer.input.as_deref(), Some("~/Down"));
                assert_eq!(answer.result.unwrap().entries[0].name, "Downloads");
                break;
            }
        }
        assert!(std::time::Instant::now() < until);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        std::fs::read(root.path().join("ordinary.txt")).unwrap(),
        b"original"
    );
}
