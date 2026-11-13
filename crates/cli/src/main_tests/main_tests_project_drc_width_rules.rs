//! Public saved native/legacy rule meaning and typed width enforcement.
use super::main_tests_project_drc::seed_board_drc_fixture;
use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{ProjectResolver, SourceShardKind};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

#[test]
fn saved_native_authored_width_rule_governs_public_drc_and_correction() {
    use eda_engine::api::native_write::{
        WriteProvenance, board_routing, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    use eda_engine::board::{NetClass, Track};
    use eda_engine::ir::geometry::Point;
    use eda_engine::substrate::CommitSource;
    let root = unique_project_root("datum-native-authored-width");
    create_native_project(&root, Some("Authored width".into())).unwrap();
    let net = seed_board_drc_fixture(&root);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let provenance = || {
        WriteProvenance::new(
            "width-rule-test",
            CommitSource::Test,
            "Canonical width fixture",
        )
    };
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let class_id = board["nets"][net.to_string()]["class"].as_str().unwrap();
    let mut class: NetClass =
        serde_json::from_value(board["net_classes"][class_id].clone()).unwrap();
    class.track_width = 100000;
    let prepared = board_routing::build_set_board_net_class(&model, provenance(), &class).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut track = Track::straight(
        Uuid::new_v4(),
        net,
        Point::new(0, 0),
        Point::new(5000000, 0),
        200000,
        1,
    );
    let prepared = board_routing::build_place_board_track(&model, provenance(), &track).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let query = || {
        let cli = Cli::try_parse_from([
            "datum-eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "drc",
        ])
        .unwrap();
        serde_json::from_str::<serde_json::Value>(&execute(cli).unwrap()).unwrap()
    };
    let width_errors = |report: &serde_json::Value| {
        report["raw_report"]["drc"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["code"] == "track_width_below_min")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert!(width_errors(&query()).is_empty());
    let rule = serde_json::json!({"uuid":Uuid::new_v4(),"name":"All width 500000nm","scope":"All","priority":1,"enabled":true,
        "rule_type":"track_width","params":{"min_nm":500000,"preferred_nm":500000,"max_nm":800000}});
    let prepared = (find_native_write_verb("datum.project.create_rule")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        serde_json::json!({"rule":rule}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let persisted_rule = model
        .materialized_source_shard_value(SourceShardKind::RulesRoot)
        .unwrap()["rules"][0]
        .clone();
    for (field, value) in rule.as_object().unwrap() {
        assert_eq!(&persisted_rule[field], value);
    }
    assert_eq!(persisted_rule["object_revision"], 0);
    let errors = width_errors(&query());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["objects"][0], track.uuid.to_string());
    assert!(
        errors[0]["message"]
            .as_str()
            .unwrap()
            .contains("required 500000nm")
    );
    track.width = 500000;
    let prepared = board_routing::build_set_board_track(&model, provenance(), &track).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(width_errors(&query()).is_empty());
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        cold.materialized_source_shard_value(SourceShardKind::RulesRoot)
            .unwrap()["rules"][0],
        persisted_rule
    );
    std::fs::remove_dir_all(root).unwrap();
}

/// A pre-existing legacy source baseline with an empty journal. This is a
/// compatibility reader fixture, not a legacy production writer or migration.
fn legacy_rule_compat_fixture(width_rule: bool, class_width_nm: i64) -> (PathBuf, Uuid) {
    use eda_engine::board::{Net, NetClass, Track};
    use eda_engine::ir::geometry::Point;
    use eda_engine::rules::ast::{Rule, RuleParams, RuleScope, RuleType};
    let root = unique_project_root("datum-persisted-rule-compat");
    create_native_project(&root, Some("Persisted rule compatibility".into())).unwrap();
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: format!("{class_width_nm}nm fallback class"),
        clearance: 100000,
        track_width: class_width_nm,
        via_drill: 300000,
        via_diameter: 600000,
        diffpair_width: 0,
        diffpair_gap: 0,
    };
    let a = Net::new(Uuid::new_v4(), "A", class.uuid);
    let b = Net::new(Uuid::new_v4(), "B", class.uuid);
    let track = Track::straight(
        Uuid::new_v4(),
        a.uuid,
        Point::new(0, 0),
        Point::new(1000000, 0),
        200000,
        1,
    );
    let peer = Track::straight(
        Uuid::new_v4(),
        b.uuid,
        Point::new(0, 500000),
        Point::new(1000000, 500000),
        500000,
        1,
    );
    let board_path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board_path).unwrap()).unwrap();
    board["nets"] = serde_json::json!({a.uuid.to_string():a,b.uuid.to_string():b});
    board["net_classes"] = serde_json::json!({class.uuid.to_string():class});
    board["tracks"] = serde_json::json!({track.uuid.to_string():track,peer.uuid.to_string():peer});
    std::fs::write(&board_path, to_json_deterministic(&board).unwrap()).unwrap();
    let mut rules = vec![Rule {
        uuid: Uuid::new_v4(),
        name: "Legacy clearance".into(),
        scope: RuleScope::All,
        priority: 0,
        enabled: true,
        rule_type: RuleType::ClearanceCopper,
        parameters: RuleParams::Clearance {
            min: if width_rule { 200000 } else { 100000 },
        },
    }];
    if width_rule {
        rules.push(Rule {
            uuid: Uuid::new_v4(),
            name: "Legacy authored width".into(),
            scope: RuleScope::All,
            priority: u32::MAX,
            enabled: true,
            rule_type: RuleType::TrackWidth,
            parameters: RuleParams::TrackWidth {
                min: 500000,
                preferred: 500000,
                max: 800000,
            },
        });
    }
    let rules_path = root.join("rules/rules.json");
    let mut source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&rules_path).unwrap()).unwrap();
    source["rules"] = serde_json::to_value(&rules).unwrap();
    std::fs::write(&rules_path, to_json_deterministic(&source).unwrap()).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        model.journal.is_empty(),
        "legacy baseline must not rewrite a journaled source"
    );
    assert_eq!(
        model
            .materialized_source_shard_value(SourceShardKind::RulesRoot)
            .unwrap()["rules"],
        serde_json::to_value(rules).unwrap()
    );
    (root, track.uuid)
}

