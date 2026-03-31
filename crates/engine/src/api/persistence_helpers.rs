use super::*;

pub(super) fn deterministic_net_class_uuid(net_uuid: Uuid, class_name: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("{net_uuid}:{class_name}").as_bytes(),
    )
}

pub(super) fn persist_rule_sidecar(
    board_path: &Path,
    board_contents: &str,
    rules: Vec<Rule>,
    loaded_rule_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = rules_sidecar::sidecar_path_for_source(board_path)?;
    if rules.is_empty() {
        if loaded_rule_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar = rules_sidecar::RuleSidecar::new(source_file, source_hash, rules);
    rules_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

pub(super) fn persist_part_assignment_sidecar(
    board_path: &Path,
    board_contents: &str,
    assignments: BTreeMap<uuid::Uuid, uuid::Uuid>,
    loaded_part_assignment_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = part_assignments_sidecar::sidecar_path_for_source(board_path);
    if assignments.is_empty() {
        if loaded_part_assignment_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar = part_assignments_sidecar::PartAssignmentsSidecar::new(
        source_file,
        source_hash,
        assignments,
    );
    part_assignments_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

pub(super) fn persist_package_assignment_sidecar(
    board_path: &Path,
    board_contents: &str,
    assignments: BTreeMap<uuid::Uuid, uuid::Uuid>,
    loaded_package_assignment_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = package_assignments_sidecar::sidecar_path_for_source(board_path);
    if assignments.is_empty() {
        if loaded_package_assignment_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar = package_assignments_sidecar::PackageAssignmentsSidecar::new(
        source_file,
        source_hash,
        assignments,
    );
    package_assignments_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

pub(super) fn net_class_sidecar_payload(board: &Board) -> (Vec<NetClass>, BTreeMap<Uuid, Uuid>) {
    let assignments: BTreeMap<Uuid, Uuid> = board
        .nets
        .values()
        .filter(|net| net.class != Uuid::nil())
        .map(|net| (net.uuid, net.class))
        .collect();
    let classes = assignments
        .values()
        .filter_map(|uuid| board.net_classes.get(uuid).cloned())
        .collect();
    (classes, assignments)
}

pub(super) fn persist_net_class_sidecar(
    board_path: &Path,
    board_contents: &str,
    payload: (Vec<NetClass>, BTreeMap<Uuid, Uuid>),
    loaded_net_class_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = net_classes_sidecar::sidecar_path_for_source(board_path);
    let (classes, assignments) = payload;
    if classes.is_empty() || assignments.is_empty() {
        if loaded_net_class_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar =
        net_classes_sidecar::NetClassesSidecar::new(source_file, source_hash, classes, assignments);
    net_classes_sidecar::write_sidecar(&sidecar_path, &sidecar)
}
