use std::collections::BTreeMap;
use std::path::Path;

use super::*;

impl Engine {
    /// M1 import dispatch. Recognizes supported design/library file kinds and
    /// routes them to the matching importer slice.
    pub fn import(&mut self, path: &Path) -> Result<ImportReport, EngineError> {
        match detect_import_kind(path) {
            Some(ImportKind::EagleLibrary) => self.import_eagle_library(path),
            Some(ImportKind::KiCadBoard) => {
                let (mut board, report) = kicad::import_board_document(path)?;
                let original_contents = std::fs::read_to_string(path)?;
                let source_hash =
                    ids_sidecar::compute_source_hash_bytes(original_contents.as_bytes());
                let rule_sidecar_path = rules_sidecar::sidecar_path_for_source(path);
                let loaded_rule_sidecar = if rule_sidecar_path.exists() {
                    match rules_sidecar::read_sidecar(&rule_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            board.rules = sidecar.rules;
                            true
                        }
                        Ok(_) => false,
                        Err(_) => false,
                    }
                } else {
                    false
                };
                self.design = Some(Design {
                    board: Some(board),
                    schematic: None,
                });
                let mut loaded_package_assignment_sidecar = false;
                let package_sidecar_path =
                    package_assignments_sidecar::sidecar_path_for_source(path);
                if package_sidecar_path.exists() {
                    match package_assignments_sidecar::read_sidecar(&package_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for (component_uuid, package_uuid) in sidecar.assignments {
                                    if let Some(package) = board.packages.get_mut(&component_uuid) {
                                        package.package = package_uuid;
                                    }
                                }
                                loaded_package_assignment_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                let mut loaded_part_assignment_sidecar = false;
                let part_sidecar_path = part_assignments_sidecar::sidecar_path_for_source(path);
                if part_sidecar_path.exists() {
                    match part_assignments_sidecar::read_sidecar(&part_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for (component_uuid, part_uuid) in sidecar.assignments {
                                    if let Some(package) = board.packages.get_mut(&component_uuid) {
                                        package.part = part_uuid;
                                    }
                                }
                                loaded_part_assignment_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                let mut loaded_net_class_sidecar = false;
                let net_class_sidecar_path = net_classes_sidecar::sidecar_path_for_source(path);
                if net_class_sidecar_path.exists() {
                    match net_classes_sidecar::read_sidecar(&net_class_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for class in sidecar.classes {
                                    board.net_classes.insert(class.uuid, class);
                                }
                                for (net_uuid, class_uuid) in sidecar.assignments {
                                    if board.net_classes.contains_key(&class_uuid)
                                        && let Some(net) = board.nets.get_mut(&net_uuid)
                                    {
                                        net.class = class_uuid;
                                    }
                                }
                                loaded_net_class_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                self.imported_source = Some(ImportedDesignSource {
                    kind: ImportKind::KiCadBoard,
                    source_path: path.to_path_buf(),
                    original_contents,
                    loaded_rule_sidecar,
                    loaded_package_assignment_sidecar,
                    loaded_part_assignment_sidecar,
                    loaded_net_class_sidecar,
                });
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.undo_depth = 0;
                self.redo_depth = 0;
                Ok(report)
            }
            Some(ImportKind::KiCadSchematic) => {
                let (schematic, report) = kicad::import_schematic_document(path)?;
                let original_contents = std::fs::read_to_string(path)?;
                self.design = Some(Design {
                    board: None,
                    schematic: Some(schematic),
                });
                self.imported_source = Some(ImportedDesignSource {
                    kind: ImportKind::KiCadSchematic,
                    source_path: path.to_path_buf(),
                    original_contents,
                    loaded_rule_sidecar: false,
                    loaded_package_assignment_sidecar: false,
                    loaded_part_assignment_sidecar: false,
                    loaded_net_class_sidecar: false,
                });
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.undo_depth = 0;
                self.redo_depth = 0;
                Ok(report)
            }
            Some(ImportKind::KiCadProject) => kicad::import_project_file(path),
            Some(ImportKind::EagleBoard) => eagle::import_board_file(path),
            Some(ImportKind::EagleSchematic) => eagle::import_schematic_file(path),
            None => Err(EngineError::Import(format!(
                "unsupported import path {}; expected .lbr, .kicad_pcb, .kicad_sch, .kicad_pro, .brd, or .sch",
                path.display()
            ))),
        }
    }

    /// M0 Eagle library import into the in-memory pool.
    pub fn import_eagle_library(&mut self, path: &Path) -> Result<ImportReport, EngineError> {
        let (imported, report) = eagle::import_library_file(path)?;
        self.merge_pool(imported);
        self.pool_index.rebuild_from_pool(&self.pool)?;
        Ok(report)
    }

    pub fn search_pool(&self, query: &str) -> Result<Vec<PartSummary>, EngineError> {
        Ok(self.pool_index.search_keyword(query)?)
    }

    pub fn get_part(&self, uuid: &uuid::Uuid) -> Result<PartDetail, EngineError> {
        let part = self.pool.parts.get(uuid).ok_or(EngineError::NotFound {
            object_type: "part",
            uuid: *uuid,
        })?;
        let entity =
            self.pool
                .entities
                .get(&part.entity)
                .ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: part.uuid,
                    target_type: "entity",
                    target_uuid: part.entity,
                })?;
        let package =
            self.pool
                .packages
                .get(&part.package)
                .ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: part.uuid,
                    target_type: "package",
                    target_uuid: part.package,
                })?;

        let mut gates: Vec<_> = entity
            .gates
            .values()
            .map(|gate| {
                let mut pins: Vec<String> = self
                    .pool
                    .units
                    .get(&gate.unit)
                    .map(|unit| unit.pins.values().map(|pin| pin.name.clone()).collect())
                    .unwrap_or_default();
                pins.sort();
                PartGateDetail {
                    name: gate.name.clone(),
                    pins,
                }
            })
            .collect();
        gates.sort_by(|a, b| a.name.cmp(&b.name));

        let mut parametric = BTreeMap::new();
        for (key, value) in &part.parametric {
            parametric.insert(key.clone(), value.clone());
        }

        Ok(PartDetail {
            uuid: part.uuid,
            mpn: part.mpn.clone(),
            manufacturer: part.manufacturer.clone(),
            value: part.value.clone(),
            description: part.description.clone(),
            datasheet: part.datasheet.clone(),
            entity: PartEntityDetail {
                name: entity.name.clone(),
                prefix: entity.prefix.clone(),
                gates,
            },
            package: PartPackageDetail {
                uuid: package.uuid,
                name: package.name.clone(),
                pads: package.pads.len(),
            },
            parametric,
            lifecycle: match part.lifecycle {
                crate::pool::Lifecycle::Active => PartLifecycle::Active,
                crate::pool::Lifecycle::Nrnd => PartLifecycle::Nrnd,
                crate::pool::Lifecycle::Eol => PartLifecycle::Eol,
                crate::pool::Lifecycle::Obsolete => PartLifecycle::Obsolete,
                crate::pool::Lifecycle::Unknown => PartLifecycle::Unknown,
            },
        })
    }

    pub fn get_package(&self, uuid: &uuid::Uuid) -> Result<PackageDetail, EngineError> {
        let package = self.pool.packages.get(uuid).ok_or(EngineError::NotFound {
            object_type: "package",
            uuid: *uuid,
        })?;
        let mut pads: Vec<_> = package
            .pads
            .values()
            .map(|pad| PackagePadDetail {
                name: pad.name.clone(),
                x_mm: nm_to_mm(pad.position.x),
                y_mm: nm_to_mm(pad.position.y),
                layer: pad.layer.to_string(),
            })
            .collect();
        pads.sort_by(|a, b| a.name.cmp(&b.name));
        let courtyard = package
            .courtyard
            .bounding_box()
            .map(|bbox| PackageCourtyardDetail {
                width: nm_to_mm(bbox.width()),
                height: nm_to_mm(bbox.height()),
            })
            .unwrap_or(PackageCourtyardDetail {
                width: 0.0,
                height: 0.0,
            });
        Ok(PackageDetail {
            uuid: package.uuid,
            name: package.name.clone(),
            pads,
            courtyard_mm: courtyard,
        })
    }

    pub fn get_package_change_candidates(
        &self,
        component_uuid: &uuid::Uuid,
    ) -> Result<PackageChangeCompatibilityReport, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let current_package_name = self
            .pool
            .packages
            .get(&component.package)
            .map(|package| package.name.clone())
            .unwrap_or_default();

        if component.part == uuid::Uuid::nil() || !self.pool.parts.contains_key(&component.part) {
            return Ok(PackageChangeCompatibilityReport {
                component_uuid: *component_uuid,
                current_part_uuid: None,
                current_package_uuid: component.package,
                current_package_name,
                current_value: component.value.clone(),
                status: PackageChangeCompatibilityStatus::NoKnownPart,
                ambiguous_package_count: 0,
                candidates: Vec::new(),
            });
        }

        let current_part_uuid = component.part;
        let current_part = self.pool.parts.get(&current_part_uuid).ok_or(EngineError::NotFound {
            object_type: "part",
            uuid: current_part_uuid,
        })?;
        let current_signature = part_pin_signature(current_part, &self.pool).ok_or_else(|| {
            EngineError::DanglingReference {
                source_type: "part",
                source_uuid: current_part_uuid,
                target_type: "entity",
                target_uuid: current_part.entity,
            }
        })?;

        let mut by_package: BTreeMap<uuid::Uuid, Vec<&crate::pool::Part>> = BTreeMap::new();
        for part in self.pool.parts.values() {
            if part.package == component.package {
                continue;
            }
            if part_pin_signature(part, &self.pool).as_ref() == Some(&current_signature) {
                by_package.entry(part.package).or_default().push(part);
            }
        }

        let mut ambiguous_package_count = 0;
        let mut candidates = Vec::new();
        for (package_uuid, parts) in by_package {
            if parts.len() != 1 {
                ambiguous_package_count += 1;
                continue;
            }
            let part = parts[0];
            let package = self.pool.packages.get(&package_uuid).ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: package_uuid,
            })?;
            let mut pin_names: Vec<_> = current_signature.iter().cloned().collect();
            pin_names.sort();
            candidates.push(PackageChangeCandidate {
                package_uuid,
                package_name: package.name.clone(),
                compatible_part_uuid: part.uuid,
                compatible_part_value: part.value.clone(),
                pin_names,
            });
        }
        candidates.sort_by(|a, b| {
            a.package_name
                .cmp(&b.package_name)
                .then_with(|| a.package_uuid.cmp(&b.package_uuid))
        });

        Ok(PackageChangeCompatibilityReport {
            component_uuid: *component_uuid,
            current_part_uuid: Some(current_part_uuid),
            current_package_uuid: component.package,
            current_package_name,
            current_value: component.value.clone(),
            status: if candidates.is_empty() {
                PackageChangeCompatibilityStatus::NoCompatiblePackages
            } else {
                PackageChangeCompatibilityStatus::CandidatesAvailable
            },
            ambiguous_package_count,
            candidates,
        })
    }

    pub fn close_project(&mut self) {
        self.design = None;
        self.imported_source = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.undo_depth = 0;
        self.redo_depth = 0;
    }

    pub fn explain_violation(
        &self,
        domain: ViolationDomain,
        index: usize,
    ) -> Result<ViolationExplanation, EngineError> {
        match domain {
            ViolationDomain::Erc => {
                let findings = self.run_erc_prechecks()?;
                let finding = findings.get(index).ok_or_else(|| {
                    EngineError::Validation(format!(
                        "erc finding index {index} is out of range ({} findings)",
                        findings.len()
                    ))
                })?;
                let objects_involved = finding
                    .object_uuids
                    .iter()
                    .enumerate()
                    .map(|(i, uuid)| {
                        let descriptor = finding.objects.get(i);
                        ViolationObjectInfo {
                            type_name: descriptor
                                .map(|obj| obj.kind.to_string())
                                .unwrap_or_else(|| "object".to_string()),
                            uuid: *uuid,
                            description: descriptor
                                .map(|obj| obj.key.clone())
                                .unwrap_or_else(|| uuid.to_string()),
                        }
                    })
                    .collect();
                Ok(ViolationExplanation {
                    explanation: finding.message.clone(),
                    rule_detail: format!("erc {} ({:?})", finding.code, finding.severity),
                    objects_involved,
                    suggestion: erc_suggestion(finding.code).to_string(),
                })
            }
            ViolationDomain::Drc => {
                let report = self.run_drc(&[
                    RuleType::Connectivity,
                    RuleType::ClearanceCopper,
                    RuleType::TrackWidth,
                    RuleType::ViaHole,
                    RuleType::ViaAnnularRing,
                    RuleType::SilkClearance,
                ])?;
                let violation = report.violations.get(index).ok_or_else(|| {
                    EngineError::Validation(format!(
                        "drc violation index {index} is out of range ({} violations)",
                        report.violations.len()
                    ))
                })?;
                let objects_involved = violation
                    .objects
                    .iter()
                    .map(|uuid| ViolationObjectInfo {
                        type_name: "board_object".to_string(),
                        uuid: *uuid,
                        description: uuid.to_string(),
                    })
                    .collect();
                Ok(ViolationExplanation {
                    explanation: violation.message.clone(),
                    rule_detail: format!(
                        "drc {} ({:?}, {:?})",
                        violation.code, violation.rule_type, violation.severity
                    ),
                    objects_involved,
                    suggestion: drc_suggestion(&violation.code).to_string(),
                })
            }
        }
    }
}
