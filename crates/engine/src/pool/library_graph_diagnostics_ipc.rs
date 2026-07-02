use super::library_graph::{LibraryGraph, LibraryGraphDiagnostic};
use super::{
    Footprint, IpcFootprintBasis, Padstack, PadstackAperture, PadstackMaskPolicy,
    PadstackPastePolicy,
};

impl LibraryGraph {
    pub(super) fn validate_ipc_footprint_basis(
        &self,
        footprint: &serde_json::Value,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(basis_value) = footprint.get("ipc_basis") else {
            return;
        };
        if basis_value.is_null() {
            return;
        }
        let basis = match serde_json::from_value::<IpcFootprintBasis>(basis_value.clone()) {
            Ok(basis) => basis,
            Err(error) => {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_ipc_footprint_basis",
                    subject: format!("{subject}#ipc_basis"),
                    message: format!(
                        "ipc_basis does not match the IPC footprint basis schema: {error}"
                    ),
                });
                return;
            }
        };
        self.validate_ipc_basis_shape(&basis, subject, diagnostics);
        if footprint
            .get("standards_basis")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|standards_basis| standards_basis != basis.naming_basis)
        {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "ipc_basis_standards_mismatch",
                subject: subject.to_string(),
                message: "footprint standards_basis does not match ipc_basis naming_basis"
                    .to_string(),
            });
        }
        if basis.family == "IPC-7351"
            && basis.revision == "B"
            && basis.package_family == "two_terminal_chip"
        {
            self.validate_ipc_two_terminal_chip_footprint(footprint, &basis, subject, diagnostics);
        }
    }

    fn validate_ipc_basis_shape(
        &self,
        basis: &IpcFootprintBasis,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let basis_subject = format!("{subject}#ipc_basis");
        for (field, value) in [
            ("family", basis.family.as_str()),
            ("revision", basis.revision.as_str()),
            ("package_family", basis.package_family.as_str()),
            ("package_code", basis.package_code.as_str()),
            ("derivation_version", basis.derivation_version.as_str()),
            ("naming_basis", basis.naming_basis.as_str()),
        ] {
            if value.trim().is_empty() {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_ipc_footprint_basis",
                    subject: basis_subject.clone(),
                    message: format!("ipc_basis field {field} must be non-empty"),
                });
            }
        }
        for (field, value) in [
            ("body_length_nm", basis.source_dimensions.body_length_nm),
            ("body_width_nm", basis.source_dimensions.body_width_nm),
            (
                "terminal_length_nm",
                basis.source_dimensions.terminal_length_nm,
            ),
            (
                "terminal_width_nm",
                basis.source_dimensions.terminal_width_nm,
            ),
        ] {
            if value <= 0 {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_ipc_footprint_basis",
                    subject: basis_subject.clone(),
                    message: format!("ipc_basis source_dimensions.{field} must be positive"),
                });
            }
        }
        if basis.courtyard_excess_nm < 0
            || basis.mask_expansion_nm < 0
            || basis.paste_reduction_nm < 0
        {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "invalid_ipc_footprint_basis",
                subject: basis_subject,
                message: "ipc_basis courtyard/mask/paste policy values must not be negative"
                    .to_string(),
            });
        }
    }

    fn validate_ipc_two_terminal_chip_footprint(
        &self,
        footprint: &serde_json::Value,
        basis: &IpcFootprintBasis,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let footprint = match serde_json::from_value::<Footprint>(footprint.clone()) {
            Ok(footprint) => footprint,
            Err(error) => {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_ipc_footprint_basis",
                    subject: subject.to_string(),
                    message: format!(
                        "IPC footprint payload cannot be decoded for validation: {error}"
                    ),
                });
                return;
            }
        };
        if footprint.pads.len() != 2 {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "ipc_basis_geometry_mismatch",
                subject: subject.to_string(),
                message: "IPC-7351B two-terminal chip footprint must contain exactly two pads"
                    .to_string(),
            });
        }
        let expected_pad_length = basis.source_dimensions.terminal_length_nm
            + basis.source_j_values.toe_nm
            + basis.source_j_values.heel_nm;
        let expected_pad_width =
            basis.source_dimensions.terminal_width_nm + (2 * basis.source_j_values.side_nm);
        for (pad_id, pad) in &footprint.pads {
            let Some(padstack_value) = self.padstacks.get(&pad.padstack) else {
                continue;
            };
            let pad_subject = format!("{subject}#pads/{pad_id}");
            let Ok(padstack) = serde_json::from_value::<Padstack>(padstack_value.clone()) else {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_ipc_footprint_basis",
                    subject: pad_subject,
                    message: "referenced IPC padstack cannot be decoded for validation".to_string(),
                });
                continue;
            };
            if padstack.aperture
                != Some(PadstackAperture::Rect {
                    width_nm: expected_pad_length,
                    height_nm: expected_pad_width,
                })
            {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "ipc_basis_geometry_mismatch",
                    subject: pad_subject.clone(),
                    message: "IPC padstack aperture does not match derived toe/heel/side geometry"
                        .to_string(),
                });
            }
            if padstack.mask_policy != PadstackMaskPolicy::ExpansionNm(basis.mask_expansion_nm) {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "ipc_basis_process_policy_mismatch",
                    subject: pad_subject.clone(),
                    message: "IPC padstack mask policy does not match ipc_basis mask_expansion_nm"
                        .to_string(),
                });
            }
            if padstack.paste_policy
                != PadstackPastePolicy::ExpansionNm(-basis.paste_reduction_nm.abs())
            {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "ipc_basis_process_policy_mismatch",
                    subject: pad_subject,
                    message:
                        "IPC padstack paste policy does not match ipc_basis paste_reduction_nm"
                            .to_string(),
                });
            }
        }
    }
}
