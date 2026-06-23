use super::*;
use eda_engine::board::BoardText;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_text_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-texts",
    ])
    .expect("CLI should parse")
}

fn journal_list(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    serde_json::from_str(&output).expect("journal-list JSON should parse")
}

#[test]
fn project_board_text_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-text");
    create_native_project(&root, Some("Board Text Demo".to_string()))
        .expect("initial scaffold should succeed");

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-text",
        root.to_str().unwrap(),
        "--text",
        "PCB TOP",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--rotation-deg",
        "90",
        "--render-intent",
        "annotation",
        "--style",
        "regular",
        "--style-class",
        "fab-note",
        "--h-align",
        "center",
        "--v-align",
        "top",
        "--mirrored",
        "--keep-upright",
        "--line-spacing-ratio-ppm",
        "1350000",
        "--bold",
        "--italic",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board text should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let text_uuid = placed["text_uuid"].as_str().unwrap().to_string();

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].uuid.to_string(), text_uuid);
    assert_eq!(texts[0].text, "PCB TOP");
    assert_eq!(texts[0].position.x, 1000);
    assert_eq!(texts[0].position.y, 2000);
    assert_eq!(texts[0].rotation, 90);
    assert_eq!(texts[0].height_nm, 1_000_000);
    assert_eq!(texts[0].stroke_width_nm, 100_000);
    assert_eq!(
        texts[0].render_intent,
        eda_engine::text::TextRenderIntent::Annotation
    );
    assert_eq!(
        texts[0].family_source,
        eda_engine::text::TextFamilySource::ImplicitDefault
    );
    assert_eq!(texts[0].style.0, "regular");
    assert_eq!(texts[0].style_class.as_deref(), Some("fab-note"));
    assert_eq!(texts[0].h_align, eda_engine::text::TextHAlign::Center);
    assert_eq!(texts[0].v_align, eda_engine::text::TextVAlign::Top);
    assert!(texts[0].mirrored);
    assert!(texts[0].keep_upright);
    assert_eq!(texts[0].line_spacing_ratio_ppm, 1_350_000);
    assert!(texts[0].bold);
    assert!(texts[0].italic);
    assert_eq!(texts[0].layer, 1);
    let journal = journal_list(&root);
    assert_eq!(journal["transactions"][0]["reason"], "place board text");

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
        "--value",
        "PCB BOT",
        "--x-nm",
        "3000",
        "--y-nm",
        "4000",
        "--rotation-deg",
        "180",
        "--height-nm",
        "1200000",
        "--stroke-width-nm",
        "150000",
        "--family",
        "jetbrains_mono",
        "--style",
        "regular",
        "--style-class",
        "reference-label",
        "--h-align",
        "right",
        "--v-align",
        "bottom",
        "--mirrored",
        "false",
        "--keep-upright",
        "false",
        "--line-spacing-ratio-ppm",
        "900000",
        "--bold",
        "false",
        "--italic",
        "false",
        "--layer",
        "2",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board text should succeed");

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "PCB BOT");
    assert_eq!(texts[0].position.x, 3000);
    assert_eq!(texts[0].position.y, 4000);
    assert_eq!(texts[0].rotation, 180);
    assert_eq!(texts[0].height_nm, 1_200_000);
    assert_eq!(texts[0].stroke_width_nm, 150_000);
    assert_eq!(
        texts[0].render_intent,
        eda_engine::text::TextRenderIntent::Annotation
    );
    assert_eq!(texts[0].family.0, "jetbrains_mono");
    assert_eq!(
        texts[0].family_source,
        eda_engine::text::TextFamilySource::Explicit
    );
    assert_eq!(texts[0].style.0, "regular");
    assert_eq!(texts[0].style_class.as_deref(), Some("reference-label"));
    assert_eq!(texts[0].h_align, eda_engine::text::TextHAlign::Right);
    assert_eq!(texts[0].v_align, eda_engine::text::TextVAlign::Bottom);
    assert!(!texts[0].mirrored);
    assert!(!texts[0].keep_upright);
    assert_eq!(texts[0].line_spacing_ratio_ppm, 900_000);
    assert!(!texts[0].bold);
    assert!(!texts[0].italic);
    assert_eq!(texts[0].layer, 2);
    let journal = journal_list(&root);
    assert_eq!(journal["transactions"][1]["reason"], "edit board text");

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board text should succeed");
    assert!(delete_output.contains("action: delete_board_text"));

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert!(texts.is_empty());
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 3);
    assert_eq!(journal["transactions"][2]["reason"], "delete board text");

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_texts: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_texts_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-text-query");
    create_native_project(&root, Some("Board Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": Uuid::new_v4(),
                    "text": "FAB",
                    "position": { "x": 10, "y": 20 },
                    "rotation": 0,
                    "height_nm": 900000,
                    "stroke_width_nm": 120000,
                    "layer": 21
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "FAB");
    assert_eq!(texts[0].height_nm, 900_000);
    assert_eq!(texts[0].stroke_width_nm, 120_000);
    assert_eq!(
        texts[0].family_source,
        eda_engine::text::TextFamilySource::ImplicitDefault
    );
    assert_eq!(texts[0].h_align, eda_engine::text::TextHAlign::Left);
    assert_eq!(texts[0].v_align, eda_engine::text::TextVAlign::Bottom);
    assert!(!texts[0].mirrored);
    assert!(!texts[0].keep_upright);
    assert_eq!(texts[0].line_spacing_ratio_ppm, 1_000_000);
    assert_eq!(texts[0].layer, 21);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_board_text_rejects_invalid_semantic_controls() {
    let root = unique_project_root("datum-eda-cli-project-board-text-invalid");
    create_native_project(&root, Some("Board Text Invalid Demo".to_string()))
        .expect("initial scaffold should succeed");

    let invalid_align_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-text",
        root.to_str().unwrap(),
        "--text",
        "BAD ALIGN",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--h-align",
        "middle",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let err = execute(invalid_align_cli).expect_err("invalid alignment should fail");
    assert!(err.to_string().contains("horizontal alignment"));

    let invalid_spacing_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-text",
        root.to_str().unwrap(),
        "--text",
        "BAD SPACING",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--line-spacing-ratio-ppm",
        "0",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let err = execute(invalid_spacing_cli).expect_err("invalid line spacing should fail");
    assert!(err.to_string().contains("line spacing ratio"));

    let _ = std::fs::remove_dir_all(&root);
}
