use super::*;
use eda_engine::board::Track;
use eda_engine::ir::serialization::to_json_deterministic;

pub(crate) fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

pub(crate) fn seed_plus_one_gap_project(root: &Path) -> (Uuid, Uuid, Uuid, PathBuf) {
    create_native_project(root, Some("Route Proposal Artifact Demo".to_string()))
        .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xb200);
    let class_uuid = Uuid::from_u128(0xb201);
    let package_a_uuid = Uuid::from_u128(0xb202);
    let package_b_uuid = Uuid::from_u128(0xb203);
    let anchor_a_uuid = Uuid::from_u128(0xb204);
    let anchor_b_uuid = Uuid::from_u128(0xb205);
    let track_a_uuid = Uuid::from_u128(0xb206);
    let track_b_uuid = Uuid::from_u128(0xb207);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xb208),
                "name": "Route Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 2000000, "y": 0 },
                        { "x": 2000000, "y": 1000000 },
                        { "x": 0, "y": 1000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 100000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 1900000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    track_a_uuid.to_string(): {
                        "uuid": track_a_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 100000, "y": 500000 },
                        "to": { "x": 700000, "y": 500000 },
                        "width": 150000,
                        "layer": 1
                    },
                    track_b_uuid.to_string(): {
                        "uuid": track_b_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 1300000, "y": 500000 },
                        "to": { "x": 1900000, "y": 500000 },
                        "width": 150000,
                        "layer": 1
                    }
                },
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 150000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid, board_json)
}

pub(crate) fn board_tracks_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-tracks",
    ])
    .expect("CLI should parse")
}

fn plus_one_gap_query_cli(root: &Path, net_uuid: Uuid, from_anchor: Uuid, to_anchor: Uuid) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "route-path-candidate-authored-copper-plus-one-gap",
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor.to_string(),
        "--to-anchor",
        &to_anchor.to_string(),
    ])
    .expect("CLI should parse")
}

pub(crate) fn seed_route_path_candidate_project(root: &Path) -> (Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xc200);
    let other_net_uuid = Uuid::from_u128(0xc201);
    let class_uuid = Uuid::from_u128(0xc202);
    let package_a_uuid = Uuid::from_u128(0xc203);
    let package_b_uuid = Uuid::from_u128(0xc204);
    let anchor_a_uuid = Uuid::from_u128(0xc205);
    let anchor_b_uuid = Uuid::from_u128(0xc206);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xc207),
                "name": "Route Path Candidate Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    },
                    other_net_uuid.to_string(): {
                        "uuid": other_net_uuid,
                        "name": "GND",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid)
}

pub(crate) fn seed_route_path_candidate_orthogonal_dogleg_project(root: &Path) -> (Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Orthogonal Dogleg Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xc300);
    let other_net_uuid = Uuid::from_u128(0xc301);
    let class_uuid = Uuid::from_u128(0xc302);
    let package_a_uuid = Uuid::from_u128(0xc303);
    let package_b_uuid = Uuid::from_u128(0xc304);
    let anchor_a_uuid = Uuid::from_u128(0xc305);
    let anchor_b_uuid = Uuid::from_u128(0xc306);
    let blocking_track_uuid = Uuid::from_u128(0xc307);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xc309),
                "name": "Route Path Candidate Orthogonal Dogleg Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000000, "y": 0 },
                        { "x": 1000000, "y": 1000000 },
                        { "x": 0, "y": 1000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 100000, "y": 100000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 900000, "y": 900000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    blocking_track_uuid.to_string(): {
                        "uuid": blocking_track_uuid,
                        "net": other_net_uuid,
                        "from": { "x": 900000, "y": 300000 },
                        "to": { "x": 900000, "y": 700000 },
                        "width": 150000,
                        "layer": 1
                    }
                },
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    },
                    other_net_uuid.to_string(): {
                        "uuid": other_net_uuid,
                        "name": "OTHER",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid)
}

pub(crate) fn seed_route_path_candidate_orthogonal_two_bend_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Orthogonal Two Bend Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xc400);
    let other_net_uuid = Uuid::from_u128(0xc401);
    let class_uuid = Uuid::from_u128(0xc402);
    let package_a_uuid = Uuid::from_u128(0xc403);
    let package_b_uuid = Uuid::from_u128(0xc404);
    let anchor_a_uuid = Uuid::from_u128(0xc405);
    let anchor_b_uuid = Uuid::from_u128(0xc406);
    let left_block_uuid = Uuid::from_u128(0xc407);
    let right_block_uuid = Uuid::from_u128(0xc408);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xc409),
                "name": "Route Path Candidate Orthogonal Two Bend Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000000, "y": 0 },
                        { "x": 1000000, "y": 1000000 },
                        { "x": 0, "y": 1000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 100000, "y": 100000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 900000, "y": 900000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    left_block_uuid.to_string(): {
                        "uuid": left_block_uuid,
                        "net": other_net_uuid,
                        "from": { "x": 100000, "y": 300000 },
                        "to": { "x": 100000, "y": 700000 },
                        "width": 150000,
                        "layer": 1
                    },
                    right_block_uuid.to_string(): {
                        "uuid": right_block_uuid,
                        "net": other_net_uuid,
                        "from": { "x": 900000, "y": 300000 },
                        "to": { "x": 900000, "y": 700000 },
                        "width": 150000,
                        "layer": 1
                    }
                },
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    },
                    other_net_uuid.to_string(): {
                        "uuid": other_net_uuid,
                        "name": "OTHER",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid)
}

