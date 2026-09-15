use super::*;

pub(super) fn daemon_surface_result(cli: &Cli) -> Result<String> {
    let save_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "save_dispatch_writes_current_m3_slice_to_requested_path",
            ])
            .current_dir(&cli.repo_root),
        "daemon save dispatch parity probe",
    )?;
    let roundtrip_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon roundtrip dispatch parity probe",
    )?;
    let delete_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_dispatch_updates_followup_check_report",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-track derived-state parity probe",
    )?;
    let via_roundtrip_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon via roundtrip dispatch parity probe",
    )?;
    let via_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-via derived-state parity probe",
    )?;
    let component_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_component_dispatch_updates_component_list",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-component parity probe",
    )?;
    let component_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_component_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-component derived-state parity probe",
    )?;
    let rule_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_persists_rule_in_memory",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule dispatch parity probe",
    )?;
    let rule_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_updates_followup_design_rules_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule derived-state parity probe",
    )?;
    let value_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_component_value",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value parity probe",
    )?;
    let value_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value derived-state parity probe",
    )?;
    let reference_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_component_reference",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference parity probe",
    )?;
    let reference_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference derived-state parity probe",
    )?;
    let move_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_component_position",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component parity probe",
    )?;
    let move_derived_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_followup_unrouted_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component derived-state parity probe",
    )?;
    let rotate_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "rotate_component_dispatch_updates_component_rotation",
            ])
            .current_dir(&cli.repo_root),
        "daemon rotate-component parity probe",
    )?;
    let rotate_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "rotate_component_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon rotate-component derived-state parity probe",
    )?;
    let assign_part_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_updates_component_value",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part parity probe",
    )?;
    let assign_part_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part derived-state parity probe",
    )?;
    let assign_part_remap_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_preserves_logical_nets_across_known_part_remap",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part logical-remap parity probe",
    )?;
    let set_package_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_updates_component_package",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package parity probe",
    )?;
    let set_package_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package net-info derived-state parity probe",
    )?;
    let set_package_with_part_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package-with-part parity probe",
    )?;
    let set_package_remap_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_preserves_logical_nets_across_known_part_remap",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package logical-remap parity probe",
    )?;
    let set_net_class_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_net_class_dispatch_updates_net_class",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-net-class parity probe",
    )?;
    let set_net_class_followup_test = m3_write_surface_common::run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_net_class_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-net-class derived-state parity probe",
    )?;

    Ok(format!(
        "behavioral dispatch tests passed: save_dispatch_writes_current_m3_slice_to_requested_path, delete_track_undo_and_redo_dispatch_round_trip, delete_track_dispatch_updates_followup_check_report, delete_via_undo_and_redo_dispatch_round_trip, delete_via_dispatch_updates_followup_net_info_query, delete_component_dispatch_updates_component_list, delete_component_dispatch_updates_followup_components_query, set_design_rule_dispatch_persists_rule_in_memory, set_design_rule_dispatch_updates_followup_design_rules_query, set_value_dispatch_updates_component_value, set_value_dispatch_updates_followup_components_query, set_reference_dispatch_updates_component_reference, set_reference_dispatch_updates_followup_components_query, move_component_dispatch_updates_component_position, move_component_dispatch_updates_followup_unrouted_query, rotate_component_dispatch_updates_component_rotation, rotate_component_dispatch_updates_followup_components_query, assign_part_dispatch_updates_component_value, assign_part_dispatch_updates_followup_net_info_query, assign_part_dispatch_preserves_logical_nets_across_known_part_remap, set_package_dispatch_updates_component_package, set_package_dispatch_updates_followup_net_info_query, set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate, set_package_dispatch_preserves_logical_nets_across_known_part_remap, set_net_class_dispatch_updates_net_class, set_net_class_dispatch_updates_followup_net_info_query (outputs: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
        save_test,
        roundtrip_test,
        delete_followup_test,
        via_roundtrip_test,
        via_followup_test,
        component_test,
        component_followup_test,
        rule_test,
        rule_followup_test,
        value_test,
        value_followup_test,
        reference_test,
        reference_followup_test,
        move_test,
        move_derived_test,
        rotate_test,
        rotate_followup_test,
        assign_part_test,
        assign_part_followup_test,
        assign_part_remap_test,
        set_package_test,
        set_package_followup_test,
        set_package_with_part_test,
        set_package_remap_test,
        set_net_class_test,
        set_net_class_followup_test
    ))
}
