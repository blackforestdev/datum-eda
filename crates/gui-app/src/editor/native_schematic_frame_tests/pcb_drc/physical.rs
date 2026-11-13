//! Real canonical physical-error setup/correction; not a GUI routing-edit claim.
use super::*;
use eda_engine::{
    board::{Net, NetClass, Track, Via},
    ir::geometry::Point,
};
use native::board_routing::*;

const CODES: [&str; 3] = [
    "track_width_below_min",
    "via_hole_out_of_range",
    "via_annular_below_min",
];

pub(super) struct PhysicalCase {
    track: Track,
    vias: [Via; 2],
    selected: usize,
    pub corrected: bool,
    historical_run: Option<Uuid>,
}

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "physical-drc-fixture",
        CommitSource::Test,
        "Explicit canonical physical fixture operation",
    )
}

impl PhysicalCase {
    pub fn prepare(root: &std::path::Path, zone: Uuid) -> Self {
        let mut model = ProjectResolver::new(root).resolve().unwrap();
        let class = NetClass {
            uuid: Uuid::new_v4(),
            name: "Physical proof".into(),
            clearance: 100_000,
            track_width: 200_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        };
        let net = Net {
            uuid: Uuid::new_v4(),
            name: "PHYSICAL_PROOF".into(),
            class: class.uuid,
            controlled_impedance: None,
        };
        let track = Track {
            uuid: Uuid::new_v4(),
            net: net.uuid,
            from: Point::new(7_000_000, 5_000_000),
            to: Point::new(8_000_000, 5_000_000),
            midpoint: None,
            width: 100_000,
            layer: 1,
        };
        let via = Via {
            uuid: Uuid::new_v4(),
            net: net.uuid,
            position: track.to,
            drill: 100_000,
            diameter: 400_000,
            from_layer: 1,
            to_layer: 1,
        };
        let annular = Via {
            uuid: Uuid::new_v4(),
            position: Point::new(7_000_000, 5_500_000),
            drill: 300_000,
            ..via.clone()
        };
        let prepared = build_place_board_net_class(&model, provenance(), &class).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        // The existing fill fixture uses the same persisted NetClass authority
        // as the physical checks; no alternate Rule JSON or reader fallback.
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let zone_net = board["zones"][zone.to_string()]["net"].as_str().unwrap();
        let mut fill_net: Net = serde_json::from_value(board["nets"][zone_net].clone()).unwrap();
        fill_net.class = class.uuid;
        let prepared = build_set_board_net(&model, provenance(), &fill_net).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        let prepared = build_place_board_net(&model, provenance(), &net).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        let prepared = build_place_board_track(&model, provenance(), &track).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        let vias = [via, annular];
        for via in &vias {
            let prepared = build_place_board_via(&model, provenance(), via).unwrap();
            commit_prepared(&mut model, root, prepared).unwrap();
        }
        Self {
            track,
            vias,
            selected: 0,
            corrected: false,
            historical_run: None,
        }
    }

    pub fn assert_report(&self, state: &datum_gui_protocol::ReviewWorkspaceState) {
        use datum_gui_protocol::selection_artifacts::diagnostic_markers::MarkerProjectionIndex;
        let findings = &state.checks.findings;
        let markers = MarkerProjectionIndex::capture_native(
            &state.scene,
            state,
            datum_gui_protocol::PaneContent::Board,
            |_| true,
            Some(&state.schematic_documents),
        );
        for code in CODES {
            let finding = findings.iter().find(|f| f.code == code);
            if self.corrected {
                assert!(finding.is_none(), "corrected {code} still reported");
            } else {
                let finding = finding.unwrap_or_else(|| panic!("missing physical error {code}"));
                assert_eq!(finding.domain, "standards");
                assert_eq!(finding.source, "drc");
                assert_eq!(finding.severity, "error");
                assert!(
                    markers.anchor(finding).is_ok(),
                    "physical finding must navigate on Board"
                );
            }
        }
    }

    pub fn marker_target(&self, state: &datum_gui_protocol::ReviewWorkspaceState) -> HitTarget {
        let finding = state
            .checks
            .findings
            .iter()
            .find(|f| f.code == CODES[self.selected])
            .unwrap();
        HitTarget::CheckFinding(finding.fingerprint.clone())
    }

    pub fn capture_name(&self) -> &'static str {
        [
            "physical-marker-selected",
            "physical-hole-selected",
            "physical-annular-selected",
        ][self.selected]
    }

    pub fn next_marker(&mut self) -> bool {
        self.selected += 1;
        self.selected < CODES.len()
    }

    pub fn assert_selected(&self, state: &datum_gui_protocol::ReviewWorkspaceState) {
        let view = datum_gui_protocol::inspector_context::finding::capture(state)
            .unwrap()
            .unwrap();
        assert_eq!(view.finding.code, CODES[self.selected]);
        assert_eq!(view.finding.severity, "error");
        assert_eq!(view.currentness, "Current persisted result");
        assert!(
            matches!(&state.selection, datum_gui_protocol::SelectionTarget::Native(v)
            if v.members.is_empty() && matches!(&v.subject, datum_gui_protocol::SelectionSubject::Diagnostic(id) if id == &view.finding.fingerprint))
        );
    }

    pub fn correct(&mut self, root: &std::path::Path, run: &str) -> DesignModel {
        let mut model = ProjectResolver::new(root).resolve().unwrap();
        self.historical_run = Some(run.parse().unwrap());
        let before = model.journal.len();
        let revision = model.model_revision.clone();
        self.track.width = 200_000;
        let prepared = build_set_board_track(&model, provenance(), &self.track).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        for via in &mut self.vias {
            via.drill = 300_000;
            via.diameter = 600_000;
            let prepared = build_set_board_via(&model, provenance(), via).unwrap();
            commit_prepared(&mut model, root, prepared).unwrap();
        }
        let model = ProjectResolver::new(root).resolve().unwrap();
        assert_ne!(model.model_revision, revision);
        assert_eq!(model.journal.len(), before + 3);
        let operations: Vec<_> = model.journal[before..]
            .iter()
            .flat_map(|t| &t.operations)
            .collect();
        assert_eq!(
            operations
                .iter()
                .filter(|op| matches!(op, Operation::SetBoardTrack { .. }))
                .count(),
            1
        );
        assert_eq!(
            operations
                .iter()
                .filter(|op| matches!(op, Operation::SetBoardVia { .. }))
                .count(),
            2
        );
        assert!(operations.iter().all(|op| matches!(
            op,
            Operation::SetBoardTrack { .. }
                | Operation::SetBoardVia { .. }
                | Operation::GuardObjectRevision { .. }
        )));
        self.corrected = true;
        model
    }

    pub fn assert_history(&self, model: &DesignModel, current: &str) {
        let old = self.historical_run.unwrap();
        let current: Uuid = current.parse().unwrap();
        assert_ne!(old, current);
        for code in CODES {
            assert!(
                model.check_runs[&old]
                    .findings
                    .iter()
                    .any(|f| f.code == code && f.severity == "error")
            );
            assert!(
                !model.check_runs[&current]
                    .findings
                    .iter()
                    .any(|f| f.code == code)
            );
        }
        assert_ne!(
            model.check_runs[&old].model_revision,
            model.check_runs[&current].model_revision
        );
    }
}