pub(crate) fn seed_route_path_candidate_via_project(root: &Path) -> (Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xd200);
    let class_uuid = Uuid::from_u128(0xd201);
    let package_a_uuid = Uuid::from_u128(0xd202);
    let package_b_uuid = Uuid::from_u128(0xd203);
    let anchor_a_uuid = Uuid::from_u128(0xd204);
    let anchor_b_uuid = Uuid::from_u128(0xd205);
    let via_uuid = Uuid::from_u128(0xd206);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xd207),
                "name": "Route Path Candidate Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 3,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": target_net_uuid,
                        "position": { "x": 2500000, "y": 1500000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 3
                    }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_uuid)
}

pub(crate) fn seed_route_path_candidate_two_via_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Two Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xe200);
    let class_uuid = Uuid::from_u128(0xe201);
    let package_a_uuid = Uuid::from_u128(0xe202);
    let package_b_uuid = Uuid::from_u128(0xe203);
    let anchor_a_uuid = Uuid::from_u128(0xe204);
    let anchor_b_uuid = Uuid::from_u128(0xe205);
    let via_a_uuid = Uuid::from_u128(0xe206);
    let via_b_uuid = Uuid::from_u128(0xe207);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xe208),
                "name": "Route Path Candidate Two Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 5,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": target_net_uuid,
                        "position": { "x": 1500000, "y": 900000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 3
                    },
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": target_net_uuid,
                        "position": { "x": 3500000, "y": 1800000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 3,
                        "to_layer": 5
                    }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_three_via_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Three Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xf30);
    let class_uuid = Uuid::from_u128(0xf31);
    let package_a_uuid = Uuid::from_u128(0xf32);
    let package_b_uuid = Uuid::from_u128(0xf33);
    let anchor_a_uuid = Uuid::from_u128(0xf34);
    let anchor_b_uuid = Uuid::from_u128(0xf35);
    let via_a_uuid = Uuid::from_u128(0xf36);
    let via_b_uuid = Uuid::from_u128(0xf37);
    let via_c_uuid = Uuid::from_u128(0xf38);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xf39),
                "name": "Route Path Candidate Three Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 2400000 }, "layer": 7, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 1200000, "y": 900000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 2500000, "y": 1500000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 3800000, "y": 2100000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_four_via_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Four Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xf90);
    let class_uuid = Uuid::from_u128(0xf91);
    let package_a_uuid = Uuid::from_u128(0xf92);
    let package_b_uuid = Uuid::from_u128(0xf93);
    let anchor_a_uuid = Uuid::from_u128(0xf94);
    let anchor_b_uuid = Uuid::from_u128(0xf95);
    let via_a_uuid = Uuid::from_u128(0xf96);
    let via_b_uuid = Uuid::from_u128(0xf97);
    let via_c_uuid = Uuid::from_u128(0xf98);
    let via_d_uuid = Uuid::from_u128(0xf99);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xf9a),
                "name": "Route Path Candidate Four Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Inner 3", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 8, "name": "Core D", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 9, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 2400000 }, "layer": 9, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 1100000, "y": 850000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 2000000, "y": 1200000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 3000000, "y": 1700000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 },
                    via_d_uuid.to_string(): { "uuid": via_d_uuid, "net": target_net_uuid, "position": { "x": 3900000, "y": 2100000 }, "drill": 300000, "diameter": 600000, "from_layer": 7, "to_layer": 9 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_five_via_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Five Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xff0);
    let class_uuid = Uuid::from_u128(0xff1);
    let package_a_uuid = Uuid::from_u128(0xff2);
    let package_b_uuid = Uuid::from_u128(0xff3);
    let anchor_a_uuid = Uuid::from_u128(0xff4);
    let anchor_b_uuid = Uuid::from_u128(0xff5);
    let via_a_uuid = Uuid::from_u128(0xff6);
    let via_b_uuid = Uuid::from_u128(0xff7);
    let via_c_uuid = Uuid::from_u128(0xff8);
    let via_d_uuid = Uuid::from_u128(0xff9);
    let via_e_uuid = Uuid::from_u128(0xffa);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xffb),
                "name": "Route Path Candidate Five Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Inner 3", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 8, "name": "Core D", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 9, "name": "Inner 4", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 10, "name": "Core E", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 11, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 2400000 }, "layer": 11, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 1000000, "y": 800000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 1700000, "y": 1100000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 2400000, "y": 1400000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 },
                    via_d_uuid.to_string(): { "uuid": via_d_uuid, "net": target_net_uuid, "position": { "x": 3100000, "y": 1800000 }, "drill": 300000, "diameter": 600000, "from_layer": 7, "to_layer": 9 },
                    via_e_uuid.to_string(): { "uuid": via_e_uuid, "net": target_net_uuid, "position": { "x": 3800000, "y": 2100000 }, "drill": 300000, "diameter": 600000, "from_layer": 9, "to_layer": 11 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_six_via_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Six Via Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x1050);
    let class_uuid = Uuid::from_u128(0x1051);
    let package_a_uuid = Uuid::from_u128(0x1052);
    let package_b_uuid = Uuid::from_u128(0x1053);
    let anchor_a_uuid = Uuid::from_u128(0x1054);
    let anchor_b_uuid = Uuid::from_u128(0x1055);
    let via_a_uuid = Uuid::from_u128(0x1056);
    let via_b_uuid = Uuid::from_u128(0x1057);
    let via_c_uuid = Uuid::from_u128(0x1058);
    let via_d_uuid = Uuid::from_u128(0x1059);
    let via_e_uuid = Uuid::from_u128(0x105a);
    let via_f_uuid = Uuid::from_u128(0x105b);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x105c),
                "name": "Route Path Candidate Six Via Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Inner 3", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 8, "name": "Core D", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 9, "name": "Inner 4", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 10, "name": "Core E", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 11, "name": "Inner 5", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 12, "name": "Core F", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 13, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 2400000 }, "layer": 13, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 900000, "y": 760000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 1450000, "y": 980000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 2050000, "y": 1230000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 },
                    via_d_uuid.to_string(): { "uuid": via_d_uuid, "net": target_net_uuid, "position": { "x": 2700000, "y": 1550000 }, "drill": 300000, "diameter": 600000, "from_layer": 7, "to_layer": 9 },
                    via_e_uuid.to_string(): { "uuid": via_e_uuid, "net": target_net_uuid, "position": { "x": 3350000, "y": 1880000 }, "drill": 300000, "diameter": 600000, "from_layer": 9, "to_layer": 11 },
                    via_f_uuid.to_string(): { "uuid": via_f_uuid, "net": target_net_uuid, "position": { "x": 3950000, "y": 2140000 }, "drill": 300000, "diameter": 600000, "from_layer": 11, "to_layer": 13 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_authored_via_chain_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some("Route Path Candidate Authored Via Chain Proposal Artifact Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x3050);
    let class_uuid = Uuid::from_u128(0x3051);
    let package_a_uuid = Uuid::from_u128(0x3052);
    let package_b_uuid = Uuid::from_u128(0x3053);
    let anchor_a_uuid = Uuid::from_u128(0x3054);
    let anchor_b_uuid = Uuid::from_u128(0x3055);
    let via_a_uuid = Uuid::from_u128(0x3056);
    let via_b_uuid = Uuid::from_u128(0x3057);
    let via_c_uuid = Uuid::from_u128(0x3058);
    let via_d_uuid = Uuid::from_u128(0x3059);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x305a),
                "name": "Route Path Candidate Authored Via Chain Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 2400000 }, "layer": 7, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 1300000, "y": 900000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 2600000, "y": 1450000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 7 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 2200000, "y": 1300000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_d_uuid.to_string(): { "uuid": via_d_uuid, "net": target_net_uuid, "position": { "x": 3600000, "y": 2000000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
    )
}

