//! Focused check for dat-zone-run-freeze-dadx: repeated Zone Run acquisition on an immutable
//! basis reuses the one completed result instead of re-running the net-graph/contact sweep per
//! click, while staying keyed on (origin, hit) so it never returns a stale region.
//!
//! The reuse is output-equivalent to a recompute by construction (the snapshot is immutable, so
//! `board_run` is a pure function of (origin, hit)); that equivalence is exactly what makes the
//! short-circuit safe. These assertions guard the correctness invariant any such cache must hold:
//! exact Run identity is preserved across a repeat, and a different hit resolves its own region
//! rather than a stale cached one (which a naive origin-only cache would get wrong).
#[path = "support/native_selection_fixture.rs"]
mod fixture;

use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject,
    selection_resolution::NativeSelectionResolution,
};
use eda_engine::ir::geometry::Point;
use fixture::{Fixture, id, rectangle};

/// A single zone filled as two disjoint islands: hit (1,1) resolves the first region, hit (21,1)
/// the second. Repeated acquisition of the same hit is the frozen "repeated Zone click" path.
#[test]
fn repeated_zone_run_acquisition_is_consistent_and_hit_keyed() {
    let mut f = Fixture::new("zone-run-reuse");
    let zone = f.zone(id(10));
    f.fill(
        &zone,
        eda_engine::substrate::ZoneFillState::Filled,
        vec![rectangle(0, 0, 4, 4), rectangle(20, 0, 4, 4)],
    );
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let zone_id = Id {
        class: Class::Zone,
        id: zone.uuid,
        instance_path: vec![],
    };

    // Repeated acquisition of the same origin+hit (the repro click sequence) returns the exact
    // same Run both times -- the completed result is reused, Run identity is preserved.
    let first = authority
        .acquire_run(zone_id.clone(), Some(Point::new(1, 1)))
        .unwrap();
    let again = authority
        .acquire_run(zone_id.clone(), Some(Point::new(1, 1)))
        .unwrap();
    assert_eq!(
        first, again,
        "a repeated same-hit acquisition preserves the Run"
    );
    assert!(matches!(first, SelectionSubject::Run(_)));

    // A different hit on the same zone origin resolves the OTHER region: the reuse is keyed on
    // (origin, hit), so it never serves a stale region. (An origin-only cache would fail here.)
    let other = authority
        .acquire_run(zone_id.clone(), Some(Point::new(21, 1)))
        .unwrap();
    assert_ne!(
        first, other,
        "a different hit must resolve its own region, not the cached one"
    );

    // Re-acquiring the first hit after the cache-displacing second hit still yields region one:
    // a displaced entry recomputes correctly rather than returning the last-cached Run.
    let back = authority
        .acquire_run(zone_id, Some(Point::new(1, 1)))
        .unwrap();
    assert_eq!(
        first, back,
        "the original hit still resolves its own region"
    );
}
