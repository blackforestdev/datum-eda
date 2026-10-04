use super::*;
use datum_gui_protocol::{Affine2DFixedPrimitive, MeshRectEmPrimitive};
fn basis() -> (Affine2DFixedPrimitive, MeshRectEmPrimitive) {
    (
        Affine2DFixedPrimitive {
            m11_ppm: 0,
            m12_ppm: -500_000,
            m21_ppm: 1_000_000,
            m22_ppm: 0,
            tx_nm: -10,
            ty_nm: 20,
        },
        MeshRectEmPrimitive {
            min_x_em_nm: -1,
            min_y_em_nm: -1,
            max_x_em_nm: 1,
            max_y_em_nm: 1,
        },
    )
}
#[test]
fn captured_layout_basis_preserves_fractional_edges_and_conservative_bounds() {
    let (transform, bbox) = basis();
    let shape = HitShape::LayoutBasis { transform, bbox };
    // Independently: (-y/2-10, x+20) maps this square to a half-nm
    // horizontal extent and a one-nm vertical extent, with inclusive edges.
    assert_eq!(
        shape.bounds(),
        Some(RectNm {
            min_x: -11,
            max_x: -9,
            min_y: 19,
            max_y: 21
        })
    );
    for y in [19, 20, 21] {
        assert!(shape.contains(PointNm { x: -10, y }));
    }
    for x in [-11, -9] {
        assert!(
            !shape.contains(PointNm { x, y: 20 }),
            "AABB is not the exact footprint"
        );
    }
    for y in [18, 22] {
        assert!(!shape.contains(PointNm { x: -10, y }));
    }
}
#[test]
fn captured_layout_basis_matches_owned_corners_for_reflection_rotation_and_extreme_translation() {
    let (mut transform, bbox) = basis();
    for (a, b, c, d, tx, ty) in [
        (1_000_000, 0, 0, 1_000_000, 0, 0),
        (-1_000_000, 0, 0, 500_000, -10, 20),
        (707_107, -707_107, 707_107, 707_107, -30, 40),
        (0, -500_000, 1_000_000, 0, i64::MAX - 2, i64::MIN + 2),
    ] {
        transform.m11_ppm = a;
        transform.m12_ppm = b;
        transform.m21_ppm = c;
        transform.m22_ppm = d;
        transform.tx_nm = tx;
        transform.ty_nm = ty;
        let corners = datum_gui_protocol::selection_geometry_catalog::layout_polygon_from_basis(
            transform, bbox,
        )
        .unwrap();
        let legacy = HitShape::Layout(corners.to_vec());
        let compact = HitShape::LayoutBasis { transform, bbox };
        assert_eq!(compact.bounds(), legacy.bounds());
        for dx in -2_i64..=2 {
            for dy in -2_i64..=2 {
                let p = PointNm {
                    x: tx.saturating_add(dx),
                    y: ty.saturating_add(dy),
                };
                assert_eq!(compact.contains(p), legacy.contains(p));
            }
        }
        let index = SpatialHitIndex::new(vec![HitRegion {
            target: "source",
            layer_id: None,
            shape: compact,
        }]);
        assert_eq!(
            index.hit_test(PointNm { x: tx, y: ty }, |_| true).target,
            Some(&"source")
        );
    }
}
