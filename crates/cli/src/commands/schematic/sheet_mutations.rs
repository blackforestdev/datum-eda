use super::connectivity_mutations::commit_schematic_write;
use super::*;
use eda_engine::api::native_write::schematic_sheets::{
    build_create_schematic_definition, build_create_schematic_sheet,
    build_create_schematic_sheet_instance, build_delete_schematic_sheet,
    build_delete_schematic_sheet_instance, build_rename_schematic_sheet,
    build_set_schematic_sheet_instance,
};
use eda_engine::substrate::{CommitReport, ProjectResolver};

pub(crate) fn create_native_project_sheet(
    root: &Path,
    name: String,
    sheet_uuid: Option<Uuid>,
) -> Result<NativeProjectSheetMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("sheet name must not be empty");
    }
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_uuid = sheet_uuid.unwrap_or_else(Uuid::new_v4);
    let sheet_key = sheet_uuid.to_string();
    if project.schematic.sheets.contains_key(&sheet_key) {
        bail!("sheet already exists in native schematic root: {sheet_uuid}");
    }
    let relative_path = format!("sheets/{sheet_uuid}.json");
    if project.root.join("schematic").join(&relative_path).exists() {
        bail!("sheet file already exists: schematic/{relative_path}");
    }
    let sheet = NativeSheetRoot {
        schema_version: 1,
        uuid: sheet_uuid,
        name: name.clone(),
        frame: None,
        symbols: BTreeMap::new(),
        wires: BTreeMap::new(),
        junctions: BTreeMap::new(),
        labels: BTreeMap::new(),
        buses: BTreeMap::new(),
        bus_entries: BTreeMap::new(),
        ports: BTreeMap::new(),
        noconnects: BTreeMap::new(),
        texts: BTreeMap::new(),
        drawings: BTreeMap::new(),
    };
    let report = commit_schematic_write(root, "create schematic sheet", |model, provenance| {
        build_create_schematic_sheet(
            model,
            provenance,
            project.schematic.uuid,
            sheet_uuid,
            &relative_path,
            serde_json::to_value(&sheet).expect("native sheet serialization must succeed"),
        )
    })?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetMutationReportView {
        action: "create_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(&relative_path)
            .display()
            .to_string(),
        name,
        cascaded_objects: 0,
    })
}

pub(crate) fn delete_native_project_sheet(
    root: &Path,
    sheet_uuid: Uuid,
) -> Result<NativeProjectSheetMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let sheet_value = model
        .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
        .with_context(|| format!("failed to materialize {}", sheet_path.display()))?;
    let sheet: NativeSheetRoot = serde_json::from_value(sheet_value.clone())
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;

    let cascaded_objects = sheet_child_object_count(&sheet);
    let report = commit_schematic_write(root, "delete schematic sheet", |model, provenance| {
        build_delete_schematic_sheet(
            model,
            provenance,
            project.schematic.uuid,
            sheet_uuid,
            relative_path,
            sheet_value,
        )
    })?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetMutationReportView {
        action: "delete_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        name: sheet.name,
        cascaded_objects,
    })
}

pub(crate) fn rename_native_project_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
) -> Result<NativeProjectSheetMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("sheet name must not be empty");
    }
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let report = commit_schematic_write(root, "rename schematic sheet", |model, provenance| {
        build_rename_schematic_sheet(model, provenance, sheet_uuid, &name)
    })?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetMutationReportView {
        action: "rename_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        name,
        cascaded_objects: 0,
    })
}