fn seed_route_path_candidate_authored_copper_graph_zone_aware_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some(
            "Route Path Candidate Authored Copper Graph Zone Aware Proposal Artifact Demo"
                .to_string(),
        ),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x3450);
    let class_uuid = Uuid::from_u128(0x3451);
    let package_a_uuid = Uuid::from_u128(0x3452);
    let package_b_uuid = Uuid::from_u128(0x3453);
    let anchor_a_uuid = Uuid::from_u128(0x3454);
    let anchor_b_uuid = Uuid::from_u128(0x3455);
    let zone_uuid = Uuid::from_u128(0x3456);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3457),
                "name": "Route Path Candidate Authored Copper Graph Zone Aware Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): { "uuid": anchor_a_uuid, "package": package_a_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 },
                    anchor_b_uuid.to_string(): { "uuid": anchor_b_uuid, "package": package_b_uuid, "name": "1", "net": target_net_uuid, "position": { "x": 4500000, "y": 600000 }, "layer": 1, "shape": "circle", "diameter": 450000, "width": 0, "height": 0 }
                },
                "tracks": {},
                "vias": {},
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": target_net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 400000, "y": 500000 },
                                { "x": 4600000, "y": 500000 },
                                { "x": 4600000, "y": 700000 },
                                { "x": 400000, "y": 700000 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": true,
                        "thermal_gap": 150000,
                        "thermal_spoke_width": 120000
                    }
                },
                "nets": {
                    target_net_uuid.to_string(): { "uuid": target_net_uuid, "name": "SIG", "class": class_uuid }
                },
                "net_classes": {
                    class_uuid.to_string(): { "uuid": class_uuid, "name": "Default", "clearance": 150000, "track_width": 200000, "via_drill": 300000, "via_diameter": 600000, "diffpair_width": 0, "diffpair_gap": 0 }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid, zone_uuid)
}

