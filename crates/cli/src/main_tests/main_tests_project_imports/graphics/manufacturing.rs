use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance,
        board_annotations::graphics::{build_create_board_graphic, build_set_board_graphic},
        commit_prepared,
    },
    board::graphics::{BoardGraphic, GraphicGeometry as G},
    ir::geometry::Point,
    substrate::CommitSource,
};
fn p(x: i64, y: i64) -> Point {
    Point::new(x * 1_000_000, y * 1_000_000)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "graphics-manufacturing",
        CommitSource::Test,
        "PM057 exact native graphic output",
    )
}
fn sources(model: &eda_engine::substrate::DesignModel) -> Vec<BoardGraphic> {
    let v = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    serde_json::from_value::<std::collections::BTreeMap<Uuid, BoardGraphic>>(v["graphics"].clone())
        .unwrap()
        .into_values()
        .collect()
}
fn command(
    root: &Path,
    verb: &str,
    layer: &str,
    path: &Path,
    flag: &str,
) -> Result<serde_json::Value> {
    let out = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            verb,
            root.to_str().unwrap(),
            "--layer",
            layer,
            flag,
            path.to_str().unwrap(),
        ])
        .unwrap(),
    )?;
    Ok(serde_json::from_str(&out).unwrap())
}
#[test]
fn native_graphic_manufacturing_preserves_holes_arcs_and_source_bytes() {
    let (root, source) = setup();
    import(&root.0, &source).unwrap();
    let mut model = ProjectResolver::new(&root.0).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let id: Uuid = serde_json::from_value(board["uuid"].clone()).unwrap();
    let mut polygon = sources(&model)
        .into_iter()
        .find(|g| matches!(g.geometry, G::Polygon { .. }))
        .unwrap();
    polygon.geometry = G::Polygon {
        outer: vec![p(0, 0), p(10, 0), p(10, 10), p(0, 10)],
        holes: vec![vec![p(2, 2), p(2, 4), p(4, 4), p(4, 2)]],
        filled: true,
    };
    polygon.width = 100001;
    let w = build_set_board_graphic(&model, provenance(), id, &polygon).unwrap();
    commit_prepared(&mut model, &root.0, w).unwrap();
    let line = BoardGraphic {
        uuid: Uuid::from_u128(999),
        layer: 37,
        width: 100001,
        geometry: G::Polyline {
            points: vec![p(0, 0), p(1, 2), p(2, 0)],
        },
        import_key: None,
    };
    let w = build_create_board_graphic(&model, provenance(), id, &line).unwrap();
    commit_prepared(&mut model, &root.0, w).unwrap();
    for (layer, family) in [(37, "silkscreen"), (49, "mechanical")] {
        if layer == 49 {
            for mut graphic in sources(&model) {
                graphic.layer = 49;
                let w = build_set_board_graphic(&model, provenance(), id, &graphic).unwrap();
                commit_prepared(&mut model, &root.0, w).unwrap();
            }
        }
        let before: Vec<_> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let journal = model.journal.len();
        let path = root.0.join(format!("{family}.gbr"));
        let layer = layer.to_string();
        let report = command(
            &root.0,
            &format!("export-gerber-{family}-layer"),
            &layer,
            &path,
            "--out",
        )
        .unwrap();
        assert_eq!(report["board_graphic_count"], 6);
        let panel = eda_engine::substrate::PanelProjection {
            schema_version: 1,
            id: Uuid::from_u128(1000),
            name: "graphics panel".into(),
            object_revision: eda_engine::substrate::ObjectRevision(0),
            board_instances: vec![
                eda_engine::substrate::PanelBoardInstance {
                    board: id,
                    x_nm: 0,
                    y_nm: 0,
                    rotation_deg: 0,
                },
                eda_engine::substrate::PanelBoardInstance {
                    board: id,
                    x_nm: 20_000_000,
                    y_nm: 30_000_000,
                    rotation_deg: 0,
                },
            ],
        };
        let board_output = std::fs::read_to_string(&path).unwrap();
        let panel_output = panelize_rs274x_gerber(&root.0, &board_output, &panel).unwrap();
        let panel_parsed = parse_rs274x_subset(&panel_output).unwrap();
        assert_eq!(
            panel_parsed.geometries.len(),
            2 * parse_rs274x_subset(&board_output).unwrap().geometries.len()
        );
        assert_eq!(panel_output.matches("%AM").count(), 1);
        assert!(panel_parsed.geometries.iter().any(|g| matches!(g,ParsedGerberGeometry::Arc{from,center_offset,..} if from.x==20_000_000 && from.y==30_000_000 && center_offset.x==1_000_000 && center_offset.y==0)));
        let report = command(
            &root.0,
            &format!("validate-gerber-{family}-layer"),
            &layer,
            &path,
            "--gerber",
        )
        .unwrap();
        assert_eq!(report["matches_expected"], true);
        let report = command(
            &root.0,
            &format!("compare-gerber-{family}-layer"),
            &layer,
            &path,
            "--gerber",
        )
        .unwrap();
        assert_eq!(report["missing_count"], 0);
        assert_eq!(report["extra_count"], 0);
        let bytes = std::fs::read(&path).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        let parsed = parse_rs274x_subset(text).unwrap();
        assert!(parsed.geometries.iter().any(|g|matches!(g,ParsedGerberGeometry::Arc{center_offset,..} if center_offset.x==1_000_000 && center_offset.y==0)));
        assert!(parsed.geometries.iter().any(|g|matches!(g,ParsedGerberGeometry::Flash{aperture:ParsedGerberAperture::Polygon{holes,..},..} if holes[0]==vec![p(2,2),p(2,4),p(4,4),p(4,2)])));
        std::fs::write(
            &path,
            text.replace("C,10.100000X9.900000", "C,10.100000X9.800000"),
        )
        .unwrap();
        let report = command(
            &root.0,
            &format!("compare-gerber-{family}-layer"),
            &layer,
            &path,
            "--gerber",
        )
        .unwrap();
        assert!(report["missing_count"].as_u64().unwrap() > 0);
        assert!(report["extra_count"].as_u64().unwrap() > 0);
        let after = ProjectResolver::new(&root.0).resolve().unwrap();
        assert_eq!(after.journal.len(), journal);
        for (path, bytes) in before {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
    let report = validate_native_project(&root.0).unwrap();
    assert!(report.schema_compatible);
}
#[test]
fn rational_centre_manufacturing_refusal_preserves_existing_artifact() {
    let (root, source) = setup();
    import(&root.0, &source).unwrap();
    let mut model = ProjectResolver::new(&root.0).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let id = serde_json::from_value(board["uuid"].clone()).unwrap();
    let mut arc = sources(&model)
        .into_iter()
        .find(|g| matches!(g.geometry, G::Arc { .. }))
        .unwrap();
    arc.geometry = G::Arc {
        from: p(0, 0),
        midpoint: p(1, 3),
        to: p(3, 0),
    };
    let w = build_set_board_graphic(&model, provenance(), id, &arc).unwrap();
    commit_prepared(&mut model, &root.0, w).unwrap();
    let path = root.0.join("pending.gbr");
    std::fs::write(&path, b"previous artifact").unwrap();
    let error = command(
        &root.0,
        "export-gerber-silkscreen-layer",
        "37",
        &path,
        "--out",
    )
    .unwrap_err();
    let error = format!("{error:#}");
    assert!(error.contains("CAM/T04"), "{error}");
    assert!(error.contains(&arc.uuid.to_string()), "{error}");
    assert_eq!(std::fs::read(&path).unwrap(), b"previous artifact");
    assert_eq!(
        ProjectResolver::new(&root.0)
            .resolve()
            .unwrap()
            .journal
            .len(),
        model.journal.len()
    );
}
