//! Same production-renderer comparator as the accepted crossprobe goldens.
//! Explicit admitted input only; ordinary native events and HUMAN review are separate.
#![cfg(feature = "visual")]

use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject as Subject,
    SelectionTarget,
    inspector_context::{InspectorActivity, InspectorOwner, InspectorSectionId as Section},
    selection_artifacts::build_workspace_selection_view,
    selection_resolution::NativeSelectionResolution,
};
use std::path::PathBuf;

#[test]
#[ignore = "requires admitted Main input and independently reviewed Inspector golden candidates"]
fn admitted_inspector_same_engine_goldens() -> anyhow::Result<()> {
    let root = PathBuf::from(std::env::var("DATUM_INSPECTOR_TEST_PROJECT")?);
    let output = PathBuf::from(std::env::var("DATUM_INSPECTOR_VISUAL_OUTPUT")?);
    let candidate = std::env::var("DATUM_INSPECTOR_CANDIDATE_ONLY").as_deref() == Ok("1");
    std::fs::create_dir_all(&output)?;
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let (mut base, model) = datum_gui_protocol::load_board_editor_workspace_with_model(&request)?;
    anyhow::ensure!(
        model.model_revision.0
            == "e165c7950ddbcb0d5ac1329aae2672382bab32130536be1d3d98c125eaa88fd3",
        "admitted Main revision changed"
    );
    let source_before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| std::fs::read(root.join(&s.path)))
        .collect::<Result<_, _>>()?;
    // The normal context summary may deliberately cap findings. Use the same
    // complete persisted-record contract as the native producer readback.
    let cli = std::env::var("DATUM_INSPECTOR_CLI")?;
    let record = std::process::Command::new(cli)
        .args(["--format", "json", "check", "show"])
        .arg(&root)
        .args(["--check-run", "4b0604c3-ec98-524e-8ae7-ebcc7a7f2102"])
        .output()?;
    anyhow::ensure!(
        record.status.success(),
        "read-only persisted CheckRun query failed"
    );
    base.checks =
        datum_gui_protocol::check_run_review_state_from_json(std::str::from_utf8(&record.stdout)?)?;
    anyhow::ensure!(
        base.checks.persisted
            && base.checks.project_id.as_deref()
                == Some(model.project.project_id.to_string().as_str())
            && base.checks.model_revision.as_deref() == Some(model.model_revision.0.as_str())
            && base.checks.findings.len() == base.checks.finding_count
            && base.checks.finding_count == 13,
        "complete admitted persisted finding basis required"
    );
    base.inspector_context.activity = Some(InspectorActivity::Quiet);
    let authority = NativeSelectionResolution::capture(&model)
        .map_err(|e| anyhow::anyhow!("selection authority: {e:?}"))?;
    let id = |class, suffix: u64| -> anyhow::Result<Id> {
        Ok(Id {
            class,
            id: format!("78000000-0000-4000-8000-{suffix:012x}").parse()?,
            instance_path: vec![],
        })
    };
    let subject = |suffix, class| id(class, suffix).map(Subject::Object);
    let net = Subject::GlobalNet("b8c64de5-ffbe-42c0-ab16-b613c88b16da".parse()?);
    let diagnostic = Subject::Diagnostic(
        "sha256:e6126fbe2f51a05b6b5d129eabc79191255364134330bc4b587da9889fa99090".into(),
    );
    let mut cases = Vec::new();
    for (name, subject) in [
        ("symbol-library", subject(1, Class::Symbol)?),
        ("footprint-source", subject(2, Class::Footprint)?),
        ("net-scoped-zero", net.clone()),
        ("net-disclosures", net),
        ("diagnostic-pin", diagnostic),
    ] {
        let mut state = base.clone();
        let view = build_workspace_selection_view(&model, &authority, subject, &state)
            .map_err(|e| anyhow::anyhow!("{name}: {e:?}"))?;
        match name {
            "symbol-library" | "footprint-source" => {
                assert_eq!(view.members.len(), 1);
                // A focused Library section with the other named sections
                // collapsed is the same read-only disclosure state as the UI.
                state.inspector_context.collapsed.extend([
                    Section::Component,
                    Section::Placement,
                    Section::Properties,
                    Section::Connectivity,
                    Section::Checks,
                ]);
                state.inspector_context.focused = Some(Section::LibraryInfo);
            }
            "net-scoped-zero" | "net-disclosures" => {
                assert_eq!(view.members.len(), 6);
                state
                    .inspector_context
                    .collapsed
                    .extend([Section::Properties, Section::Connectivity]);
                state.ui.selection_inspector.scroll_offset = usize::MAX;
                if name == "net-disclosures" {
                    state.ui.selection_inspector.inventory_expanded = true;
                    state.ui.selection_inspector.filters_expanded = true;
                }
            }
            "diagnostic-pin" => {
                assert!(view.members.is_empty());
                assert!(view.related.is_empty());
                let datum_gui_protocol::selection_view::InspectionValue::Common(marker) =
                    view.field("schematic_marker")
                else {
                    anyhow::bail!("current qualified Pin marker unavailable");
                };
                assert_eq!(
                    marker["navigation_anchor_nm"]["x"].as_i64(),
                    Some(-10_000_000)
                );
                assert_eq!(
                    marker["navigation_anchor_nm"]["y"].as_i64(),
                    Some(1_500_000)
                );
                assert_eq!(
                    marker["selected_identity"].as_str(),
                    Some("sha256:e6126fbe2f51a05b6b5d129eabc79191255364134330bc4b587da9889fa99090")
                );
            }
            _ => unreachable!(),
        }
        state.selection = SelectionTarget::Native(Box::new(view));
        if name.starts_with("net-") {
            let context = datum_gui_protocol::inspector_context::InspectorContext::capture(&state);
            let scoped = context
                .sections
                .iter()
                .flat_map(|s| &s.fields)
                .find(|f| f.label == "Scoped ERC")
                .ok_or_else(|| anyhow::anyhow!("scoped ERC field missing"))?;
            assert_eq!(
                context.field_text(scoped, usize::MAX),
                "0 errors · 0 warnings · 0 infos"
            );
            for (label, expected) in [
                ("Project findings", "13"),
                (
                    "Scope coverage",
                    "3 schematic electrical members of 6; other members are not evaluated by ERC",
                ),
                ("Evaluated coverage", "Clean — schematic_connectivity only"),
            ] {
                let field = context
                    .sections
                    .iter()
                    .flat_map(|s| &s.fields)
                    .find(|f| f.label == label)
                    .ok_or_else(|| anyhow::anyhow!("{label} field missing"))?;
                assert_eq!(context.field_text(field, usize::MAX), expected);
            }
        }
        cases.push((name, state));
    }
    let child = base
        .schematic_documents
        .rows
        .iter()
        .find(|row| row.name == "Child A")
        .ok_or_else(|| anyhow::anyhow!("admitted child A missing"))?
        .key
        .clone();
    base.schematic_documents
        .prepare_scene(&child, &root, &model)?;
    let pane = base
        .schematic_documents
        .open_beside(&mut base.ui.layout, &child)
        .ok_or_else(|| anyhow::anyhow!("open qualified child refused"))?;
    // Runtime's shared pane transition publishes both focus owners together.
    base.ui.focus = datum_gui_protocol::ApplicationFocus::Editor(pane);
    assert_eq!(base.ui.layout.focused, pane);
    base.schematic_documents.inspected = Some(child.clone());
    base.inspector_context.owner = InspectorOwner::Navigator;
    assert!(matches!(base.selection, SelectionTarget::None));
    assert_eq!(
        datum_gui_protocol::inspector_context::InspectorContext::capture(&base).document,
        Some(&child)
    );
    cases.push(("qualified-empty-document", base));
    let golden_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/golden/inspector");
    for (size, width, height) in [("normal", 1280, 768), ("narrow", 960, 720)] {
        let mut renderer = datum_gui_render::visual_capture::OffscreenRenderer::new(width, height)?;
        for (name, state) in &cases {
            let name = format!("{name}-{size}");
            let actual = renderer.render_workspace(state, None)?;
            assert_eq!(actual.dimensions(), (width, height));
            actual.save(output.join(format!("{name}.png")))?;
            if candidate {
                println!("CANDIDATE_ONLY {name}: rendered; golden comparison NOT RUN");
            } else {
                let expected =
                    image::open(golden_root.join(format!("{name}.golden.png")))?.to_rgba8();
                assert_eq!(
                    actual.dimensions(),
                    expected.dimensions(),
                    "{name} dimensions"
                );
                assert_eq!(
                    actual.as_raw(),
                    expected.as_raw(),
                    "{name} exact renderer golden"
                );
                println!("GOLDEN_MATCH {name}: exact same-engine pixels");
            }
        }
    }
    for (shard, bytes) in model.source_shards.iter().zip(source_before) {
        assert_eq!(
            std::fs::read(root.join(&shard.path))?,
            bytes,
            "read-only admitted source"
        );
    }
    Ok(())
}