fn seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some(
            "Route Path Candidate Authored Copper Graph Zone Obstacle Aware Proposal Artifact Demo"
                .to_string(),
        ),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x3850);
    let class_uuid = Uuid::from_u128(0x3851);
    let package_a_uuid = Uuid::from_u128(0x3852);
    let package_b_uuid = Uuid::from_u128(0x3853);
    let anchor_a_uuid = Uuid::from_u128(0x3854);
    let anchor_b_uuid = Uuid::from_u128(0x3855);
    let zone_uuid = Uuid::from_u128(0x3856);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3857),
                "name": "Route Path Candidate Authored Copper Graph Zone Obstacle Aware Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": target_net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 400000, "y": 500000 },
                                { "x": 4600000, "y": 500000 },
                                { "x": 4600000, "y": 700000 },
                                { "x": 400000, "y": 700000 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": true,
                        "thermal_gap": 150000,
                        "thermal_spoke_width": 120000
                    }
                },
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid, zone_uuid)
}

fn seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(root, Some("Topology Aware".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::from_u128(0x3900);
    let class_uuid = Uuid::from_u128(0x3901);
    let from_anchor_uuid = Uuid::from_u128(0x3902);
    let to_anchor_uuid = Uuid::from_u128(0x3903);
    let anchor_via_uuid = Uuid::from_u128(0x3904);
    let lower_track_first_uuid = Uuid::from_u128(0x3905);
    let lower_track_second_uuid = Uuid::from_u128(0x3906);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3907),
                "name": "Topology Aware Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Inner", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 5000000 },
                        { "x": 0, "y": 5000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    from_anchor_uuid.to_string(): {
                        "uuid": from_anchor_uuid,
                        "package": Uuid::from_u128(0x3908),
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    to_anchor_uuid.to_string(): {
                        "uuid": to_anchor_uuid,
                        "package": Uuid::from_u128(0x3909),
                        "name": "2",
                        "net": net_uuid,
                        "position": { "x": 3500000, "y": 500000 },
                        "layer": 2,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    Uuid::from_u128(0x390a).to_string(): {
                        "uuid": Uuid::from_u128(0x390a),
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 1500000, "y": 500000 },
                        "width": 120000,
                        "layer": 1
                    },
                    lower_track_first_uuid.to_string(): {
                        "uuid": lower_track_first_uuid,
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 2000000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    },
                    lower_track_second_uuid.to_string(): {
                        "uuid": lower_track_second_uuid,
                        "net": net_uuid,
                        "from": { "x": 2000000, "y": 500000 },
                        "to": { "x": 3500000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    },
                    Uuid::from_u128(0x390b).to_string(): {
                        "uuid": Uuid::from_u128(0x390b),
                        "net": net_uuid,
                        "from": { "x": 1500000, "y": 500000 },
                        "to": { "x": 3500000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    }
                },
                "vias": {
                    anchor_via_uuid.to_string(): {
                        "uuid": anchor_via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    },
                    Uuid::from_u128(0x390c).to_string(): {
                        "uuid": Uuid::from_u128(0x390c),
                        "net": net_uuid,
                        "position": { "x": 1500000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 100000,
                        "track_width": 120000,
                        "via_drill": 150000,
                        "via_diameter": 300000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        net_uuid,
        from_anchor_uuid,
        to_anchor_uuid,
        anchor_via_uuid,
        lower_track_first_uuid,
        lower_track_second_uuid,
    )
}

fn seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    create_native_project(root, Some("Layer Balance Aware".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::from_u128(0x3910);
    let class_uuid = Uuid::from_u128(0x3911);
    let from_anchor_uuid = Uuid::from_u128(0x3912);
    let to_anchor_uuid = Uuid::from_u128(0x3913);
    let selected_via_uuid = Uuid::from_u128(20);
    let selected_track_uuid = Uuid::from_u128(21);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3914),
                "name": "Layer Balance Aware Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Inner", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 5000000 },
                        { "x": 0, "y": 5000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    from_anchor_uuid.to_string(): {
                        "uuid": from_anchor_uuid,
                        "package": Uuid::from_u128(0x3915),
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    to_anchor_uuid.to_string(): {
                        "uuid": to_anchor_uuid,
                        "package": Uuid::from_u128(0x3916),
                        "name": "2",
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 500000 },
                        "layer": 2,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    Uuid::from_u128(10).to_string(): {
                        "uuid": Uuid::from_u128(10),
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 2000000, "y": 500000 },
                        "width": 120000,
                        "layer": 1
                    },
                    selected_track_uuid.to_string(): {
                        "uuid": selected_track_uuid,
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 2000000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    }
                },
                "vias": {
                    Uuid::from_u128(11).to_string(): {
                        "uuid": Uuid::from_u128(11),
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    },
                    selected_via_uuid.to_string(): {
                        "uuid": selected_via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    },
                    Uuid::from_u128(22).to_string(): {
                        "uuid": Uuid::from_u128(22),
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 500000 },
                        "from_layer": 2,
                        "to_layer": 1,
                        "diameter": 300000,
                        "drill": 150000
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 100000,
                        "track_width": 120000,
                        "via_drill": 150000,
                        "via_diameter": 300000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (
        net_uuid,
        from_anchor_uuid,
        to_anchor_uuid,
        selected_via_uuid,
        selected_track_uuid,
    )
}

pub(crate) fn seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(
    root: &Path,
) -> (Uuid, Uuid, Uuid, Uuid) {
    create_native_project(
        root,
        Some(
            "Route Path Candidate Authored Copper Graph Obstacle Aware Proposal Artifact Demo"
                .to_string(),
        ),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x3950);
    let class_uuid = Uuid::from_u128(0x3951);
    let package_a_uuid = Uuid::from_u128(0x3952);
    let package_b_uuid = Uuid::from_u128(0x3953);
    let anchor_a_uuid = Uuid::from_u128(0x3954);
    let anchor_b_uuid = Uuid::from_u128(0x3955);
    let track_uuid = Uuid::from_u128(0x3956);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3957),
                "name": "Route Path Candidate Authored Copper Graph Obstacle Aware Proposal Artifact Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    track_uuid.to_string(): {
                        "uuid": track_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 500000, "y": 600000 },
                        "to": { "x": 4500000, "y": 2400000 },
                        "width": 200000,
                        "layer": 1
                    }
                },
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    (target_net_uuid, anchor_a_uuid, anchor_b_uuid, track_uuid)
}

