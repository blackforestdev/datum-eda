//! One canonical batch on the exact governed artifact baseline, followed by
//! real producers against the resulting revision. No shard or artifact graft.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance},
    board::BoardText,
    substrate::{CommitSource, Operation, ProjectResolver},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, process::Command};
use uuid::Uuid;

const COUNT: usize = 100000;
fn text_id(n: usize) -> Uuid {
    Uuid::from_u128(0x70000000000040008000000000001000 + n as u128)
}
fn invoke(cli: &Path, args: &[String]) -> Result<Value> {
    let out = Command::new(cli)
        .args(["--format", "json"])
        .args(args)
        .output()?;
    ensure!(
        out.status.success(),
        "actual producer {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(serde_json::from_slice(&out.stdout)?)
}
pub(super) fn build(root: &Path, cli: &Path) -> Result<()> {
    let report_path = root.join("s5a-collision-fixture.json");
    ensure!(
        root.is_dir() && cli.is_file() && !report_path.exists(),
        "fresh exact baseline copy and pinned CLI required"
    );
    let mut model = ProjectResolver::new(root).resolve()?;
    ensure!(
        model.project.project_id == "20000000-0000-4000-8000-000000000001".parse::<Uuid>()?,
        "governed Project identity"
    );
    ensure!(
        model.model_revision.0
            == "85f3b5df6fc6f00742a44bd4e4c79fa779d3f5304212dc954d00f992850078c9",
        "exact pre-write artifact baseline revision"
    );
    ensure!(
        (0..COUNT).all(|n| !model.objects.contains_key(&text_id(n))),
        "BoardText IDs must be globally unused"
    );
    let original_revision = model.model_revision.clone();
    let original_objects = model.objects.clone();
    let mut operations = Vec::new();
    operations.try_reserve_exact(COUNT)?;
    for n in 0..COUNT {
        // Deserialize through the public typed defaults, then serialize the
        // complete BoardText authority into the ordinary operation payload.
        let text: BoardText = serde_json::from_value(json!({
            "uuid": text_id(n), "text": "I",
            "position": {"x": (n % 400) as i64 * 2000000, "y": (n / 400) as i64 * 2000000},
            "rotation": 0, "layer": 1, "height_nm": 640000, "stroke_width_nm": 100000
        }))?;
        operations.push(Operation::CreateBoardText {
            text_id: text.uuid,
            text: serde_json::to_value(text)?,
        });
    }
    let write = BatchComposer::compose(
        &model,
        WriteProvenance::new(
            "s5a-fixture",
            CommitSource::Test,
            "governed same-pane maximal-channel fixture: one 100000 BoardText canonical batch",
        ),
    )
    .push_ops(operations)
    .finish()?;
    let batch = write.batch.clone();
    let commit = native_write::commit_prepared(&mut model, root, write)?;
    let reopened = ProjectResolver::new(root).resolve()?;
    ensure!(
        reopened.model_revision == model.model_revision && reopened.objects == model.objects,
        "journal reopen must preserve complete object authority"
    );
    ensure!(
        reopened.objects.len() == original_objects.len() + COUNT,
        "complete authored population, not a truncated counter"
    );
    for (id, before) in &original_objects {
        ensure!(
            reopened
                .objects
                .get(id)
                .is_some_and(|after| after.domain == before.domain && after.kind == before.kind),
            "baseline object identity/class changed: {id}"
        );
    }
    for n in 0..COUNT {
        ensure!(
            reopened.objects.contains_key(&text_id(n)),
            "missing canonical BoardText {}",
            text_id(n)
        );
    }
    // The check is a normal canonical producer write. Export AFTER that write
    // so the saved proposal is generated against the final current basis.
    let check = invoke(
        cli,
        &[
            "check".into(),
            "run".into(),
            root.display().to_string(),
            "--profile".into(),
            "drc".into(),
        ],
    )?;
    let artifact = root.join("collision-route-proposal.json");
    let export = invoke(
        cli,
        &[
            "project".into(),
            "export-route-proposal".into(),
            root.display().to_string(),
            "--net".into(),
            "74000000-0000-4000-8000-000000000001".into(),
            "--from-anchor".into(),
            "74000000-0000-4000-8000-000000000002".into(),
            "--to-anchor".into(),
            "74000000-0000-4000-8000-000000000003".into(),
            "--out".into(),
            artifact.display().to_string(),
        ],
    )?;
    let current = ProjectResolver::new(root).resolve()?;
    let review = invoke(
        cli,
        &[
            "project".into(),
            "review-route-proposal".into(),
            "--artifact".into(),
            artifact.display().to_string(),
        ],
    )?;
    let mut classes = BTreeMap::<String, usize>::new();
    for object in current.objects.values() {
        *classes
            .entry(format!("{}:{}", object.domain, object.kind))
            .or_default() += 1;
    }
    let report = json!({
        "family": "maximal-channel-collision", "project_id": current.project.project_id,
        "baseline_revision": original_revision, "model_revision": current.model_revision,
        "canonical_batch": batch, "commit": commit,
        "text_population": {"count":COUNT,"first":text_id(0),"last":text_id(COUNT-1),
            "columns":400,"rows":250,"spacing_nm":2000000,"literal":"I","layer":1,
            "height_nm":640000,"stroke_width_nm":100000,"defaults":"public typed BoardText defaults"},
        "baseline_objects":original_objects,"current_object_class_counts":classes,
        "producer_receipts":{"check":check,"export":export,"review":review},"artifact":artifact,
        "complete_board_selection_oracle":"derive full visible authored Track/Pad/Outline/Text classes from source-qualified projection; baseline extras are not part of the 100000 text count",
        "native_acceptance":"not established; construction and producer records only",
        "limits":"No private shard/artifact graft, restored old producer basis, new rule/check/profile/cap, or full C04/HUMAN acceptance"
    });
    std::fs::write(&report_path, eda_test_harness::canonical_json(&report)?)?;
    println!("{}", report_path.display());
    Ok(())
}