fn compat_drc_query(root: &Path) -> serde_json::Value {
    let cli = Cli::try_parse_from([
        "datum-eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drc",
    ])
    .unwrap();
    serde_json::from_str(&execute(cli).unwrap()).unwrap()
}

fn compat_findings<'a>(report: &'a serde_json::Value, code: &str) -> Vec<&'a serde_json::Value> {
    report["raw_report"]["drc"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["code"] == code)
        .collect()
}

#[test]
fn persisted_legacy_rule_source_preserves_no_width_rule_class_fallback() {
    let (root, _) = legacy_rule_compat_fixture(false, 300000);
    let path = root.join("rules/rules.json");
    let before = std::fs::read(&path).unwrap();
    let report = compat_drc_query(&root);
    let widths = compat_findings(&report, "track_width_below_min");
    assert_eq!(widths.len(), 1);
    assert!(
        widths[0]["message"]
            .as_str()
            .unwrap()
            .contains("required 300000nm"),
        "class fallback must not become the weaker 100000nm default"
    );
    for code in [
        "track_width_policy_unresolved",
        "nominal_geometry_unavailable",
        "clearance_copper",
    ] {
        assert!(
            compat_findings(&report, code).is_empty(),
            "unexpected {code}"
        );
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_typed_board_width_rules_share_scope_priority_range_and_refusal() {
    use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
    let (root, track) = legacy_rule_compat_fixture(true, 100000);
    let project = load_native_project_with_resolved_board(&root).unwrap();
    let mut board = build_native_project_board(&project).unwrap();
    let net = board.tracks[&track].net;
    let class = board.nets[&net].class;
    let width = board
        .rules
        .iter()
        .position(|r| r.rule_type == RuleType::TrackWidth)
        .unwrap();
    let check =
        |board: &eda_engine::board::Board| eda_engine::drc::run(board, &[RuleType::TrackWidth]);
    for scope in [
        RuleScope::All,
        RuleScope::Net(net),
        RuleScope::NetClass(class),
        RuleScope::Layer(1),
    ] {
        board.rules[width].scope = scope;
        let report = check(&board);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.violations[0].code, "track_width_below_min");
        assert!(report.violations[0].message.contains("required 500000nm"));
    }
    for scope in [
        RuleScope::Net(Uuid::new_v4()),
        RuleScope::NetClass(Uuid::new_v4()),
        RuleScope::Layer(2),
    ] {
        board.rules[width].scope = scope;
        assert!(check(&board).passed);
    }
    board.rules[width].scope = RuleScope::All;
    board.tracks.get_mut(&track).unwrap().width = 800000;
    assert!(check(&board).passed, "authored upper bound is inclusive");
    board.tracks.get_mut(&track).unwrap().width = 900000;
    assert_eq!(check(&board).violations[0].code, "track_width_above_max");
    board.rules[width].enabled = false;
    assert!(
        check(&board).passed,
        "class fallback has no invented maximum"
    );
    board.rules[width].enabled = true;
    board.tracks.get_mut(&track).unwrap().width = 200000;
    let mut lower = board.rules[width].clone();
    lower.uuid = Uuid::new_v4();
    lower.priority = 0;
    lower.parameters = RuleParams::TrackWidth {
        min: 100000,
        preferred: 100000,
        max: 1000000,
    };
    board.rules.push(lower);
    assert!(
        check(&board).violations[0]
            .message
            .contains("required 500000nm")
    );
    let mut tie = board.rules[width].clone();
    tie.uuid = Uuid::new_v4();
    board.rules.push(tie);
    let report = check(&board);
    assert!(
        report
            .violations
            .iter()
            .all(|f| f.code == "track_width_policy_unresolved")
    );
    assert!(
        !report.passed,
        "ambiguous policy cannot fall back to the class"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn persisted_legacy_and_native_rules_share_meaning_without_source_rewrite_or_fallback() {
    use eda_engine::api::native_write::{
        WriteProvenance, commit_prepared,
        project::{build_create_project_rule, build_set_project_rule},
    };
    use eda_engine::substrate::CommitSource;
    let (root, _) = legacy_rule_compat_fixture(true, 100000);
    let path = root.join("rules/rules.json");
    let legacy_bytes = std::fs::read(&path).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let legacy = model
        .materialized_source_shard_value(SourceShardKind::RulesRoot)
        .unwrap()["rules"]
        .clone();
    let report = compat_drc_query(&root);
    assert_eq!(compat_findings(&report, "track_width_below_min").len(), 1);
    assert!(
        compat_findings(&report, "track_width_below_min")[0]["message"]
            .as_str()
            .unwrap()
            .contains("required 500000nm")
    );
    assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
    assert!(compat_findings(&report, "nominal_geometry_unavailable").is_empty());
    assert_eq!(
        compat_findings(&report, "clearance_copper").len(),
        1,
        "legacy clearance200k must govern the150k gap even before any native rule exists"
    );
    assert_eq!(std::fs::read(&path).unwrap(), legacy_bytes);
    let provenance = || {
        WriteProvenance::new(
            "rule-compat-test",
            CommitSource::Test,
            "Canonical native rule over persisted legacy baseline",
        )
    };
    // Full native priority must beat legacy u32::MAX, including when its
    // explicit width is weaker. Board.rules contains only the legacy500k rule.
    for rule in [
        serde_json::json!({"uuid":Uuid::new_v4(),"name":"Native width 300000","scope":"All","priority":u64::MAX-1,"enabled":true,"rule_type":"track_width","params":{"min_nm":300000,"preferred_nm":300000,"max_nm":800000}}),
        serde_json::json!({"uuid":Uuid::new_v4(),"name":"Native clearance 200000","scope":"All","priority":1,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":200000}}),
    ] {
        model = ProjectResolver::new(&root).resolve().unwrap();
        let (prepared, _) = build_create_project_rule(&model, provenance(), rule).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let report = compat_drc_query(&root);
    let widths = compat_findings(&report, "track_width_below_min");
    assert_eq!(widths.len(), 1);
    assert!(
        widths[0]["message"]
            .as_str()
            .unwrap()
            .contains("required 300000nm")
    );
    assert_eq!(compat_findings(&report, "clearance_copper").len(), 1);
    assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
    assert!(compat_findings(&report, "nominal_geometry_unavailable").is_empty());
    let mut unsupported = serde_json::json!({"uuid":Uuid::new_v4(),"name":"Unsupported width scope","scope":"All","priority":u64::MAX-1,"enabled":true,"rule_type":"track_width","params":{"min_nm":100000,"preferred_nm":100000,"max_nm":800000}});
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) =
        build_create_project_rule(&model, provenance(), unsupported.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let tied = compat_drc_query(&root);
    assert!(
        compat_findings(&tied, "track_width_policy_unresolved")
            .iter()
            .any(|f| f["message"].as_str().unwrap().contains("ambiguous"))
    );
    assert!(compat_findings(&tied, "track_width_below_min").is_empty());
    unsupported["scope"] = serde_json::json!({"And":["All",{"Layer":1}]});
    unsupported["priority"] = serde_json::json!(u64::MAX);
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) = build_set_project_rule(&model, provenance(), unsupported.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();

    let report = compat_drc_query(&root);
    assert!(!compat_findings(&report, "track_width_policy_unresolved").is_empty());
    assert!(
        compat_findings(&report, "track_width_below_min").is_empty(),
        "unresolvable model policy must never use the legacy Board500k fallback"
    );
    assert!(
        compat_findings(&report, "track_width_policy_unresolved")
            .iter()
            .any(|f| f["message"].as_str().unwrap().contains("And"))
    );
    unsupported["enabled"] = serde_json::json!(false);
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) = build_set_project_rule(&model, provenance(), unsupported).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let report = compat_drc_query(&root);
    assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
    assert!(
        compat_findings(&report, "track_width_below_min")[0]["message"]
            .as_str()
            .unwrap()
            .contains("required 300000nm")
    );
    let persisted = model
        .materialized_source_shard_value(SourceShardKind::RulesRoot)
        .unwrap();
    assert_eq!(persisted["rules"][0], legacy[0]);
    assert_eq!(persisted["rules"][1], legacy[1]);
    let bytes = std::fs::read(&path).unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        cold.materialized_source_shard_value(SourceShardKind::RulesRoot)
            .unwrap(),
        persisted
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_native_width_scopes_disabled_and_bounds_preserve_actual_rule_meaning() {
    use eda_engine::api::native_write::{
        WriteProvenance,
        board_routing::build_set_board_track,
        commit_prepared,
        project::{build_create_project_rule, build_set_project_rule},
    };
    use eda_engine::board::Track;
    use eda_engine::substrate::CommitSource;
    let (root, track_id) = legacy_rule_compat_fixture(false, 100000);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut track: Track =
        serde_json::from_value(board["tracks"][track_id.to_string()].clone()).unwrap();
    let class = board["nets"][track.net.to_string()]["class"].clone();
    let provenance = || {
        WriteProvenance::new(
            "native-width-scope-test",
            CommitSource::Test,
            "Canonical native width scope and range operation",
        )
    };
    let mut rule = serde_json::json!({"uuid":Uuid::new_v4(),"name":"Scoped native width","scope":"All","priority":u64::MAX-1,"enabled":true,"rule_type":"track_width","params":{"min_nm":500000,"preferred_nm":500000,"max_nm":800000}});
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) = build_create_project_rule(&model, provenance(), rule.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    for scope in [
        serde_json::json!("All"),
        serde_json::json!({"Net":track.net}),
        serde_json::json!({"NetClass":class}),
        serde_json::json!({"Layer":1}),
    ] {
        rule["scope"] = scope;
        model = ProjectResolver::new(&root).resolve().unwrap();
        let (prepared, _) = build_set_project_rule(&model, provenance(), rule.clone()).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        let report = compat_drc_query(&root);
        let widths = compat_findings(&report, "track_width_below_min");
        assert_eq!(widths.len(), 1);
        assert!(
            widths[0]["message"]
                .as_str()
                .unwrap()
                .contains("required 500000nm")
        );
        assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
    }
    for scope in [
        serde_json::json!({"Net":Uuid::new_v4()}),
        serde_json::json!({"NetClass":Uuid::new_v4()}),
        serde_json::json!({"Layer":2}),
    ] {
        rule["scope"] = scope;
        model = ProjectResolver::new(&root).resolve().unwrap();
        let (prepared, _) = build_set_project_rule(&model, provenance(), rule.clone()).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        let report = compat_drc_query(&root);
        assert!(compat_findings(&report, "track_width_below_min").is_empty());
        assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
    }
    rule["enabled"] = serde_json::json!(false);
    rule["scope"] = serde_json::json!({"And":["All",{"Layer":1}]});
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) = build_set_project_rule(&model, provenance(), rule.clone()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let report = compat_drc_query(&root);
    assert!(compat_findings(&report, "track_width_below_min").is_empty());
    assert!(
        compat_findings(&report, "track_width_policy_unresolved").is_empty(),
        "disabled unsupported scope is not evaluated"
    );
    rule["enabled"] = serde_json::json!(true);
    rule["scope"] = serde_json::json!("All");
    model = ProjectResolver::new(&root).resolve().unwrap();
    let (prepared, _) = build_set_project_rule(&model, provenance(), rule).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    // Keep clearance independent while changing the tested track's width.
    let mut peer: Track = serde_json::from_value(
        board["tracks"]
            .as_object()
            .unwrap()
            .values()
            .find(|v| v["uuid"] != track_id.to_string())
            .unwrap()
            .clone(),
    )
    .unwrap();
    peer.from.y = 1000000;
    peer.to.y = 1000000;
    model = ProjectResolver::new(&root).resolve().unwrap();
    let prepared = build_set_board_track(&model, provenance(), &peer).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    for (width, expected) in [
        (500000, None),
        (800000, None),
        (800001, Some("track_width_above_max")),
        (499999, Some("track_width_below_min")),
    ] {
        track.width = width;
        model = ProjectResolver::new(&root).resolve().unwrap();
        let prepared = build_set_board_track(&model, provenance(), &track).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        let report = compat_drc_query(&root);
        assert!(compat_findings(&report, "track_width_policy_unresolved").is_empty());
        for code in ["track_width_below_min", "track_width_above_max"] {
            assert_eq!(
                compat_findings(&report, code).len(),
                usize::from(expected == Some(code))
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn captured_model_width_refuses_out_of_journal_rule_and_board_changes() {
    use eda_engine::rules::ast::{RuleParams, RuleType};
    let (root, track) = legacy_rule_compat_fixture(true, 100000);
    let (project, model) = load_native_project_with_resolved_board_and_model(&root).unwrap();
    assert!(model.journal.is_empty());
    let mut board = build_native_project_board_with_model(&project, Some(&model)).unwrap();
    let width = board
        .rules
        .iter()
        .position(|r| r.rule_type == RuleType::TrackWidth)
        .unwrap();
    // The model must never substitute a weaker typed projection for its source.
    board.rules[width].parameters = RuleParams::TrackWidth {
        min: 100000,
        preferred: 100000,
        max: 800000,
    };
    let check = |board: &eda_engine::board::Board| {
        eda_engine::drc::run_with_current_zone_fills_and_waivers(
            board,
            &[RuleType::TrackWidth],
            &model,
            &[],
        )
    };
    let before = check(&board);
    assert_eq!(before.violations.len(), 1);
    assert!(before.violations[0].message.contains("required 500000nm"));
    let rules_path = root.join("rules/rules.json");
    let rule_bytes = std::fs::read(&rules_path).unwrap();
    let mut changed: serde_json::Value = serde_json::from_slice(&rule_bytes).unwrap();
    changed["rules"][1]["parameters"]["TrackWidth"]["min"] = serde_json::json!(100000);
    changed["rules"][1]["parameters"]["TrackWidth"]["preferred"] = serde_json::json!(100000);
    // Deliberate external-file corruption is a negative control over an empty
    // journal baseline, not a committed change or a production writing path.
    std::fs::write(&rules_path, to_json_deterministic(&changed).unwrap()).unwrap();
    let stale = check(&board);
    assert!(!stale.passed);
    assert!(
        stale
            .violations
            .iter()
            .all(|f| f.code == "track_width_policy_unresolved")
    );
    assert!(
        stale.violations[0]
            .message
            .contains("stale captured source basis")
    );
    std::fs::write(&rules_path, &rule_bytes).unwrap();
    assert!(
        check(&board).violations[0]
            .message
            .contains("required 500000nm")
    );
    let board_path = root.join("board/board.json");
    let board_bytes = std::fs::read(&board_path).unwrap();
    let mut changed: serde_json::Value = serde_json::from_slice(&board_bytes).unwrap();
    changed["tracks"][track.to_string()]["width"] = serde_json::json!(500000);
    board.tracks.get_mut(&track).unwrap().width = 500000;
    std::fs::write(&board_path, to_json_deterministic(&changed).unwrap()).unwrap();
    let stale = check(&board);
    assert!(
        !stale.passed,
        "a matching forged Board must not certify an old model"
    );
    assert!(
        stale
            .violations
            .iter()
            .all(|f| f.code == "track_width_policy_unresolved")
    );
    assert!(
        stale.violations[0]
            .message
            .contains("stale captured source basis")
    );
    std::fs::write(&board_path, &board_bytes).unwrap();
    board.tracks.get_mut(&track).unwrap().width = 200000;
    assert!(
        check(&board).violations[0]
            .message
            .contains("required 500000nm")
    );
    assert_eq!(std::fs::read(rules_path).unwrap(), rule_bytes);
    assert_eq!(std::fs::read(board_path).unwrap(), board_bytes);
    std::fs::remove_dir_all(root).unwrap();
}