#[test]
fn project_route_proposal_artifact_exports_inspects_and_applies_plus_one_gap_route() {
    let root = unique_project_root("datum-eda-cli-project-route-proposal-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _) = seed_plus_one_gap_project(&root);
    let artifact = root.join("route-proposal.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-path-proposal",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "authored-copper-plus-one-gap",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_plus_one_gap_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "draw_track");
    assert_eq!(action["reason"], "authored_copper_plus_one_gap");
    assert_eq!(action["width_nm"], 150000);
    assert_eq!(action["from"]["x"], 700000);
    assert_eq!(action["to"]["x"], 1300000);

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-proposal-artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(inspect_report["actions"], 1);
    assert_eq!(inspect_report["draw_track_actions"], 1);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-route-proposal-artifact",
        root.to_str().unwrap(),
        "--artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "apply_route_proposal_artifact");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 1);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 700000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1300000);
    assert_eq!(apply_report["applied"][0]["width_nm"], 150000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().any(|track| {
        track.net == target_net_uuid
            && track.from.x == 700000
            && track.to.x == 1300000
            && track.width == 150000
    }));

    let route_output = execute(plus_one_gap_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("route query should succeed");
    let route_report: serde_json::Value =
        serde_json::from_str(&route_output).expect("route report should parse");
    assert_eq!(
        route_report["status"],
        "no_path_under_current_authored_constraints"
    );
    assert_eq!(route_report["summary"]["path_gap_step_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_proposal_artifact_exports_inspects_and_applies_full_path() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-proposal-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);
    let artifact = root.join("route-path-candidate-proposal.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-path-proposal",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "route-path-candidate",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(export_report["contract"], "m5_route_path_candidate_v2");

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0]["proposal_action"], "draw_track");
    assert_eq!(actions[0]["reason"], "route_path_candidate");
    assert_eq!(actions[0]["width_nm"], 200000);
    assert_eq!(actions[0]["from"]["x"], 500000);
    assert_eq!(actions[0]["to"]["x"], 4500000);
    assert_eq!(actions[0]["selected_path_segment_index"], 0);
    assert_eq!(actions[0]["selected_path_segment_count"], 1);

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-proposal-artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(inspect_report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(inspect_report["actions"], 1);
    assert_eq!(inspect_report["draw_track_actions"], 1);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-route-proposal-artifact",
        root.to_str().unwrap(),
        "--artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "apply_route_proposal_artifact");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 1);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 4500000);
    assert_eq!(apply_report["applied"][0]["width_nm"], 200000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].from.x, 500000);
    assert_eq!(tracks[0].to.x, 4500000);
    assert_eq!(tracks[0].width, 200000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_proposal_artifact_exports_single_layer_candidate_via_generic_surface() {
    let root = unique_project_root("datum-eda-cli-project-route-path-proposal-artifact-generic");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);
    let artifact = root.join("route-path-proposal-generic.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(export_report["contract"], "m5_route_path_candidate_v2");

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "draw_track");
    assert_eq!(action["reason"], "route_path_candidate");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_via_proposal_artifact_exports_inspects_and_applies_full_path() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-via-proposal-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_uuid) =
        seed_route_path_candidate_via_project(&root);
    let artifact = root.join("route-path-candidate-via-proposal.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-path-proposal",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "route-path-candidate-via",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 2);
    assert_eq!(export_report["contract"], "m5_route_path_candidate_via_v1");

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0]["proposal_action"], "draw_track");
    assert_eq!(actions[0]["reason"], "route_path_candidate_via");
    assert_eq!(actions[0]["reused_via_uuid"], via_uuid.to_string());
    assert_eq!(actions[0]["selected_path_segment_index"], 0);
    assert_eq!(actions[0]["selected_path_segment_count"], 2);
    assert_eq!(actions[1]["reused_via_uuid"], via_uuid.to_string());
    assert_eq!(actions[1]["selected_path_segment_index"], 1);

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-proposal-artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(inspect_report["contract"], "m5_route_path_candidate_via_v1");
    assert_eq!(inspect_report["actions"], 2);
    assert_eq!(inspect_report["draw_track_actions"], 2);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-route-proposal-artifact",
        root.to_str().unwrap(),
        "--artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "apply_route_proposal_artifact");
    assert_eq!(apply_report["artifact_actions"], 2);
    assert_eq!(apply_report["applied_actions"], 2);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 2);
    assert!(tracks.iter().all(|track| track.width == 200000));
    assert!(tracks.iter().any(|track| track.layer == 1));
    assert!(tracks.iter().any(|track| track.layer == 3));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_two_via_proposal_artifact_exports_inspects_and_applies_full_path() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-two-via-proposal-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_two_via_project(&root);
    let artifact = root.join("route-path-candidate-two-via-proposal.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-path-proposal",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "route-path-candidate-two-via",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 3);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_two_via_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0]["proposal_action"], "draw_track");
    assert_eq!(actions[0]["reason"], "route_path_candidate_two_via");
    assert_eq!(actions[0]["reused_via_uuid"], via_a_uuid.to_string());
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([via_a_uuid.to_string(), via_b_uuid.to_string()])
    );
    assert_eq!(actions[0]["selected_path_segment_index"], 0);
    assert_eq!(actions[0]["selected_path_segment_count"], 3);
    assert_eq!(actions[1]["selected_path_segment_index"], 1);
    assert_eq!(actions[2]["selected_path_segment_index"], 2);

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-route-proposal-artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_two_via_v1"
    );
    assert_eq!(inspect_report["actions"], 3);
    assert_eq!(inspect_report["draw_track_actions"], 3);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-route-proposal-artifact",
        root.to_str().unwrap(),
        "--artifact",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "apply_route_proposal_artifact");
    assert_eq!(apply_report["artifact_actions"], 3);
    assert_eq!(apply_report["applied_actions"], 3);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().all(|track| track.width == 200000));
    assert!(tracks.iter().any(|track| track.layer == 1));
    assert!(tracks.iter().any(|track| track.layer == 3));
    assert!(tracks.iter().any(|track| track.layer == 5));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_three_via_proposal_artifact_exports_inspects_and_applies_full_path()
{
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-three-via-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        seed_route_path_candidate_three_via_project(&root);
    let artifact = root.join("route-path-candidate-three-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-three-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 4);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_three_via_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(actions[0]["reason"], "route_path_candidate_three_via");
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([
            via_a_uuid.to_string(),
            via_b_uuid.to_string(),
            via_c_uuid.to_string()
        ])
    );
    assert_eq!(actions.len(), 4);

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_three_via_v1"
    );
    assert_eq!(inspect_report["draw_track_actions"], 4);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["applied_actions"], 4);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 4);
    assert!(tracks.iter().any(|track| track.layer == 1));
    assert!(tracks.iter().any(|track| track.layer == 3));
    assert!(tracks.iter().any(|track| track.layer == 5));
    assert!(tracks.iter().any(|track| track.layer == 7));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_four_via_proposal_artifact_exports_inspects_and_applies_full_path()
{
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-four-via-proposal-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
    ) = seed_route_path_candidate_four_via_project(&root);
    let artifact = root.join("route-path-candidate-four-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-four-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 5);

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([
            via_a_uuid.to_string(),
            via_b_uuid.to_string(),
            via_c_uuid.to_string(),
            via_d_uuid.to_string()
        ])
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["applied_actions"], 5);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 5);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_five_via_proposal_artifact_exports_inspects_and_applies_full_path()
{
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-five-via-proposal-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
    ) = seed_route_path_candidate_five_via_project(&root);
    let artifact = root.join("route-path-candidate-five-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-five-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 6);

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([
            via_a_uuid.to_string(),
            via_b_uuid.to_string(),
            via_c_uuid.to_string(),
            via_d_uuid.to_string(),
            via_e_uuid.to_string()
        ])
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["applied_actions"], 6);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 6);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_six_via_proposal_artifact_exports_inspects_and_applies_full_path() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-six-via-proposal-artifact");
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    ) = seed_route_path_candidate_six_via_project(&root);
    let artifact = root.join("route-path-candidate-six-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-six-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 7);

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([
            via_a_uuid.to_string(),
            via_b_uuid.to_string(),
            via_c_uuid.to_string(),
            via_d_uuid.to_string(),
            via_e_uuid.to_string(),
            via_f_uuid.to_string()
        ])
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["applied_actions"], 7);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 7);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_via_chain_proposal_artifact_exports_inspects_and_applies_full_path()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-via-chain-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_authored_via_chain_project(&root);
    let artifact = root.join("route-path-candidate-authored-via-chain-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-authored-via-chain",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 3);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_via_chain_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("artifact actions");
    assert_eq!(
        actions[0]["reason"],
        "route_path_candidate_authored_via_chain"
    );
    assert_eq!(
        actions[0]["reused_via_uuids"],
        serde_json::json!([via_a_uuid.to_string(), via_b_uuid.to_string()])
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["applied_actions"], 3);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 3);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_zone_aware_proposal_artifact_exports_inspects_and_applies_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-zone-aware-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, zone_uuid) =
        seed_route_path_candidate_authored_copper_graph_zone_aware_project(&root);
    let artifact = root.join("route-path-candidate-authored-copper-graph-zone-aware-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "zone_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        action["reason"],
        "route_path_candidate_authored_copper_graph_policy_zone_aware"
    );
    assert_eq!(action["reused_object_kind"], "zone");
    assert_eq!(action["reused_object_uuid"], zone_uuid.to_string());

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(inspect_report["actions"], 1);
    assert_eq!(inspect_report["draw_track_actions"], 0);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert!(tracks.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal_artifact_exports_inspects_and_applies_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-zone-obstacle-aware-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, zone_uuid) =
        seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_project(&root);
    let artifact =
        root.join("route-path-candidate-authored-copper-graph-zone-obstacle-aware-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "zone_obstacle_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        action["reason"],
        "route_path_candidate_authored_copper_graph_policy_zone_obstacle_aware"
    );
    assert_eq!(action["reused_object_kind"], "zone");
    assert_eq!(action["reused_object_uuid"], zone_uuid.to_string());

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(inspect_report["actions"], 1);
    assert_eq!(inspect_report["draw_track_actions"], 0);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert!(tracks.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal_artifact_exports_inspects_and_applies_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-proposal-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        anchor_via_uuid,
        lower_track_first_uuid,
        lower_track_second_uuid,
    ) = seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_project(
        &root,
    );
    let artifact = root.join(
        "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-proposal.json",
    );

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "zone_obstacle_topology_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 3);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("actions should be an array");
    assert_eq!(actions[0]["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        actions[0]["reason"],
        "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_aware"
    );
    assert_eq!(actions[0]["reused_object_kind"], "via");
    assert_eq!(
        actions[0]["reused_object_uuid"],
        anchor_via_uuid.to_string()
    );
    assert_eq!(actions[1]["reused_object_kind"], "track");
    assert_eq!(
        actions[1]["reused_object_uuid"],
        lower_track_first_uuid.to_string()
    );
    assert_eq!(actions[2]["reused_object_kind"], "track");
    assert_eq!(
        actions[2]["reused_object_uuid"],
        lower_track_second_uuid.to_string()
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(inspect_report["actions"], 3);
    assert_eq!(inspect_report["draw_track_actions"], 0);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 3);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 4);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal_artifact_exports_inspects_and_applies_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware-proposal-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        selected_via_uuid,
        selected_track_uuid,
    ) = seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_project(
        &root,
    );
    let artifact = root.join(
        "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-layer-balance-aware-proposal.json",
    );

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "zone_obstacle_topology_layer_balance_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 2);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("actions should be an array");
    assert_eq!(actions[0]["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        actions[0]["reason"],
        "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_layer_balance_aware"
    );
    assert_eq!(actions[0]["reused_object_kind"], "via");
    assert_eq!(
        actions[0]["reused_object_uuid"],
        selected_via_uuid.to_string()
    );
    assert_eq!(actions[1]["reused_object_kind"], "track");
    assert_eq!(
        actions[1]["reused_object_uuid"],
        selected_track_uuid.to_string()
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(inspect_report["actions"], 2);
    assert_eq!(inspect_report["draw_track_actions"], 0);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 2);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 2);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_obstacle_aware_proposal_artifact_exports_inspects_and_applies_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-obstacle-aware-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, track_uuid) =
        seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(&root);
    let artifact =
        root.join("route-path-candidate-authored-copper-graph-obstacle-aware-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "obstacle_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        action["reason"],
        "route_path_candidate_authored_copper_graph_policy_obstacle_aware"
    );
    assert_eq!(action["reused_object_kind"], "track");
    assert_eq!(action["reused_object_uuid"], track_uuid.to_string());

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    assert_eq!(
        inspect_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(inspect_report["actions"], 1);
    assert_eq!(inspect_report["draw_track_actions"], 0);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks: Vec<Track> = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed"),
    )
    .expect("track query output should parse");
    assert_eq!(tracks.len(), 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_policy_proposal_artifact_exports_plain_policy_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-policy-plain-proposal-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, track_uuid) =
        seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(&root);
    let artifact =
        root.join("route-path-candidate-authored-copper-graph-policy-plain-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "plain",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 1);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        action["reason"],
        "route_path_candidate_authored_copper_graph_policy_plain"
    );
    assert_eq!(action["reused_object_kind"], "track");
    assert_eq!(action["reused_object_uuid"], track_uuid.to_string());

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_proposal_artifact_exports_policy_candidate_via_generic_surface() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-proposal-artifact-generic-authored-copper-graph",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, track_uuid) =
        seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(&root);
    let artifact = root.join("route-path-proposal-generic-authored-copper-graph.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "plain",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let action = &artifact_value["actions"][0];
    assert_eq!(action["proposal_action"], "reuse_existing_copper_step");
    assert_eq!(
        action["reason"],
        "route_path_candidate_authored_copper_graph_policy_plain"
    );
    assert_eq!(action["reused_object_uuid"], track_uuid.to_string());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_proposal_rejects_policy_for_non_policy_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-proposal-artifact-generic-policy-misuse",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);
    let artifact = root.join("route-path-proposal-generic-policy-misuse.json");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-path-proposal",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "route-path-candidate",
        "--policy",
        "plain",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let err = execute(export_cli).expect_err("non-policy candidate should reject --policy");
    assert_eq!(
        err.to_string(),
        "export-route-path-proposal --policy is supported only for candidate authored-copper-graph"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_path_candidate_authored_copper_graph_policy_proposal_artifact_exports_layer_balance_policy_noop()
 {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-graph-policy-layer-balance-proposal-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        selected_via_uuid,
        selected_track_uuid,
    ) = seed_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_project(
        &root,
    );
    let artifact =
        root.join("route-path-candidate-authored-copper-graph-policy-layer-balance-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "authored-copper-graph",
            "--policy",
            "zone_obstacle_topology_layer_balance_aware",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 2);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"]
        .as_array()
        .expect("actions should be an array");
    assert_eq!(
        actions[0]["reason"],
        "route_path_candidate_authored_copper_graph_policy_zone_obstacle_topology_layer_balance_aware"
    );
    assert_eq!(actions[0]["reused_object_kind"], "via");
    assert_eq!(
        actions[0]["reused_object_uuid"],
        selected_via_uuid.to_string()
    );
    assert_eq!(actions[1]["reused_object_kind"], "track");
    assert_eq!(
        actions[1]["reused_object_uuid"],
        selected_track_uuid.to_string()
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 2);
    assert_eq!(apply_report["applied_actions"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_dogleg_candidate() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-orthogonal-dogleg-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) =
        seed_route_path_candidate_orthogonal_dogleg_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-dogleg-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-dogleg",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 2);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_dogleg_v1"
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    assert_eq!(artifact_value["actions"].as_array().unwrap().len(), 2);
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_dogleg"
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 2);
    assert_eq!(apply_report["applied_actions"], 2);

    let board_tracks_output = execute(board_tracks_query_cli(&root)).expect("query should succeed");
    let board_tracks: serde_json::Value =
        serde_json::from_str(&board_tracks_output).expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len(), 3);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_two_bend_candidate() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-orthogonal-two-bend-artifact");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) =
        seed_route_path_candidate_orthogonal_two_bend_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-two-bend-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-two-bend",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["action"], "export_route_path_proposal");
    assert_eq!(export_report["actions"], 3);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_two_bend_v1"
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 3);
    assert_eq!(apply_report["applied_actions"], 3);

    let board_tracks_output = execute(board_tracks_query_cli(&root)).expect("query should succeed");
    let board_tracks: serde_json::Value =
        serde_json::from_str(&board_tracks_output).expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len(), 5);

    let _ = std::fs::remove_dir_all(&root);
}
