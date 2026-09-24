use super::*;

pub(crate) const NATIVE_COMBINED_CHECK_PROFILE: &str = "native-combined";
const ERC_CHECK_PROFILE: &str = "erc";
const DRC_CHECK_PROFILE: &str = "drc";
const STANDARDS_CHECK_PROFILE: &str = "standards";
const MANUFACTURING_CHECK_PROFILE: &str = "manufacturing";
const RELEASE_CHECK_PROFILE: &str = "release";

pub(crate) fn query_native_project_check_profiles(
    root: &Path,
) -> Result<NativeProjectCheckProfilesView> {
    let model = eda_engine::substrate::ProjectResolver::new(root).resolve()?;
    let profiles = native_check_profile_descriptors();
    Ok(NativeProjectCheckProfilesView {
        contract: "check_profiles_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        default_profile_id: "native-combined",
        profile_count: profiles.len(),
        profiles,
    })
}

pub(crate) fn resolve_native_project_check_profile(profile: Option<&str>) -> Result<&'static str> {
    match profile.unwrap_or(NATIVE_COMBINED_CHECK_PROFILE) {
        NATIVE_COMBINED_CHECK_PROFILE => Ok(NATIVE_COMBINED_CHECK_PROFILE),
        ERC_CHECK_PROFILE => Ok(ERC_CHECK_PROFILE),
        DRC_CHECK_PROFILE => Ok(DRC_CHECK_PROFILE),
        STANDARDS_CHECK_PROFILE => Ok(STANDARDS_CHECK_PROFILE),
        MANUFACTURING_CHECK_PROFILE => Ok(MANUFACTURING_CHECK_PROFILE),
        RELEASE_CHECK_PROFILE => Ok(RELEASE_CHECK_PROFILE),
        profile => bail!(
            "unsupported check profile {profile}; current native-project surface supports native-combined, erc, drc, standards, manufacturing, release"
        ),
    }
}

pub(crate) fn filter_check_run_findings_for_profile(
    profile_id: &str,
    findings: &mut Vec<NativeProjectCheckFindingView>,
) {
    findings.retain(|finding| match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE => true,
        ERC_CHECK_PROFILE => finding.domain == "erc",
        DRC_CHECK_PROFILE => finding.domain == "drc",
        MANUFACTURING_CHECK_PROFILE => finding.domain == "manufacturing",
        STANDARDS_CHECK_PROFILE => finding.domain == "standards",
        _ => true,
    });
    for (index, finding) in findings.iter_mut().enumerate() {
        finding.index = index;
    }
}

pub(crate) fn check_profile_includes_relationships(profile_id: &str) -> bool {
    is_combined_profile(profile_id)
}

pub(crate) fn check_profile_includes_erc(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == ERC_CHECK_PROFILE
}

pub(crate) fn check_profile_includes_artifacts(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == MANUFACTURING_CHECK_PROFILE
}

pub(crate) fn check_profile_includes_zone_fills(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == STANDARDS_CHECK_PROFILE
}

pub(crate) fn check_profile_drc_rules(profile_id: &str) -> &'static [RuleType] {
    match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE | DRC_CHECK_PROFILE => &[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
            RuleType::ProcessAperture,
        ],
        STANDARDS_CHECK_PROFILE => &[
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::ProcessAperture,
        ],
        _ => &[],
    }
}

fn is_combined_profile(profile_id: &str) -> bool {
    [NATIVE_COMBINED_CHECK_PROFILE, RELEASE_CHECK_PROFILE].contains(&profile_id)
}

pub(crate) fn profile_basis_for_check_run(profile_id: &str) -> CheckRunProfileBasis {
    let profile = native_check_profile_descriptor(profile_id);
    CheckRunProfileBasis {
        profile_id: profile.profile_id.to_string(),
        domains: profile
            .domains
            .iter()
            .map(|domain| domain.to_string())
            .collect(),
        description: profile.description.to_string(),
        standards_basis: (profile.profile_id == STANDARDS_CHECK_PROFILE)
            .then_some("datum.process_aperture_and_geometry.current".to_string()),
        standards_basis_detail: (profile.profile_id == STANDARDS_CHECK_PROFILE)
            .then(|| standards_basis_detail("datum.process_aperture_and_geometry.current")),
    }
}

