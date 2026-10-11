//! Manual Gerber layer lifecycle: execute, render text/JSON and preserve exit status.
use crate::*;

impl ProjectExportGerberOutlineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, out } = self;

        let report = export_native_project_gerber_outline(&path, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_outline_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectExportGerberCopperLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layer, out } = self;

        let report = export_native_project_gerber_copper_layer(&path, layer, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_copper_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectExportGerberSoldermaskLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layer, out } = self;

        let report = export_native_project_gerber_soldermask_layer(&path, layer, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_soldermask_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectExportGerberSilkscreenLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layer, out } = self;

        let report = export_native_project_gerber_silkscreen_layer(&path, layer, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_silkscreen_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectExportGerberPasteLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layer, out } = self;

        let report = export_native_project_gerber_paste_layer(&path, layer, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_paste_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectExportGerberMechanicalLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, layer, out } = self;

        let report = export_native_project_gerber_mechanical_layer(&path, layer, &out)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_mechanical_export_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectValidateGerberOutlineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, gerber } = self;

        let report = validate_native_project_gerber_outline(&path, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_outline_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, if report.matches_expected { 0 } else { 1 }))
    }
}

impl ProjectValidateGerberCopperLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = validate_native_project_gerber_copper_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_copper_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl ProjectValidateGerberSoldermaskLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = validate_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_soldermask_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl ProjectValidateGerberSilkscreenLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = validate_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_silkscreen_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl ProjectValidateGerberPasteLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = validate_native_project_gerber_paste_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_paste_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl ProjectValidateGerberMechanicalLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = validate_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_mechanical_validation_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl ProjectCompareGerberOutlineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, gerber } = self;

        let report = compare_native_project_gerber_outline(&path, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_outline_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareGerberCopperLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = compare_native_project_gerber_copper_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_copper_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareGerberSoldermaskLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = compare_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_soldermask_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareGerberSilkscreenLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = compare_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_silkscreen_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareGerberPasteLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = compare_native_project_gerber_paste_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_paste_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}

impl ProjectCompareGerberMechanicalLayerArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            layer,
            gerber,
        } = self;

        let report = compare_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
        let output = match format {
            OutputFormat::Text => render_native_project_gerber_mechanical_comparison_text(&report),
            OutputFormat::Json => render_output(format, &report),
        };
        Ok((output, 0))
    }
}
