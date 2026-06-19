use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore = "Layer B requires a pinned headless display environment; run manually under xvfb/visual CI"]
fn layer_b_shell_screenshot_first_slice() {
    let output_path =
        std::env::temp_dir().join(format!("datum-layer-b-shell-{}.png", std::process::id()));
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/engine/testdata/golden/text/native/text-fidelity-repro");
    let status = Command::new(env!("CARGO_BIN_EXE_datum-gui"))
        .arg("--project-root")
        .arg(project_root)
        .arg("--visual-test")
        .arg("--window-size")
        .arg("1280x768")
        .arg("--screenshot-out")
        .arg(&output_path)
        .arg("--exit-after-screenshot")
        .status()
        .expect("launch datum-gui visual shell fixture");

    assert!(status.success(), "datum-gui visual shell launch failed");
    let metadata = std::fs::metadata(&output_path).expect("visual shell screenshot should exist");
    assert!(
        metadata.len() > 0,
        "visual shell screenshot should be non-empty"
    );
    let _ = std::fs::remove_file(output_path);
}