pub(crate) fn create_native_project_sheet_definition(
    root: &Path,
    root_sheet_uuid: Uuid,
    name: String,
    definition_uuid: Option<Uuid>,
) -> Result<NativeProjectSheetDefinitionMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("sheet definition name must not be empty");
    }
    let project = load_native_project_with_resolved_board(root)?;
    let root_sheet_key = root_sheet_uuid.to_string();
    if !project.schematic.sheets.contains_key(&root_sheet_key) {
        bail!("root sheet not found in native schematic root: {root_sheet_uuid}");
    }
    let definition_uuid = definition_uuid.unwrap_or_else(Uuid::new_v4);
    let definition_key = definition_uuid.to_string();
    if project.schematic.definitions.contains_key(&definition_key) {
        bail!("sheet definition already exists in native schematic root: {definition_uuid}");
    }
    let relative_path = format!("definitions/{definition_uuid}.json");
    if project.root.join("schematic").join(&relative_path).exists() {
        bail!("sheet definition file already exists: schematic/{relative_path}");
    }
    let definition = NativeSheetDefinitionRoot {
        schema_version: 1,
        uuid: definition_uuid,
        root_sheet: root_sheet_uuid,
        name: name.clone(),
    };
    let report = commit_schematic_write(
        root,
        "create schematic sheet definition",
        |model, provenance| {
            build_create_schematic_definition(
                model,
                provenance,
                project.schematic.uuid,
                definition_uuid,
                &relative_path,
                serde_json::to_value(&definition)
                    .expect("native sheet definition serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetDefinitionMutationReportView {
        action: "create_sheet_definition".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        definition_uuid: definition_uuid.to_string(),
        definition_path: project
            .root
            .join("schematic")
            .join(&relative_path)
            .display()
            .to_string(),
        root_sheet_uuid: root_sheet_uuid.to_string(),
        name,
    })
}

pub(crate) fn create_native_project_sheet_instance(
    root: &Path,
    definition_uuid: Uuid,
    parent_sheet_uuid: Option<Uuid>,
    name: String,
    x_nm: i64,
    y_nm: i64,
    instance_uuid: Option<Uuid>,
) -> Result<NativeProjectSheetInstanceMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("sheet instance name must not be empty");
    }
    let project = load_native_project_with_resolved_board(root)?;
    if !project
        .schematic
        .definitions
        .contains_key(&definition_uuid.to_string())
    {
        bail!("sheet definition not found in native schematic root: {definition_uuid}");
    }
    if let Some(parent_sheet_uuid) = parent_sheet_uuid {
        if !project
            .schematic
            .sheets
            .contains_key(&parent_sheet_uuid.to_string())
        {
            bail!("parent sheet not found in native schematic root: {parent_sheet_uuid}");
        }
    }
    let instance_uuid = instance_uuid.unwrap_or_else(Uuid::new_v4);
    if project
        .schematic
        .instances
        .iter()
        .any(|instance| instance.uuid == instance_uuid)
    {
        bail!("sheet instance already exists in native schematic root: {instance_uuid}");
    }
    let instance = NativeSchematicInstance {
        uuid: instance_uuid,
        definition: definition_uuid,
        parent_sheet: parent_sheet_uuid,
        position: NativePoint { x: x_nm, y: y_nm },
        name: name.clone(),
        ports: Vec::new(),
    };
    let report = commit_schematic_write(
        root,
        "create schematic sheet instance",
        |model, provenance| {
            build_create_schematic_sheet_instance(
                model,
                provenance,
                project.schematic.uuid,
                instance_uuid,
                serde_json::to_value(&instance)
                    .expect("native sheet instance serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "create_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        instance_uuid: instance_uuid.to_string(),
        definition_uuid: definition_uuid.to_string(),
        parent_sheet_uuid: parent_sheet_uuid.map(|uuid| uuid.to_string()),
        port_uuid: None,
        name,
        x_nm,
        y_nm,
    })
}

pub(crate) fn delete_native_project_sheet_instance(
    root: &Path,
    instance_uuid: Uuid,
) -> Result<NativeProjectSheetInstanceMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    let report = commit_schematic_write(
        root,
        "delete schematic sheet instance",
        |model, provenance| {
            build_delete_schematic_sheet_instance(
                model,
                provenance,
                project.schematic.uuid,
                instance_uuid,
                serde_json::to_value(&instance)
                    .expect("native sheet instance serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "delete_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        instance_uuid: instance.uuid.to_string(),
        definition_uuid: instance.definition.to_string(),
        parent_sheet_uuid: instance.parent_sheet.map(|uuid| uuid.to_string()),
        port_uuid: None,
        name: instance.name,
        x_nm: instance.position.x,
        y_nm: instance.position.y,
    })
}

pub(crate) fn move_native_project_sheet_instance(
    root: &Path,
    instance_uuid: Uuid,
    x_nm: i64,
    y_nm: i64,
) -> Result<NativeProjectSheetInstanceMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let previous_instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    let mut instance = previous_instance.clone();
    instance.position = NativePoint { x: x_nm, y: y_nm };
    let report = commit_schematic_write(
        root,
        "move schematic sheet instance",
        |model, provenance| {
            build_set_schematic_sheet_instance(
                model,
                provenance,
                project.schematic.uuid,
                instance_uuid,
                serde_json::to_value(&previous_instance)
                    .expect("native sheet instance serialization must succeed"),
                serde_json::to_value(&instance)
                    .expect("native sheet instance serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "move_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        instance_uuid: instance.uuid.to_string(),
        definition_uuid: instance.definition.to_string(),
        parent_sheet_uuid: instance.parent_sheet.map(|uuid| uuid.to_string()),
        port_uuid: None,
        name: instance.name,
        x_nm,
        y_nm,
    })
}

pub(crate) fn bind_native_project_sheet_instance_port(
    root: &Path,
    instance_uuid: Uuid,
    port_uuid: Uuid,
) -> Result<NativeProjectSheetInstanceMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (port_sheet_uuid, _port_path, _port_value, _port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    let previous_instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    if previous_instance.parent_sheet != Some(port_sheet_uuid) {
        bail!(
            "sheet instance {instance_uuid} parent sheet does not match port sheet {port_sheet_uuid}"
        );
    }
    if previous_instance.ports.contains(&port_uuid) {
        bail!("sheet instance {instance_uuid} already binds port {port_uuid}");
    }
    let mut instance = previous_instance.clone();
    instance.ports.push(port_uuid);
    instance.ports.sort();
    let report = commit_schematic_write(
        root,
        "bind schematic sheet instance port",
        |model, provenance| {
            build_set_schematic_sheet_instance(
                model,
                provenance,
                project.schematic.uuid,
                instance_uuid,
                serde_json::to_value(&previous_instance)
                    .expect("native sheet instance serialization must succeed"),
                serde_json::to_value(&instance)
                    .expect("native sheet instance serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "bind_sheet_instance_port".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        instance_uuid: instance.uuid.to_string(),
        definition_uuid: instance.definition.to_string(),
        parent_sheet_uuid: instance.parent_sheet.map(|uuid| uuid.to_string()),
        port_uuid: Some(port_uuid.to_string()),
        name: instance.name,
        x_nm: instance.position.x,
        y_nm: instance.position.y,
    })
}

pub(crate) fn unbind_native_project_sheet_instance_port(
    root: &Path,
    instance_uuid: Uuid,
    port_uuid: Uuid,
) -> Result<NativeProjectSheetInstanceMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let previous_instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    if !previous_instance.ports.contains(&port_uuid) {
        bail!("sheet instance {instance_uuid} does not bind port {port_uuid}");
    }
    let mut instance = previous_instance.clone();
    instance.ports.retain(|candidate| candidate != &port_uuid);
    let report = commit_schematic_write(
        root,
        "unbind schematic sheet instance port",
        |model, provenance| {
            build_set_schematic_sheet_instance(
                model,
                provenance,
                project.schematic.uuid,
                instance_uuid,
                serde_json::to_value(&previous_instance)
                    .expect("native sheet instance serialization must succeed"),
                serde_json::to_value(&instance)
                    .expect("native sheet instance serialization must succeed"),
            )
        },
    )?;
    let evidence = schematic_commit_evidence(&report);

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "unbind_sheet_instance_port".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        model_revision: evidence.model_revision,
        created_ids: evidence.created_ids,
        modified_ids: evidence.modified_ids,
        instance_uuid: instance.uuid.to_string(),
        definition_uuid: instance.definition.to_string(),
        parent_sheet_uuid: instance.parent_sheet.map(|uuid| uuid.to_string()),
        port_uuid: Some(port_uuid.to_string()),
        name: instance.name,
        x_nm: instance.position.x,
        y_nm: instance.position.y,
    })
}

fn sheet_child_object_count(sheet: &NativeSheetRoot) -> usize {
    sheet.symbols.len()
        + sheet.wires.len()
        + sheet.junctions.len()
        + sheet.labels.len()
        + sheet.buses.len()
        + sheet.bus_entries.len()
        + sheet.ports.len()
        + sheet.noconnects.len()
        + sheet.texts.len()
        + sheet.drawings.len()
}

struct SchematicCommitEvidence {
    model_revision: String,
    created_ids: Vec<String>,
    modified_ids: Vec<String>,
}

fn schematic_commit_evidence(report: &CommitReport) -> SchematicCommitEvidence {
    SchematicCommitEvidence {
        model_revision: report.transaction.after_model_revision.0.clone(),
        created_ids: report
            .transaction
            .diff
            .created
            .iter()
            .map(ToString::to_string)
            .collect(),
        modified_ids: report
            .transaction
            .diff
            .modified
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}
