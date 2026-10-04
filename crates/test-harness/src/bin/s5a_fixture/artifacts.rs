//! Existing producer artifacts bind action/finding identities; scene code never
//! generates them. Authored fixture geometry uses only canonical operations.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance},
    board::{Net, NetClass, PadLayerConnection, PadShape, PlacedPad},
    ir::geometry::{Point, Polygon},
    substrate::{CommitSource, Operation, ProjectResolver, transaction_journal_path},
};
use serde_json::{Value, json};
use std::{path::Path, process::Command};
use uuid::Uuid;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x74000000000040008000000000000000 + n)
}
fn invoke(cli: &Path, args: &[String]) -> Result<Value> {
    let output = Command::new(cli)
        .args(["--format", "json"])
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "native producer {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
pub(super) fn build(root: &Path, cli: &Path) -> Result<()> {
    ensure!(
        cli.is_file() && !root.exists(),
        "fresh root and existing pinned CLI required"
    );
    let baseline = super::recipe::build_and_check(root, 1)?;
    let no_checks = root.join("variants/no-checks");
    let no_checks_recipe = super::recipe::build_and_check(&no_checks, 1)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let provenance = || {
        WriteProvenance::new(
            "s5a-fixture",
            CommitSource::Test,
            "source-bound native artifact fixture",
        )
    };
    let class = NetClass {
        uuid: id(10),
        name: "Artifact fixture route".into(),
        clearance: 200000,
        track_width: 200000,
        via_drill: 250000,
        via_diameter: 1000000,
        diffpair_width: 200000,
        diffpair_gap: 200000,
    };
    let net = Net::new(id(1), "artifact route", class.uuid);
    let pads: Vec<_> = [(2, 0), (3, 10000000)]
        .into_iter()
        .map(|(n, x)| PlacedPad {
            uuid: id(n),
            package: Uuid::nil(),
            name: format!("P{n}"),
            net: Some(net.uuid),
            position: Point::new(x, 50000000),
            layer: 1,
            copper_layers: vec![1],
            shape: PadShape::Rect,
            diameter: 0,
            width: 2000000,
            height: 2000000,
            drill: 0,
            rotation: 0,
            roundrect_rratio_ppm: 250000,
            mask_layers: vec![],
            paste_layers: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            layer_connection: PadLayerConnection::Separate,
        })
        .collect();
    let ops = vec![
        Operation::CreateBoardNetClass {
            net_class_id: class.uuid,
            net_class: serde_json::to_value(class)?,
        },
        Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(&net)?,
        },
    ];
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .push_ops(pads.iter().map(|p| Operation::CreateBoardPad {
            pad_id: p.uuid,
            pad: serde_json::to_value(p).unwrap(),
        }))
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let board: Uuid = baseline["board_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("actual canonical board identity"))?
        .parse()?;
    let outline = Polygon::new(vec![
        Point::new(-50000000, -50000000),
        Point::new(200000000, -50000000),
        Point::new(200000000, 100000000),
        Point::new(-50000000, 100000000),
    ]);
    let write =
        native_write::board_layout::build_set_board_outline(&model, provenance(), board, &outline)?;
    native_write::commit_prepared(&mut model, root, write)?;
    let artifact = root.join("route-proposal.json");
    let export = invoke(
        cli,
        &[
            "project".into(),
            "export-route-proposal".into(),
            root.display().to_string(),
            "--net".into(),
            net.uuid.to_string(),
            "--from-anchor".into(),
            pads[0].uuid.to_string(),
            "--to-anchor".into(),
            pads[1].uuid.to_string(),
            "--out".into(),
            artifact.display().to_string(),
        ],
    )?;
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
    let reopened = ProjectResolver::new(root).resolve()?;
    let files: Vec<_> = reopened
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = transaction_journal_path(root);
    let bytes = std::fs::read(&journal)?;
    let review = invoke(
        cli,
        &[
            "project".into(),
            "review-route-proposal".into(),
            "--artifact".into(),
            artifact.display().to_string(),
        ],
    )?;
    for (path, before) in files {
        ensure!(
            std::fs::read(path)? == before,
            "source unchanged around artifact review"
        );
    }
    ensure!(
        std::fs::read(&journal)? == bytes,
        "journal unchanged around artifact review"
    );
    let report = json!({"family":"non-authored-channels","construction_baseline":baseline,"project_id":reopened.project.project_id,
        "model_revision":reopened.model_revision.0,"artifact":artifact,"producer_receipts":{"export":export,"check":check,"review":review},
        "no_check_variant":no_checks_recipe,"route_anchors":pads.iter().map(|p|p.uuid).collect::<Vec<_>>(),"route_net":net.uuid,
        "nonmutation":"complete source/journal byte equality around actual saved artifact review and reopen",
        "unverified":["generic production proposal action authority","native pointer/keyboard/visual lifecycle","C04/C05/C06 acceptance"],
        "manufacturing":"CAM/T04 remains open; fixture rules are authored design constraints, not manufacturing allowance"});
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&report)?,
    )?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