pub(crate) fn check_run_coverage_for_profile(profile_id: &str) -> Vec<CheckRunCoverageEntry> {
    [
        coverage_entry(
            profile_id,
            "relationships",
            "resolver_diagnostics",
            "project",
            None,
        ),
        coverage_entry(
            profile_id,
            "erc",
            "schematic_connectivity",
            "schematic",
            None,
        ),
        coverage_entry(profile_id, "drc", "board_geometry", "board", None),
        coverage_entry(
            profile_id,
            "standards",
            "zone_fill_state",
            "board_zones",
            Some("datum.zone_fill_honesty.current"),
        ),
        coverage_entry(
            profile_id,
            "standards",
            "process_aperture_policy",
            "board_pads_tracks_vias",
            Some("datum.process_aperture_and_geometry.current"),
        ),
        coverage_entry(
            profile_id,
            "manufacturing",
            "artifact_validation",
            "generated_artifacts",
            None,
        ),
        coverage_entry(profile_id, "drc", "clearance_copper", "board_copper", None),
        coverage_entry(
            profile_id,
            "drc",
            "silk_clearance_copper",
            "board_silkscreen",
            None,
        ),
        not_implemented_entry(
            "erc",
            "hierarchical_power_intent",
            "schematic_hierarchy",
            None,
        ),
    ]
    .into_iter()
    .collect()
}

fn native_check_profile_descriptor(profile_id: &str) -> NativeProjectCheckProfileView {
    native_check_profile_descriptors()
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .unwrap_or(NativeProjectCheckProfileView {
            profile_id: NATIVE_COMBINED_CHECK_PROFILE,
            name: "Native Combined",
            status: "current_default",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Current deterministic native-project profile combining resolver diagnostics, ERC, DRC, standards, and artifact validation findings.",
            selection_supported: true,
        })
}

fn native_check_profile_descriptors() -> Vec<NativeProjectCheckProfileView> {
    vec![
        NativeProjectCheckProfileView {
            profile_id: "native-combined",
            name: "Native Combined",
            status: "current_default",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Current deterministic native-project profile combining resolver diagnostics, ERC, DRC, standards, and artifact validation findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "erc",
            name: "ERC",
            status: "supported",
            domains: vec!["erc"],
            description: "Electrical-rule focused profile over native schematic connectivity findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "drc",
            name: "DRC",
            status: "supported",
            domains: vec!["drc"],
            description: "Physical-rule focused profile over board geometry findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "standards",
            name: "Standards",
            status: "supported",
            domains: vec!["standards"],
            description: "Standards-focused profile for process-aperture, track-width, via-geometry, and ZoneFill honesty findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "manufacturing",
            name: "Manufacturing",
            status: "supported",
            domains: vec!["manufacturing"],
            description: "Manufacturing-evidence profile for generated artifact validation findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "release",
            name: "Release Readiness",
            status: "supported",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Readiness-only CheckRun profile over deterministic native domains; it does not issue a Release or create revision authority.",
            selection_supported: true,
        },
    ]
}

fn coverage_entry(
    profile_id: &str,
    domain: &str,
    rule_id: &str,
    target_scope: &str,
    standards_basis: Option<&str>,
) -> CheckRunCoverageEntry {
    CheckRunCoverageEntry {
        domain: domain.to_string(),
        rule_id: rule_id.to_string(),
        status: coverage_status_for_profile(profile_id, domain, rule_id).to_string(),
        target_scope: target_scope.to_string(),
        basis_id: Some(format!("datum.check.coverage.{domain}.{rule_id}.v1")),
        rule_revision: Some("v1".to_string()),
        standards_basis: standards_basis.map(str::to_string),
        standards_basis_detail: standards_basis.map(standards_basis_detail),
    }
}

fn not_implemented_entry(
    domain: &str,
    rule_id: &str,
    target_scope: &str,
    basis_id: Option<&str>,
) -> CheckRunCoverageEntry {
    CheckRunCoverageEntry {
        domain: domain.to_string(),
        rule_id: rule_id.to_string(),
        status: "not_implemented".to_string(),
        target_scope: target_scope.to_string(),
        basis_id: basis_id.map(str::to_string),
        rule_revision: None,
        standards_basis: None,
        standards_basis_detail: None,
    }
}

fn coverage_status_for_profile(profile_id: &str, domain: &str, rule_id: &str) -> &'static str {
    match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE => "evaluated",
        ERC_CHECK_PROFILE => {
            if domain == "erc" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        DRC_CHECK_PROFILE => {
            if domain == "drc" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        MANUFACTURING_CHECK_PROFILE => {
            if domain == "manufacturing" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        STANDARDS_CHECK_PROFILE
            if domain == "standards"
                && matches!(rule_id, "process_aperture_policy" | "zone_fill_state") =>
        {
            "evaluated"
        }
        _ => "filtered_by_profile",
    }
}
