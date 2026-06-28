use super::*;

pub(crate) fn execute_project_import_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ImportKicadFootprint(ProjectImportKiCadFootprintArgs {
            path,
            source,
            pool,
        }) => Ok((
            render_output(
                format,
                &import_native_project_kicad_footprint(&path, &source, &pool)?,
            ),
            0,
        )),
        ProjectCommands::ImportKicadBoard(ProjectImportKiCadBoardArgs { path, source }) => Ok((
            render_output(format, &import_native_project_kicad_board(&path, &source)?),
            0,
        )),
        ProjectCommands::ImportKicadSchematic(ProjectImportKiCadSchematicArgs { path, source }) => {
            Ok((
                render_output(
                    format,
                    &import_native_project_kicad_schematic(&path, &source)?,
                ),
                0,
            ))
        }
        ProjectCommands::ImportEagleLibrary(ProjectImportEagleLibraryArgs {
            path,
            source,
            pool,
        }) => Ok((
            render_output(
                format,
                &import_native_project_eagle_library(&path, &source, &pool)?,
            ),
            0,
        )),
        _ => unreachable!("project import dispatcher received non-import command"),
    }
}
