use super::*;

pub(super) fn seed_curated_route_strategy_profile_divergence_fixture(
    root: &Path,
) -> Result<(Uuid, Uuid, Uuid)> {
    let created = create_native_project(
        root,
        Some("Route Strategy Curated Profile Divergence Demo".to_string()),
    )?;

    let target_net_uuid = Uuid::from_u128(0xc280);
    let class_uuid = Uuid::from_u128(0xc281);
    let package_a_uuid = Uuid::from_u128(0xc282);
    let package_b_uuid = Uuid::from_u128(0xc283);
    let anchor_a_uuid = Uuid::from_u128(0xc284);
    let anchor_b_uuid = Uuid::from_u128(0xc285);
    let authored_track_uuid = Uuid::from_u128(0xc286);
    let spec = route_proposal::RouteStrategyFixtureBoardSpec {
        stackup_layers: vec![StackupLayer::new(
            1,
            "Top Copper",
            StackupLayerType::Copper,
            35_000,
        )],
        outline: route_strategy_fixture_outline(4_000_000, 1_000_000),
        net_classes: vec![route_strategy_fixture_net_class(class_uuid)],
        nets: vec![route_strategy_fixture_net(
            target_net_uuid,
            "SIG",
            class_uuid,
        )],
        pads: vec![
            route_strategy_fixture_pad(
                anchor_a_uuid,
                package_a_uuid,
                target_net_uuid,
                Point {
                    x: 500_000,
                    y: 500_000,
                },
                1,
                400_000,
            ),
            route_strategy_fixture_pad(
                anchor_b_uuid,
                package_b_uuid,
                target_net_uuid,
                Point {
                    x: 3_500_000,
                    y: 500_000,
                },
                1,
                400_000,
            ),
        ],
        tracks: vec![Track::straight(
            authored_track_uuid,
            target_net_uuid,
            Point {
                x: 500_000,
                y: 500_000,
            },
            Point {
                x: 3_500_000,
                y: 500_000,
            },
            200_000,
            1,
        )],
        vias: Vec::new(),
    };
    commit_route_strategy_fixture_board(root, route_strategy_fixture_board_uuid(&created)?, &spec)?;
    Ok((target_net_uuid, anchor_a_uuid, anchor_b_uuid))
}
