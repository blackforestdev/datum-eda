//! Hierarchy diagnostics consume the canonical per-occurrence membership resolver.
use super::*;
pub(super) fn findings(schematic: &Schematic, config: &ErcConfig) -> Vec<ErcFinding> {
    if schematic.sheet_instances.is_empty() {
        return legacy_mismatch_findings(schematic, config);
    }
    let unbound = match connectivity::occurrence_partition::unbound_interfaces(schematic) {
        Ok(unbound) => unbound,
        Err(error) => {
            return vec![build_finding(
                "hierarchical_connectivity_mismatch",
                severity_for(
                    config,
                    "hierarchical_connectivity_mismatch",
                    ErcSeverity::Error,
                ),
                format!("Hierarchical interface resolution unavailable: {error}"),
                None,
                None,
                None,
                vec![object_ref("schematic", schematic.uuid.to_string())],
                vec![schematic.uuid],
                Vec::new(),
            )];
        }
    };
    unbound.into_iter().map(|(occurrence, sheet_id)| {
        let sheet = &schematic.sheets[&sheet_id];
        let label = &sheet.labels[&occurrence.source_id];
        let path = occurrence.instance_path.iter().map(Uuid::to_string).collect::<Vec<_>>().join("/");
        build_finding(
            "hierarchical_connectivity_mismatch",
            severity_for(config, "hierarchical_connectivity_mismatch", ErcSeverity::Warning),
            format!("Hierarchical label {} is unbound in {} (occurrence {path}); no parent interface connection", label.name, sheet.name),
            None, None, None,
            vec![object_ref("hierarchical_label", label.name.clone()),
                 object_ref("source_uuid", label.uuid.to_string()),
                 object_ref("sheet_uuid", sheet_id.to_string()),
                 object_ref("sheet_occurrence", path)],
            vec![label.uuid], Vec::new(),
        )
    }).collect()
}

fn legacy_mismatch_findings(schematic: &Schematic, config: &ErcConfig) -> Vec<ErcFinding> {
    let mut findings = Vec::new();
    for sheet in schematic.sheets.values() {
        let mut label_names: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
        for label in sheet.labels.values() {
            if matches!(label.kind, crate::schematic::LabelKind::Hierarchical) {
                label_names
                    .entry(label.name.clone())
                    .or_default()
                    .push(label.uuid);
            }
        }
        let mut port_names: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
        for port in sheet.ports.values() {
            port_names
                .entry(port.name.clone())
                .or_default()
                .push(port.uuid);
        }

        if port_names.is_empty() {
            continue;
        }

        let missing_ports: Vec<_> = label_names
            .keys()
            .filter(|name| !port_names.contains_key(*name))
            .cloned()
            .collect();
        let missing_labels: Vec<_> = port_names
            .keys()
            .filter(|name| !label_names.contains_key(*name))
            .cloned()
            .collect();

        if missing_ports.is_empty() && missing_labels.is_empty() {
            continue;
        }

        let mut object_uuids = Vec::new();
        for name in &missing_ports {
            if let Some(uuids) = label_names.get(name) {
                object_uuids.extend(uuids.iter().copied());
            }
        }
        for name in &missing_labels {
            if let Some(uuids) = port_names.get(name) {
                object_uuids.extend(uuids.iter().copied());
            }
        }
        object_uuids.sort();
        object_uuids.dedup();

        let mut objects = vec![object_ref("sheet", sheet.name.clone())];
        for name in &missing_ports {
            objects.push(object_ref("hierarchical_label", name.clone()));
        }
        for name in &missing_labels {
            objects.push(object_ref("port", name.clone()));
        }

        let mut message_parts = Vec::new();
        if !missing_ports.is_empty() {
            message_parts.push(format!(
                "labels without matching ports: {}",
                missing_ports.join(", ")
            ));
        }
        if !missing_labels.is_empty() {
            message_parts.push(format!(
                "ports without matching labels: {}",
                missing_labels.join(", ")
            ));
        }
        findings.push(build_finding(
            "hierarchical_connectivity_mismatch",
            severity_for(
                config,
                "hierarchical_connectivity_mismatch",
                ErcSeverity::Warning,
            ),
            format!(
                "sheet {} has hierarchical interface mismatch ({})",
                sheet.name,
                message_parts.join("; ")
            ),
            None,
            None,
            None,
            objects,
            object_uuids,
            Vec::new(),
        ));
    }
    findings
}
