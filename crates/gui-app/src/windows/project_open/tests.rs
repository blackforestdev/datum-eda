#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;

#[test]
fn late_answers_cannot_cross_copied_root_generation_cancel_or_busy_write() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let base =
        development_workspace::path("open-project-app-tests").join(std::process::id().to_string());
    let roots = [
        base.join("origin"),
        base.join("identical copied destination"),
    ];
    for root in &roots {
        std::fs::create_dir_all(root).unwrap();
        std::fs::copy(fixture.join("project.json"), root.join("project.json")).unwrap();
    }
    assert_eq!(
        std::fs::read(roots[0].join("project.json")).unwrap(),
        std::fs::read(roots[1].join("project.json")).unwrap()
    );
    let origin = ProjectOpenOrigin::capture(&roots[0], 3).unwrap();
    assert!(validate_publication_origin(&origin, &roots[0], 3, 7, 7, true, false).is_ok());
    assert!(validate_publication_origin(&origin, &roots[1], 3, 7, 7, true, false).is_err());
    assert!(validate_publication_origin(&origin, &roots[0], 4, 7, 7, true, false).is_err());
    assert!(validate_publication_origin(&origin, &roots[0], 3, 7, 8, true, false).is_err());
    assert!(validate_publication_origin(&origin, &roots[0], 3, 7, 7, false, false).is_err());
    assert!(validate_publication_origin(&origin, &roots[0], 3, 7, 7, true, true).is_err());
    assert_eq!(
        std::fs::read(roots[0].join("project.json")).unwrap(),
        std::fs::read(roots[1].join("project.json")).unwrap()
    );
}
