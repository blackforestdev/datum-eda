use super::super::source_shard::{
    source_shard_taxon_for_path, validate_source_shard_ownership_path,
};
use super::*;

#[test]
fn datum_sidecar_and_generated_evidence_paths_have_concrete_taxonomy() {
    for (kind, relative_path, expected_taxon) in [
        (
            SourceShardKind::Relationship,
            ".datum/relationships/example.json",
            SourceShardTaxon::Relationship,
        ),
        (
            SourceShardKind::ComponentInstance,
            ".datum/component_instances/example.json",
            SourceShardTaxon::ComponentInstance,
        ),
        (
            SourceShardKind::VariantOverlay,
            ".datum/variants/example.json",
            SourceShardTaxon::VariantOverlay,
        ),
        (
            SourceShardKind::ManufacturingPlan,
            ".datum/manufacturing_plans/example.json",
            SourceShardTaxon::ManufacturingPlan,
        ),
        (
            SourceShardKind::PanelProjection,
            ".datum/panel_projections/example.json",
            SourceShardTaxon::PanelProjection,
        ),
        (
            SourceShardKind::OutputJob,
            ".datum/output_jobs/example.json",
            SourceShardTaxon::OutputJob,
        ),
        (
            SourceShardKind::ImportMap,
            ".datum/import_map/example.json",
            SourceShardTaxon::ImportMap,
        ),
        (
            SourceShardKind::ProposalMetadata,
            ".datum/proposals/example.json",
            SourceShardTaxon::ProposalMetadata,
        ),
        (
            SourceShardKind::ForwardAnnotationReview,
            ".datum/forward_annotation_review/review.json",
            SourceShardTaxon::ForwardAnnotationReview,
        ),
        (
            SourceShardKind::OutputJobRun,
            ".datum/output_job_runs/example.json",
            SourceShardTaxon::OutputJobRun,
        ),
        (
            SourceShardKind::ArtifactRun,
            ".datum/artifact_runs/example.json",
            SourceShardTaxon::ArtifactRun,
        ),
        (
            SourceShardKind::CheckRun,
            ".datum/check_runs/example.json",
            SourceShardTaxon::CheckRun,
        ),
        (
            SourceShardKind::ZoneFill,
            ".datum/zone_fills/example.json",
            SourceShardTaxon::ZoneFill,
        ),
        (
            SourceShardKind::ArtifactMetadata,
            ".datum/artifacts/example.json",
            SourceShardTaxon::ArtifactMetadata,
        ),
    ] {
        validate_source_shard_ownership_path(&kind, relative_path)
            .unwrap_or_else(|error| panic!("{kind:?} should own {relative_path}: {error}"));
        assert_eq!(
            source_shard_taxon_for_path(&kind, relative_path),
            Some(expected_taxon)
        );
    }
}
