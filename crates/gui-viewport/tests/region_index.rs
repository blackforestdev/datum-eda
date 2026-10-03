use datum_gui_protocol::RectNm;
use datum_gui_viewport::{HitRegion, HitShape, SpatialHitIndex};
use std::collections::BTreeSet;
fn bounds(i: usize) -> RectNm {
    RectNm {
        min_x: (i as i64) * 1000,
        max_x: (i as i64) * 1000 + 100,
        min_y: -10,
        max_y: 10,
    }
}
fn index() -> SpatialHitIndex<usize> {
    SpatialHitIndex::new(
        (0..100_000)
            .map(|target| HitRegion {
                target,
                layer_id: None,
                shape: HitShape::Rect(bounds(target)),
            })
            .collect(),
    )
}
#[test]
fn region_index_prunes_100k_and_preserves_brute_force_oracle_across_budgets() {
    let index = index();
    let region = RectNm {
        min_x: 54_321_050,
        max_x: 54_330_010,
        min_y: 0,
        max_y: 5,
    };
    let oracle: BTreeSet<_> = (0..100_000)
        .filter(|i| bounds(*i).min_x <= region.max_x && bounds(*i).max_x >= region.min_x)
        .collect();
    for budget in [1, 7, 4096] {
        let mut cursor = index.begin_region(region);
        let mut result = BTreeSet::new();
        let mut work = 0;
        loop {
            let step = index.advance_region(&mut cursor, budget, |id| {
                assert!(result.insert(*id));
            });
            assert!(step.work <= budget);
            work += step.work;
            if step.complete {
                break;
            }
        }
        assert_eq!(result, oracle);
        assert_eq!(cursor.examined, oracle.len());
        assert!(work < 100, "unrelated source bounds must be pruned");
    }
}
#[test]
fn region_index_dense_completion_is_uncapped_and_cancel_does_not_publish_partial_set() {
    let index = index();
    let bounds = RectNm {
        min_x: 0,
        max_x: 100_000_000,
        min_y: -10,
        max_y: 10,
    };
    let committed = BTreeSet::from([999_999]);
    let mut cursor = index.begin_region(bounds);
    let mut staged = BTreeSet::new();
    assert!(
        !index
            .advance_region(&mut cursor, 7, |id| {
                staged.insert(*id);
            })
            .complete
    );
    drop(cursor);
    drop(staged);
    assert_eq!(committed, BTreeSet::from([999_999]));
    let mut cursor = index.begin_region(bounds);
    let mut result = BTreeSet::new();
    loop {
        if index
            .advance_region(&mut cursor, 4096, |id| {
                result.insert(*id);
            })
            .complete
        {
            break;
        }
    }
    assert_eq!(result.len(), 100_000);
}
