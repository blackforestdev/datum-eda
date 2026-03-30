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
