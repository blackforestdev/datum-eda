//! Native generated-copper epochs, canonical evidence reuse and cold readback.
use super::*;
use eda_engine::api::native_write::{
    WriteProvenance,
    artifacts::build_set_check_run,
    board_routing::zone::{FillRequest, capture_board_basis, prepare_fill},
    commit_prepared,
};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::substrate::{CheckRun, CommitSource, DesignModel, ModelRevision, ProjectResolver};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "basis-test",
        CommitSource::Test,
        "Explicit native check input change",
    )
}
struct Fixture {
    root: PathBuf,
    zone: Uuid,
    revision: ModelRevision,
    sources: Vec<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let root = unique_project_root("datum-native-drc-zone-inputs");
        create_native_project(&root, Some("Native DRC zone inputs".into())).unwrap();
        build_native_check_fixture(&root);
        seed_board_process_aperture_fixture(&root);
        let outline = Polygon::new(vec![
            Point::new(-2_000_000, -2_000_000),
            Point::new(8_000_000, -2_000_000),
            Point::new(8_000_000, 3_000_000),
            Point::new(-2_000_000, 3_000_000),
        ]);
        set_native_project_board_outline(&root, outline).unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let board = model
            .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        let net = board["nets"]
            .as_object()
            .unwrap()
            .values()
            .find(|net| net["name"] == "SIG")
            .unwrap()["uuid"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let zone = place_native_project_board_zone(
            &root,
            net,
            Polygon::new(vec![
                Point::new(-1_000_000, -1_000_000),
                Point::new(6_000_000, -1_000_000),
                Point::new(6_000_000, 2_000_000),
                Point::new(-1_000_000, 2_000_000),
            ]),
            1,
            0,
            false,
            0,
            0,
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let zone = zone.zone_uuid.parse().unwrap();
        let sources = model.source_shards.iter().map(|s| s.path.clone()).collect();
        Self {
            root,
            zone,
            revision: model.model_revision,
            sources,
        }
    }
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn authored(&self) -> Vec<Vec<u8>> {
        self.sources
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect()
    }
    fn fill(&self) {
        let mut model = self.model();
        let request = FillRequest {
            basis: capture_board_basis(&model).unwrap(),
            zone: Some(self.zone),
            net: None,
        };
        let prepared = prepare_fill(&model, provenance(), &request).unwrap();
        commit_prepared(&mut model, &self.root, prepared).unwrap();
        let cold = self.model();
        assert_eq!(cold.model_revision, self.revision);
        let fill = cold.current_zone_fill(self.zone).expect(
            "actual bounded fill must be current Filled, never a substituted Unsupported pass",
        );
        assert!(!fill.islands.is_empty());
    }
    fn run(&self) -> NativeProjectCheckRunView {
        run_native_project_check_with_profile(&self.root, Some("drc")).unwrap()
    }
    fn show(&self, id: Uuid) -> serde_json::Value {
        serde_json::from_str(
            &execute(
                Cli::try_parse_from([
                    "eda",
                    "--format",
                    "json",
                    "check",
                    "show",
                    self.root.to_str().unwrap(),
                    "--check-run",
                    &id.to_string(),
                ])
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn native_drc_zone_copper_basis_changes_after_fill_and_cold_reopen() {
    let f = Fixture::new();
    let authored = f.authored();
    let before = f.run();
    let before_id = before.check_run_id;
    let prior = f.model();
    let length = prior.journal.len();
    assert_eq!(f.run().check_run_id, before_id);
    assert_eq!(
        f.model().journal.len(),
        length,
        "identical actual facts are a no-write reuse"
    );
    f.fill();
    let filled = f.model();
    assert!(
        filled
            .qualify_native_drc_zone_copper_basis(
                before
                    .raw_report
                    .get(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY)
            )
            .is_err()
    );
    let shown = f.show(before_id);
    assert_eq!(
        shown["check_run"]["raw_report"], before.raw_report,
        "old evidence remains readable and unchanged"
    );
    let after = f.run();
    assert_ne!(after.check_run_id, before_id);
    let mut cold = f.model();
    assert_eq!(cold.model_revision, f.revision);
    cold.qualify_native_drc_zone_copper_basis(
        after
            .raw_report
            .get(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY),
    )
    .unwrap();
    let length = cold.journal.len();
    assert_eq!(f.run().check_run_id, after.check_run_id);
    assert_eq!(f.model().journal.len(), length);
    let correct = cold.check_runs[&after.check_run_id].clone();
    let mut prior_facts = correct.clone();
    prior_facts.raw_report["prior_producer_note"] = serde_json::json!("different stored facts");
    let prepared = build_set_check_run(&cold, provenance(), &prior_facts).unwrap();
    commit_prepared(&mut cold, &f.root, prepared).unwrap();
    let length = f.model().journal.len();
    let refreshed = f.run();
    assert_eq!(refreshed.check_run_id, after.check_run_id);
    cold = f.model();
    assert_eq!(
        cold.journal.len(),
        length + 1,
        "different facts cannot be silently reused"
    );
    assert_eq!(cold.check_runs[&after.check_run_id], correct);
    cold.commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    cold = f.model();
    assert_eq!(cold.check_runs[&after.check_run_id], prior_facts);
    cold.commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(f.model().check_runs[&after.check_run_id], correct);
    assert_eq!(f.authored(), authored);
}

#[test]
fn guarded_drc_zone_fill_race_refuses_new_and_existing_evidence() {
    for (existing, change_facts) in [(false, false), (true, false), (true, true)] {
        let f = Fixture::new();
        if existing {
            f.run();
        }
        let model = f.model();
        let fingerprint = model.native_drc_zone_copper_fingerprint().unwrap();
        // A real captured check is sufficient for this commit-refusal control;
        // no stale result is accepted or persisted as full profile evidence.
        let captured =
            capture_native_drc_inputs(&f.root, &model, &[RuleType::Connectivity]).unwrap();
        if change_facts {
            let mut current = f.model();
            let mut record = current
                .check_runs
                .values()
                .find(|r| r.profile_id == "drc")
                .unwrap()
                .clone();
            record.raw_report["concurrent_producer_note"] =
                serde_json::json!("changed after capture");
            let prepared = build_set_check_run(&current, provenance(), &record).unwrap();
            commit_prepared(&mut current, &f.root, prepared).unwrap();
        } else {
            f.fill();
        }
        let unchanged = check_source_bytes(&f.root);
        assert!(
            finish_check_run(
                &f.root,
                model,
                captured.project,
                captured.report,
                "drc",
                true,
                true
            )
            .is_err(),
            "stale source or changed persisted facts must refuse atomically"
        );
        assert_eq!(check_source_bytes(&f.root), unchanged);
        if !change_facts {
            let current = f.model();
            let args = Cli::try_parse_from([
                "eda",
                "check",
                "run",
                f.root.to_str().unwrap(),
                "--profile",
                "drc",
                "--expected-project",
                &current.project.project_id.to_string(),
                "--expected-revision",
                &current.model_revision.0,
                "--expected-zone-copper-fingerprint",
                &fingerprint,
                "--progress-job",
                &Uuid::new_v4().to_string(),
            ])
            .unwrap();
            assert!(execute(args).is_err());
            assert_eq!(check_source_bytes(&f.root), unchanged);
        }
    }
}

#[test]
fn native_drc_legacy_and_unknown_copper_basis_remain_readable() {
    let f = Fixture::new();
    let authored = f.authored();
    let view = f.run();
    let id = view.check_run_id;
    let original = f.model().check_runs[&id].clone();
    for version in [None, Some(2), Some(0)] {
        let mut model = f.model();
        let mut record = original.clone();
        match version {
            None => {
                record
                    .raw_report
                    .as_object_mut()
                    .unwrap()
                    .remove(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY);
            }
            Some(2) => {
                record.raw_report[CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY]["version"] =
                    serde_json::json!(2)
            }
            _ => {
                record.raw_report[CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY] =
                    serde_json::json!("malformed legacy extension")
            }
        }
        let prepared = build_set_check_run(&model, provenance(), &record).unwrap();
        commit_prepared(&mut model, &f.root, prepared).unwrap();
        let shown = f.show(id);
        assert_eq!(
            shown["check_run"]["findings"],
            serde_json::to_value(&original.findings).unwrap()
        );
        assert!(
            f.model()
                .qualify_native_drc_zone_copper_basis(
                    shown["check_run"]["raw_report"]
                        .get(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY)
                )
                .is_err()
        );
        assert_eq!(f.run().check_run_id, id);
        assert_eq!(
            f.model().check_runs[&id],
            original,
            "explicit rerun canonically refreshes incompatible stored facts"
        );
    }
    f.fill();
    let mut model = f.model();
    assert!(
        model
            .qualify_native_drc_zone_copper_basis(
                original
                    .raw_report
                    .get(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY)
            )
            .is_err()
    );
    model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let restored = f.model();
    restored
        .qualify_native_drc_zone_copper_basis(
            original
                .raw_report
                .get(CheckRun::NATIVE_DRC_ZONE_COPPER_BASIS_KEY),
        )
        .unwrap();
    assert_eq!(restored.model_revision, f.revision);
    assert_eq!(f.authored(), authored);
}
