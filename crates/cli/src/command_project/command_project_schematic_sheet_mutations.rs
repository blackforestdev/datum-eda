use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

pub(crate) fn create_native_project_sheet(
    root: &Path,
    name: String,
    sheet_uuid: Option<Uuid>,
) -> Result<NativeProjectSheetMutationReportView> {
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("sheet name must not be empty");
    }
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "create schematic sheet",
        vec![Operation::CreateSchematicSheet {
            schematic_id: project.schematic.uuid,
            sheet_id: sheet_uuid,
            relative_path: relative_path.clone(),
            sheet: serde_json::to_value(&sheet).expect("native sheet serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetMutationReportView {
        action: "create_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "delete schematic sheet",
        vec![Operation::DeleteSchematicSheet {
            schematic_id: project.schematic.uuid,
            sheet_id: sheet_uuid,
            relative_path: relative_path.clone(),
            sheet: sheet_value,
        }],
    )?;

    Ok(NativeProjectSheetMutationReportView {
        action: "delete_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    commit_schematic_operations(
        root,
        "rename schematic sheet",
        vec![Operation::SetSchematicSheetName {
            sheet_id: sheet_uuid,
            name: name.clone(),
        }],
    )?;

    Ok(NativeProjectSheetMutationReportView {
        action: "rename_sheet".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "create schematic sheet definition",
        vec![Operation::CreateSchematicDefinition {
            schematic_id: project.schematic.uuid,
            definition_id: definition_uuid,
            relative_path: relative_path.clone(),
            definition: serde_json::to_value(&definition)
                .expect("native sheet definition serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetDefinitionMutationReportView {
        action: "create_sheet_definition".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "create schematic sheet instance",
        vec![Operation::CreateSchematicSheetInstance {
            schematic_id: project.schematic.uuid,
            instance_id: instance_uuid,
            instance: serde_json::to_value(&instance)
                .expect("native sheet instance serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "create_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
    let instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    commit_schematic_operations(
        root,
        "delete schematic sheet instance",
        vec![Operation::DeleteSchematicSheetInstance {
            schematic_id: project.schematic.uuid,
            instance_id: instance_uuid,
            instance: serde_json::to_value(&instance)
                .expect("native sheet instance serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "delete_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
    let previous_instance = project
        .schematic
        .instances
        .iter()
        .find(|candidate| candidate.uuid == instance_uuid)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sheet instance not found: {instance_uuid}"))?;
    let mut instance = previous_instance.clone();
    instance.position = NativePoint { x: x_nm, y: y_nm };
    commit_schematic_operations(
        root,
        "move schematic sheet instance",
        vec![Operation::SetSchematicSheetInstance {
            schematic_id: project.schematic.uuid,
            instance_id: instance_uuid,
            previous_instance: serde_json::to_value(&previous_instance)
                .expect("native sheet instance serialization must succeed"),
            instance: serde_json::to_value(&instance)
                .expect("native sheet instance serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "move_sheet_instance".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "bind schematic sheet instance port",
        vec![Operation::SetSchematicSheetInstance {
            schematic_id: project.schematic.uuid,
            instance_id: instance_uuid,
            previous_instance: serde_json::to_value(&previous_instance)
                .expect("native sheet instance serialization must succeed"),
            instance: serde_json::to_value(&instance)
                .expect("native sheet instance serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "bind_sheet_instance_port".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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
    let project = load_native_project(root)?;
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
    commit_schematic_operations(
        root,
        "unbind schematic sheet instance port",
        vec![Operation::SetSchematicSheetInstance {
            schematic_id: project.schematic.uuid,
            instance_id: instance_uuid,
            previous_instance: serde_json::to_value(&previous_instance)
                .expect("native sheet instance serialization must succeed"),
            instance: serde_json::to_value(&instance)
                .expect("native sheet instance serialization must succeed"),
        }],
    )?;

    Ok(NativeProjectSheetInstanceMutationReportView {
        action: "unbind_sheet_instance_port".to_string(),
        project_root: project.root.display().to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
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

fn commit_schematic_operations(
    root: &Path,
    reason: &str,
    operations: Vec<Operation>,
) -> Result<()> {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: reason.to_string(),
                },
                operations,
            },
        )
        .map(|_| ())
        .map_err(Into::into)
}
