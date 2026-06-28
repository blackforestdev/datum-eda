use super::*;

pub(crate) fn execute_check_command(
    format: &OutputFormat,
    action: CheckCommands,
) -> Result<(String, i32)> {
    match action {
        CheckCommands::Run(CheckRunArgs { path, profile }) => Ok((
            render_output(
                format,
                &run_native_project_check_with_profile(&path, profile.as_deref())?,
            ),
            0,
        )),
        CheckCommands::List(CheckListArgs { path }) => Ok((
            render_output(format, &query_native_project_check_run_list(&path)?),
            0,
        )),
        CheckCommands::Show(CheckShowArgs { path, check_run }) => Ok((
            render_output(
                format,
                &query_native_project_check_run_show(&path, check_run)?,
            ),
            0,
        )),
        CheckCommands::Profiles(CheckProfilesArgs { path }) => Ok((
            render_output(format, &query_native_project_check_profiles(&path)?),
            0,
        )),
        CheckCommands::FillZones(CheckFillZonesArgs {
            path,
            zone_uuid,
            net_uuid,
        }) => Ok((
            render_output(
                format,
                &fill_native_project_zones(&path, zone_uuid, net_uuid)?,
            ),
            0,
        )),
        CheckCommands::RepairStandards(CheckRepairStandardsArgs { path }) => Ok((
            render_output(
                format,
                &generate_native_project_standards_repair_proposals(&path)?,
            ),
            0,
        )),
        CheckCommands::Waive(CheckWaiveArgs {
            path,
            fingerprint,
            rationale,
            created_by,
        }) => Ok((
            render_output(
                format,
                &waive_native_project_finding(&path, &fingerprint, &rationale, created_by)?,
            ),
            0,
        )),
        CheckCommands::AcceptDeviation(CheckAcceptDeviationArgs {
            path,
            fingerprint,
            rationale,
            accepted_by,
        }) => Ok((
            render_output(
                format,
                &accept_native_project_deviation(&path, &fingerprint, &rationale, accepted_by)?,
            ),
            0,
        )),
        CheckCommands::Imported(CheckImportedArgs { path, fail_on }) => {
            let report = run_check(&path)?;
            let output = match format {
                OutputFormat::Text => render_check_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, check_exit_code(&report, fail_on)))
        }
    }
}
