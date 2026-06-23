use super::*;

pub(crate) fn execute_project_import_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    let ProjectCommands::ImportKicadFootprint(ProjectImportKiCadFootprintArgs {
        path,
        source,
        pool,
    }) = command
    else {
        unreachable!("project import dispatcher received non-import command");
    };
    Ok((
        render_output(
            format,
            &import_native_project_kicad_footprint(&path, &source, &pool)?,
        ),
        0,
    ))
}
