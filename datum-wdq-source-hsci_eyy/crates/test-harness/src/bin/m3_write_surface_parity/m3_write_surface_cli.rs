use super::*;

#[path = "../m3_write_surface_cli/m3_write_surface_cli_assign_package/mod.rs"]
mod m3_write_surface_cli_assign_package;
#[path = "../m3_write_surface_cli/m3_write_surface_cli_core.rs"]
mod m3_write_surface_cli_core;
#[path = "../m3_write_surface_cli/m3_write_surface_cli_motion_netclass.rs"]
mod m3_write_surface_cli_motion_netclass;

pub(super) fn check_cli_surface(cli: &Cli) -> SurfaceCheck {
    if let Ok(evidence) = cli_retired_persistence_surface_result(cli) {
        return SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Passed,
            evidence,
        };
    }

    match (
        m3_write_surface_cli_core::cli_surface_result(cli),
        m3_write_surface_cli_core::cli_via_surface_result(cli),
        m3_write_surface_cli_core::cli_component_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_move_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_rotate_surface_result(cli),
        m3_write_surface_cli_core::cli_rule_surface_result(cli),
        m3_write_surface_cli_core::cli_value_surface_result(cli),
        m3_write_surface_cli_core::cli_reference_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_assign_part_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_assign_part_remap_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_remap_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_with_part_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_set_net_class_surface_result(cli),
    ) {
        (
            Ok(track_evidence),
            Ok(via_evidence),
            Ok(component_evidence),
            Ok(move_evidence),
            Ok(rotate_evidence),
            Ok(rule_evidence),
            Ok(value_evidence),
            Ok(reference_evidence),
            Ok(assign_part_evidence),
            Ok(assign_part_remap_evidence),
            Ok(set_package_evidence),
            Ok(set_package_remap_evidence),
            Ok(set_package_with_part_evidence),
            Ok(net_class_evidence),
        ) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Passed,
            evidence: format!(
                "{track_evidence}; {via_evidence}; {component_evidence}; {move_evidence}; {rotate_evidence}; {rule_evidence}; {value_evidence}; {reference_evidence}; {assign_part_evidence}; {assign_part_remap_evidence}; {set_package_evidence}; {set_package_remap_evidence}; {set_package_with_part_evidence}; {net_class_evidence}"
            ),
        },
        (Err(err), _, _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, Err(err), _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, Err(err), _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, Err(err), _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, Err(err), _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, Err(err), _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, Err(err), _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, Err(err), _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, Err(err), _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, Err(err), _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, Err(err), _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, Err(err), _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, Err(err), _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, _, Err(err)) => {
            let evidence = err.to_string();
            if evidence.contains("legacy KiCad modify persistence is retired") {
                SurfaceCheck {
                    surface: "cli_modify_surface".to_string(),
                    status: Status::Passed,
                    evidence,
                }
            } else {
                SurfaceCheck {
                    surface: "cli_modify_surface".to_string(),
                    status: Status::Failed,
                    evidence,
                }
            }
        }
    }
}

fn cli_retired_persistence_surface_result(cli: &Cli) -> Result<String> {
    let roundtrip_output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "datum-eda-cli",
            "--",
            "--format",
            "json",
            "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--undo")
        .arg("1")
        .arg("--redo")
        .arg("1")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI non-persistence parity probe")?;
    if !roundtrip_output.status.success() {
        bail!(
            "CLI non-persistence parity probe failed with status {:?}: {}",
            roundtrip_output.status.code(),
            String::from_utf8_lossy(&roundtrip_output.stderr).trim()
        );
    }
    let roundtrip: CliModifyReport = serde_json::from_slice(&roundtrip_output.stdout)
        .context("failed to parse CLI non-persistence JSON output")?;
    let expected_last_description = format!("redo delete_track {}", cli.track_uuid);
    if roundtrip
        .last_result
        .as_ref()
        .map(|result| result.description.as_str())
        != Some(expected_last_description.as_str())
    {
        bail!("CLI non-persistence last_result description mismatch");
    }

    let target = unique_temp_path("cli-retired-save-probe", "kicad_pcb");
    let save_output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "datum-eda-cli",
            "--",
            "--format",
            "json",
            "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI retired save guard probe")?;
    if save_output.status.success() {
        bail!("CLI retired save guard unexpectedly allowed legacy persistence");
    }
    let stderr = String::from_utf8_lossy(&save_output.stderr);
    if !stderr.contains("legacy KiCad modify persistence is retired for production builds") {
        bail!(
            "CLI retired save guard returned unexpected message: {}",
            stderr.trim()
        );
    }

    Ok(format!(
        "non_persistent_roundtrip_last={expected_last_description}, legacy_kicad_modify_save_retired=true"
    ))
}
