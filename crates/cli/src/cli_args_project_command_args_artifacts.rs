use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectNewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project display name; defaults to the directory basename
    #[arg(long)]
    pub(crate) name: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectQueryArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// What to query
    #[command(subcommand)]
    pub(crate) what: NativeProjectQueryCommands,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectExcellonDrillArgs {
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectGerberArgs {
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberOutlineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberCopperLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberSoldermaskLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Soldermask layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberSilkscreenLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Silkscreen layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberPasteLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Paste layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportGerberMechanicalLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Mechanical layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Output Gerber path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberOutlineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberCopperLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Copper layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberSoldermaskLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Soldermask layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberSilkscreenLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Silkscreen layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberPasteLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Paste layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateGerberMechanicalLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Mechanical layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to validate
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberOutlineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberCopperLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Copper layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberSoldermaskLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Soldermask layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberSilkscreenLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Silkscreen layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberPasteLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Paste layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareGerberMechanicalLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Mechanical layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    /// Gerber path to compare
    #[arg(long = "gerber")]
    pub(crate) gerber: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportForwardAnnotationAuditArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output path for the audit artifact
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectForwardAnnotationAuditArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectApplyForwardAnnotationActionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable proposal action ID
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
    /// Explicit package UUID for add_component actions
    #[arg(long = "package")]
    pub(crate) package_uuid: Option<Uuid>,
    /// Explicit part UUID override for add_component actions
    #[arg(long = "part")]
    pub(crate) part_uuid: Option<Uuid>,
    /// Placement X coordinate in nm for add_component actions
    #[arg(long = "x-nm")]
    pub(crate) x_nm: Option<i64>,
    /// Placement Y coordinate in nm for add_component actions
    #[arg(long = "y-nm")]
    pub(crate) y_nm: Option<i64>,
    /// Placement layer for add_component actions
    #[arg(long = "layer")]
    pub(crate) layer: Option<i32>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectApplyForwardAnnotationReviewedArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportRoutePathProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Accepted deterministic candidate family
    #[arg(long = "candidate", value_enum)]
    pub(crate) candidate: NativeProjectRouteApplyCandidateArg,
    /// Accepted authored-copper-graph policy when required by the candidate family
    #[arg(
        long = "policy",
        value_enum,
        required_if_eq("candidate", "authored-copper-graph")
    )]
    pub(crate) policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportRouteProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Deterministic selector profile
    #[arg(long = "profile", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) profile: NativeProjectRouteProposalProfileArg,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Deterministic selector profile
    #[arg(long = "profile", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) profile: NativeProjectRouteProposalProfileArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteProposalExplainArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Deterministic selector profile
    #[arg(long = "profile", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) profile: NativeProjectRouteProposalProfileArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectReviewRouteProposalArgs {
    /// Project root directory when reviewing the current selected proposal
    #[arg(required_unless_present = "artifact", conflicts_with = "artifact")]
    pub(crate) path: Option<PathBuf>,
    /// Net UUID when reviewing the current selected proposal
    #[arg(long = "net", required_unless_present = "artifact")]
    pub(crate) net_uuid: Option<Uuid>,
    /// Source anchor pad UUID when reviewing the current selected proposal
    #[arg(long = "from-anchor", required_unless_present = "artifact")]
    pub(crate) from_anchor_pad_uuid: Option<Uuid>,
    /// Target anchor pad UUID when reviewing the current selected proposal
    #[arg(long = "to-anchor", required_unless_present = "artifact")]
    pub(crate) to_anchor_pad_uuid: Option<Uuid>,
    /// Deterministic selector profile when reviewing the current selected proposal
    #[arg(long = "profile", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) profile: NativeProjectRouteProposalProfileArg,
    /// Saved route proposal artifact path to review instead of live project selection
    #[arg(long = "artifact", conflicts_with_all = ["path", "net_uuid", "from_anchor_pad_uuid", "to_anchor_pad_uuid"])]
    pub(crate) artifact: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteStrategyReportArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Accepted deterministic routing objective from the selector profile vocabulary
    #[arg(long = "objective", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) objective: NativeProjectRouteProposalProfileArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteStrategyCompareArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteStrategyDeltaArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteStrategyBatchEvaluateArgs {
    /// Versioned batch request manifest path
    #[arg(long = "requests")]
    pub(crate) requests: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectWriteRouteStrategyCuratedFixtureSuiteArgs {
    /// Output directory for the curated fixture projects
    #[arg(long = "out-dir")]
    pub(crate) out_dir: PathBuf,
    /// Optional path for the generated batch request manifest
    #[arg(long = "manifest")]
    pub(crate) manifest: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCaptureRouteStrategyCuratedBaselineArgs {
    /// Output directory for the curated fixture projects and saved baseline artifact
    #[arg(long = "out-dir")]
    pub(crate) out_dir: PathBuf,
    /// Optional path for the generated batch request manifest
    #[arg(long = "manifest")]
    pub(crate) manifest: Option<PathBuf>,
    /// Optional path for the saved batch-result baseline artifact
    #[arg(long = "result")]
    pub(crate) result: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectRouteStrategyBatchResultArgs {
    /// Saved batch result artifact path
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateRouteStrategyBatchResultArgs {
    /// Saved batch result artifact path
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareRouteStrategyBatchResultArgs {
    /// Earlier saved batch result artifact path
    pub(crate) before: PathBuf,
    /// Later saved batch result artifact path
    pub(crate) after: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
pub(crate) enum NativeProjectRouteStrategyBatchGatePolicyArg {
    #[value(name = "strict_identical")]
    StrictIdentical,
    #[value(name = "allow_aggregate_only")]
    AllowAggregateOnly,
    #[value(name = "fail_on_recommendation_change")]
    FailOnRecommendationChange,
}

#[derive(clap::Args)]
pub(crate) struct ProjectGateRouteStrategyBatchResultArgs {
    /// Earlier saved batch result artifact path
    pub(crate) before: PathBuf,
    /// Later saved batch result artifact path
    pub(crate) after: PathBuf,
    /// Deterministic gate policy
    #[arg(long = "policy", value_enum, default_value_t = NativeProjectRouteStrategyBatchGatePolicyArg::StrictIdentical)]
    pub(crate) policy: NativeProjectRouteStrategyBatchGatePolicyArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSummarizeRouteStrategyBatchResultsArgs {
    /// Directory containing saved batch result artifacts
    #[arg(long = "dir", conflicts_with = "artifacts")]
    pub(crate) dir: Option<PathBuf>,
    /// Explicit saved batch result artifact paths
    #[arg(long = "artifact", conflicts_with = "dir")]
    pub(crate) artifacts: Vec<PathBuf>,
    /// Optional baseline artifact to gate all other compatible artifacts against
    #[arg(long = "baseline")]
    pub(crate) baseline: Option<PathBuf>,
    /// Deterministic gate policy to use when --baseline is provided
    #[arg(long = "policy", value_enum, default_value_t = NativeProjectRouteStrategyBatchGatePolicyArg::StrictIdentical)]
    pub(crate) policy: NativeProjectRouteStrategyBatchGatePolicyArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteApplySelectedArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Deterministic selector profile
    #[arg(long = "profile", value_enum, default_value_t = NativeProjectRouteProposalProfileArg::Default)]
    pub(crate) profile: NativeProjectRouteProposalProfileArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectRouteProposalArtifactArgs {
    /// Artifact path
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRevalidateRouteProposalArtifactArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectApplyRouteProposalArtifactArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRouteApplyArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Source anchor pad UUID
    #[arg(long = "from-anchor")]
    pub(crate) from_anchor_pad_uuid: Uuid,
    /// Target anchor pad UUID
    #[arg(long = "to-anchor")]
    pub(crate) to_anchor_pad_uuid: Uuid,
    /// Accepted deterministic candidate family
    #[arg(long = "candidate", value_enum)]
    pub(crate) candidate: NativeProjectRouteApplyCandidateArg,
    /// Accepted authored-copper-graph policy when required by the candidate family
    #[arg(
        long = "policy",
        value_enum,
        required_if_eq("candidate", "authored-copper-graph")
    )]
    pub(crate) policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub(crate) enum NativeProjectRouteApplyCandidateArg {
    #[value(name = "route-path-candidate")]
    RoutePathCandidate,
    #[value(name = "route-path-candidate-via")]
    RoutePathCandidateVia,
    #[value(name = "route-path-candidate-two-via")]
    RoutePathCandidateTwoVia,
    #[value(name = "route-path-candidate-three-via")]
    RoutePathCandidateThreeVia,
    #[value(name = "route-path-candidate-four-via")]
    RoutePathCandidateFourVia,
    #[value(name = "route-path-candidate-five-via")]
    RoutePathCandidateFiveVia,
    #[value(name = "route-path-candidate-six-via")]
    RoutePathCandidateSixVia,
    #[value(name = "route-path-candidate-authored-via-chain")]
    RoutePathCandidateAuthoredViaChain,
    #[value(name = "route-path-candidate-orthogonal-dogleg")]
    RoutePathCandidateOrthogonalDogleg,
    #[value(name = "route-path-candidate-orthogonal-two-bend")]
    RoutePathCandidateOrthogonalTwoBend,
    #[value(name = "route-path-candidate-orthogonal-graph")]
    RoutePathCandidateOrthogonalGraph,
    #[value(name = "route-path-candidate-orthogonal-graph-via")]
    RoutePathCandidateOrthogonalGraphVia,
    #[value(name = "route-path-candidate-orthogonal-graph-two-via")]
    RoutePathCandidateOrthogonalGraphTwoVia,
    #[value(name = "route-path-candidate-orthogonal-graph-three-via")]
    RoutePathCandidateOrthogonalGraphThreeVia,
    #[value(name = "route-path-candidate-orthogonal-graph-four-via")]
    RoutePathCandidateOrthogonalGraphFourVia,
    #[value(name = "route-path-candidate-orthogonal-graph-five-via")]
    RoutePathCandidateOrthogonalGraphFiveVia,
    #[value(name = "route-path-candidate-orthogonal-graph-six-via")]
    RoutePathCandidateOrthogonalGraphSixVia,
    #[value(name = "authored-copper-plus-one-gap")]
    AuthoredCopperPlusOneGap,
    #[value(name = "authored-copper-graph")]
    AuthoredCopperGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub(crate) enum NativeProjectRouteProposalProfileArg {
    #[value(name = "default")]
    Default,
    #[value(name = "authored-copper-priority")]
    AuthoredCopperPriority,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportForwardAnnotationProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectExportForwardAnnotationProposalSelectionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable proposal action IDs to retain
    #[arg(long = "action-id")]
    pub(crate) action_ids: Vec<String>,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSelectForwardAnnotationProposalArtifactArgs {
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
    /// Stable proposal action IDs to retain
    #[arg(long = "action-id")]
    pub(crate) action_ids: Vec<String>,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectInspectForwardAnnotationProposalArtifactArgs {
    /// Artifact path
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateForwardAnnotationProposalArtifactArgs {
    /// Artifact path
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCompareForwardAnnotationProposalArtifactArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectFilterForwardAnnotationProposalArtifactArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
    /// Output artifact path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlanForwardAnnotationProposalArtifactApplyArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectApplyForwardAnnotationProposalArtifactArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectImportForwardAnnotationArtifactReviewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectReplaceForwardAnnotationArtifactReviewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Artifact path
    #[arg(long = "artifact")]
    pub(crate) artifact: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeferForwardAnnotationActionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable proposal action ID
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRejectForwardAnnotationActionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable proposal action ID
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearForwardAnnotationActionReviewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable proposal action ID
    #[arg(long = "action-id")]
    pub(crate) action_id: String,
}
