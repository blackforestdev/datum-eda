use super::*;
use crate::terminal_accessibility_platform::selection_fixture as fixture;

#[test]
fn same_source_revision_refreshes_canonical_proposal_status_without_rebuilding_geometry_authority()
{
    use eda_engine::substrate::{
        self, CommitProvenance, CommitSource, OperationBatch, ProposalCreateRequest,
        ProposalSource, ProposalStatus,
    };
    let mut f = fixture::Fixture::new("metadata-cache");
    let track = f.line(
        fixture::id(1),
        eda_engine::ir::geometry::Point::new(0, 0),
        eda_engine::ir::geometry::Point::new(10000, 0),
    );
    let expected = f.model.model_revision.clone();
    let p = substrate::create_draft_proposal_from_batch(
        &mut f.model,
        &f.root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: OperationBatch {
                batch_id: fixture::id(3),
                expected_model_revision: Some(expected),
                provenance: CommitProvenance {
                    actor: "cache-proof".into(),
                    source: CommitSource::Test,
                    reason: "proposal lifetime refresh".into(),
                },
                operations: vec![substrate::Operation::SetBoardTrack {
                    track_id: track.uuid,
                    track: serde_json::to_value(track).unwrap(),
                }],
            },
            rationale: "unchanged source".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    let mut cache = SelectionReadCache::default();
    let initial = cache.resolve(&f.root).unwrap();
    let authority = NativeSelectionResolution::capture(&initial).unwrap();
    let mut session = NativeSelectionSession {
        model: initial,
        authority,
        last_click: None,
        lineage: None,
        binding_bases: Default::default(),
        region_bases: Default::default(),
    };
    substrate::review_proposal_status(
        &mut f.model,
        &f.root,
        p.proposal_id,
        ProposalStatus::Rejected,
    )
    .unwrap();
    let reopened = cache.resolve(&f.root).unwrap();
    assert_eq!(reopened.model_revision, session.model.model_revision);
    let snapshot_revision = session.authority.snapshot().revision().clone();
    let authored = session.authority.authored_identities().clone();
    let first_source = session.authority.authored_identities().first().unwrap() as *const _;
    let before = f.source_bytes();
    assert!(session.refresh_same_revision(reopened).is_none());
    assert_eq!(
        session.model.proposals[&p.proposal_id].status,
        ProposalStatus::Rejected
    );
    assert_eq!(session.authority.snapshot().revision(), &snapshot_revision);
    assert_eq!(session.authority.authored_identities(), &authored);
    assert!(std::ptr::eq(
        first_source,
        session.authority.authored_identities().first().unwrap()
    ));
    fixture::Fixture::assert_bytes(&before);
    f.line(
        fixture::id(9),
        eda_engine::ir::geometry::Point::new(20000, 0),
        eda_engine::ir::geometry::Point::new(30000, 0),
    );
    assert!(
        session
            .refresh_same_revision(Arc::new(f.model.clone()))
            .is_some()
    );
}
