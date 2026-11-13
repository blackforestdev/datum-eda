//! Canonical native Rule meaning crosses the public DRC reader without rewriting.
use super::*;
use eda_engine::api::native_write::{
    WriteProvenance,
    board_routing::{build_place_board_net, build_place_board_net_class, build_place_board_track},
    commit_prepared,
    project::{build_create_project_rule, build_set_project_rule},
};
use eda_engine::board::{Net, NetClass, Track};
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind};
use serde_json::{Value, json};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "native-clearance-proof",
        CommitSource::Test,
        "Canonical Rule fixture",
    )
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("datum-native-clearance-{}", Uuid::new_v4()));
        create_native_project(&root, Some("Native clearance".into())).unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let class = NetClass {
            uuid: Uuid::new_v4(),
            name: "Existing class".into(),
            clearance: 100_000,
            track_width: 100_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        };
        let prepared = build_place_board_net_class(&model, provenance(), &class).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        // Centre separation 250000 minus two half-widths 50000: real gap150000.
        for (name, y) in [("A", 0), ("B", 250_000)] {
            let net = Net::new(Uuid::new_v4(), name, class.uuid);
            let prepared = build_place_board_net(&model, provenance(), &net).unwrap();
            commit_prepared(&mut model, &root, prepared).unwrap();
            let track = Track {
                uuid: Uuid::new_v4(),
                net: net.uuid,
                from: Point::new(0, y),
                to: Point::new(1_000_000, y),
                midpoint: None,
                width: 100_000,
                layer: 1,
            };
            let prepared = build_place_board_track(&model, provenance(), &track).unwrap();
            commit_prepared(&mut model, &root, prepared).unwrap();
        }
        Self { root }
    }

    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }

    fn rule(id: Uuid, minimum: i64, priority: u64) -> Value {
        json!({"uuid":id,"name":"Authored native clearance","scope":"All",
            "priority":priority,"enabled":true,"rule_type":"clearance_copper",
            "params":{"min_nm":minimum}})
    }

    fn write_rule(&self, value: Value, create: bool) {
        let mut model = self.model();
        let (prepared, _) = if create {
            build_create_project_rule(&model, provenance(), value)
        } else {
            build_set_project_rule(&model, provenance(), value)
        }
        .unwrap();
        commit_prepared(&mut model, &self.root, prepared).unwrap();
    }

    fn rules(&self) -> Value {
        self.model()
            .materialized_source_shard_value(SourceShardKind::RulesRoot)
            .unwrap()
    }

    fn persisted_rule(&self, input: &Value) -> Value {
        let rules = self.rules();
        let persisted = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|rule| rule["uuid"] == input["uuid"])
            .unwrap();
        for (key, value) in input.as_object().unwrap() {
            assert_eq!(&persisted[key], value, "canonical Rule field {key}");
        }
        assert!(persisted["object_revision"].as_u64().is_some());
        persisted.clone()
    }

    fn rules_bytes(&self) -> Vec<u8> {
        let model = self.model();
        let path = &model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::RulesRoot)
            .unwrap()
            .path;
        std::fs::read(path).unwrap()
    }

    fn run(&self) -> NativeProjectCheckRunView {
        run_native_project_check_with_profile(&self.root, Some("drc")).unwrap()
    }

    fn clearance_count(&self) -> usize {
        self.run()
            .findings
            .iter()
            .filter(|f| f.code == "clearance_copper")
            .count()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn canonical_clearance_rule_changes_actual_public_drc_and_survives_history_and_cold() {
    let f = Fixture::new();
    assert_eq!(
        f.clearance_count(),
        0,
        "the existing100000 class permits the150000 gap"
    );
    let id = Uuid::new_v4();
    let priority = u64::from(u32::MAX) + 2;
    let authored = Fixture::rule(id, 200_000, priority);
    f.write_rule(authored.clone(), true);
    let authored = f.persisted_rule(&authored);
    let bytes = f.rules_bytes();
    let revision = f.model().model_revision;
    let report = f.run();
    let findings = report
        .findings
        .iter()
        .filter(|v| v.code == "clearance_copper")
        .collect::<Vec<_>>();
    assert_eq!(
        findings.len(),
        1,
        "a canonical200000 Rule must override the100000 class"
    );
    assert!(findings[0].message.contains("200000"));
    assert_eq!(
        f.rules_bytes(),
        bytes,
        "DRC never rewrites Rule JSON into a legacy dialect"
    );
    assert_eq!(f.model().model_revision, revision);
    assert_eq!(f.rules()["rules"][0], authored);

    // Exercise the independent source-qualified public DRC caller too.
    let direct =
        query_native_project_drc_with_rules(&f.root, &[RuleType::ClearanceCopper]).unwrap();
    assert_eq!(
        direct
            .violations
            .iter()
            .filter(|v| v.code == "clearance_copper")
            .count(),
        1
    );
    assert_eq!(f.rules_bytes(), bytes);

    let updated = Fixture::rule(id, 100_000, priority);
    f.write_rule(updated.clone(), false);
    let updated = f.persisted_rule(&updated);
    let mut model = f.model();
    model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let mut undone = authored.clone();
    undone["object_revision"] = json!(updated["object_revision"].as_u64().unwrap() + 1);
    assert_eq!(f.rules()["rules"][0], undone);
    model = f.model();
    model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let mut redone = updated.clone();
    redone["object_revision"] = json!(undone["object_revision"].as_u64().unwrap() + 1);
    assert_eq!(f.rules()["rules"][0], redone);
    let bytes = f.rules_bytes();
    let next = f.run();
    assert!(!next.findings.iter().any(|v| v.code == "clearance_copper"));
    assert_ne!(next.check_run_id, report.check_run_id);
    let cold = f.model();
    assert!(
        cold.check_runs[&report.check_run_id]
            .findings
            .iter()
            .any(|v| v.code == "clearance_copper")
    );
    assert!(
        !cold.check_runs[&next.check_run_id]
            .findings
            .iter()
            .any(|v| v.code == "clearance_copper")
    );
    assert_eq!(f.rules_bytes(), bytes);
    assert_eq!(
        f.rules()["rules"][0],
        redone,
        "UUID, full priority and parameters survive cold readback"
    );
}

#[test]
fn native_clearance_priority_disabled_layer_and_ambiguity_keep_persisted_meaning() {
    let f = Fixture::new();
    let low = Fixture::rule(Uuid::new_v4(), 200_000, u64::from(u32::MAX));
    let mut high = Fixture::rule(Uuid::new_v4(), 100_000, u64::from(u32::MAX) + 2);
    f.write_rule(low.clone(), true);
    let persisted_low = f.persisted_rule(&low);
    f.write_rule(high.clone(), true);
    // Narrowing the high priority to u32 would reverse these two actual rules.
    let clear = f.run();
    assert!(
        !clear.findings.iter().any(|finding| matches!(
            finding.code.as_str(),
            "clearance_copper" | "nominal_geometry_unavailable" | "track_width_policy_unresolved"
        )),
        "high-priority clearance must be certified clear, never unavailable: {:?}",
        clear.findings
    );
    high["enabled"] = json!(false);
    f.write_rule(high.clone(), false);
    assert_eq!(f.clearance_count(), 1);
    high["enabled"] = json!(true);
    high["scope"] = json!({"Layer":2});
    f.write_rule(high.clone(), false);
    assert_eq!(
        f.clearance_count(),
        1,
        "an inactive layer scope cannot override layer1"
    );
    high["scope"] = json!("All");
    high["priority"] = low["priority"].clone();
    f.write_rule(high.clone(), false);
    let persisted_high = f.persisted_rule(&high);
    let report = f.run();
    assert!(
        report
            .findings
            .iter()
            .any(|v| v.code == "nominal_geometry_unavailable" && v.message.contains("ambiguous")),
        "an applicable policy ambiguity must remain unavailable, never class fallback"
    );
    assert_eq!(f.rules()["rules"], json!([persisted_low, persisted_high]));
}

#[test]
fn native_rule_reader_preserves_legacy_ast_and_refuses_unbacked_or_unsupported_native_policy() {
    let f = Fixture::new();
    let mut project = load_native_project_with_resolved_board(&f.root).unwrap();
    let legacy = Rule {
        uuid: Uuid::new_v4(),
        name: "Legacy".into(),
        scope: RuleScope::All,
        priority: 7,
        enabled: true,
        rule_type: RuleType::ClearanceCopper,
        parameters: RuleParams::Clearance { min: 123_456 },
    };
    // Read-only compatibility control, not a claim that the canonical native
    // writer accepts the old AST representation or that legacy rules are new.
    project.rules.rules = vec![serde_json::to_value(&legacy).unwrap()];
    let board = build_native_project_board(&project).unwrap();
    assert_eq!(board.rules, vec![legacy]);

    f.write_rule(Fixture::rule(Uuid::new_v4(), 200_000, 0), true);
    let project = load_native_project_with_resolved_board(&f.root).unwrap();
    assert!(
        build_native_project_board(&project)
            .unwrap_err()
            .to_string()
            .contains("captured native Project")
    );
    let unsupported = json!({"uuid":Uuid::new_v4(),"name":"Native via constraint",
        "scope":"All","priority":0,"enabled":true,"rule_type":"via_hole",
        "params":{"min_nm":100000,"max_nm":300000}});
    f.write_rule(unsupported, true);
    let (project, model) = load_native_project_with_resolved_board_and_model(&f.root).unwrap();
    let error = build_native_project_board_with_model(&project, Some(&model)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("no supported native Board evaluator")
    );
}
