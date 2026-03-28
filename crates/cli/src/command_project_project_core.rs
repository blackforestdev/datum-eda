use super::*;

pub(super) fn ensure_project_root(root: &Path) -> Result<()> {
    if root.exists() {
        if !root.is_dir() {
            bail!(
                "project root exists but is not a directory: {}",
                root.display()
            );
        }
    } else {
        std::fs::create_dir_all(root)
            .with_context(|| format!("failed to create project root {}", root.display()))?;
    }
    Ok(())
}

pub(super) fn load_existing_ids(root: &Path) -> Result<Option<ExistingProjectIds>> {
    let project_path = root.join("project.json");
    if !project_path.exists() {
        return Ok(None);
    }

    let project_text = std::fs::read_to_string(&project_path)
        .with_context(|| format!("failed to read {}", project_path.display()))?;
    let manifest: NativeProjectManifest = serde_json::from_str(&project_text)
        .with_context(|| format!("failed to parse {}", project_path.display()))?;

    let schematic_path = root.join(&manifest.schematic);
    let board_path = root.join(&manifest.board);
    let schematic_text = std::fs::read_to_string(&schematic_path)
        .with_context(|| format!("failed to read {}", schematic_path.display()))?;
    let board_text = std::fs::read_to_string(&board_path)
        .with_context(|| format!("failed to read {}", board_path.display()))?;
    let schematic: NativeSchematicRoot = serde_json::from_str(&schematic_text)
        .with_context(|| format!("failed to parse {}", schematic_path.display()))?;
    let board: NativeBoardRoot = serde_json::from_str(&board_text)
        .with_context(|| format!("failed to parse {}", board_path.display()))?;

    Ok(Some(ExistingProjectIds {
        project_uuid: manifest.uuid,
        schematic_uuid: schematic.uuid,
        board_uuid: board.uuid,
    }))
}

pub(super) struct LoadedNativeProject {
    pub(super) root: std::path::PathBuf,
    pub(super) manifest: NativeProjectManifest,
    pub(super) schematic: NativeSchematicRoot,
    pub(super) board: NativeBoardRoot,
    pub(super) rules: NativeRulesRoot,
    pub(super) schematic_path: std::path::PathBuf,
    pub(super) board_path: std::path::PathBuf,
    pub(super) rules_path: std::path::PathBuf,
}

pub(super) struct NativeSchematicCounts {
    pub(super) symbols: usize,
    pub(super) wires: usize,
    pub(super) junctions: usize,
    pub(super) labels: usize,
    pub(super) ports: usize,
    pub(super) buses: usize,
    pub(super) bus_entries: usize,
    pub(super) noconnects: usize,
    pub(super) texts: usize,
    pub(super) drawings: usize,
}

pub(super) fn load_native_project(root: &Path) -> Result<LoadedNativeProject> {
    let root = root.to_path_buf();
    if !root.is_dir() {
        bail!(
            "project root does not exist or is not a directory: {}",
            root.display()
        );
    }

    let manifest_path = root.join("project.json");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: NativeProjectManifest = serde_json::from_str(&manifest_text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let schematic_path = root.join(&manifest.schematic);
    let board_path = root.join(&manifest.board);
    let rules_path = root.join(&manifest.rules);
    let schematic_text = std::fs::read_to_string(&schematic_path)
        .with_context(|| format!("failed to read {}", schematic_path.display()))?;
    let board_text = std::fs::read_to_string(&board_path)
        .with_context(|| format!("failed to read {}", board_path.display()))?;
    let rules_text = std::fs::read_to_string(&rules_path)
        .with_context(|| format!("failed to read {}", rules_path.display()))?;
    let schematic: NativeSchematicRoot = serde_json::from_str(&schematic_text)
        .with_context(|| format!("failed to parse {}", schematic_path.display()))?;
    let board: NativeBoardRoot = serde_json::from_str(&board_text)
        .with_context(|| format!("failed to parse {}", board_path.display()))?;
    let rules: NativeRulesRoot = serde_json::from_str(&rules_text)
        .with_context(|| format!("failed to parse {}", rules_path.display()))?;

    Ok(LoadedNativeProject {
        root,
        manifest,
        schematic,
        board,
        rules,
        schematic_path,
        board_path,
        rules_path,
    })
}

pub(super) fn collect_schematic_counts(
    root: &Path,
    schematic: &NativeSchematicRoot,
) -> Result<NativeSchematicCounts> {
    let mut symbols = 0usize;
    let mut wires = 0usize;
    let mut junctions = 0usize;
    let mut labels = 0usize;
    let mut ports = 0usize;
    let mut buses = 0usize;
    let mut bus_entries = 0usize;
    let mut noconnects = 0usize;
    let mut texts = 0usize;
    let mut drawings = 0usize;

    for sheet_path in schematic.sheets.values() {
        let path = root.join("schematic").join(sheet_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        symbols += json_object_len(&sheet_value, "symbols");
        wires += json_object_len(&sheet_value, "wires");
        junctions += json_object_len(&sheet_value, "junctions");
        labels += json_object_len(&sheet_value, "labels");
        ports += json_object_len(&sheet_value, "ports");
        buses += json_object_len(&sheet_value, "buses");
        bus_entries += json_object_len(&sheet_value, "bus_entries");
        noconnects += json_object_len(&sheet_value, "noconnects");
        texts += json_object_len(&sheet_value, "texts");
        drawings += json_object_len(&sheet_value, "drawings");
    }

    Ok(NativeSchematicCounts {
        symbols,
        wires,
        junctions,
        labels,
        ports,
        buses,
        bus_entries,
        noconnects,
        texts,
        drawings,
    })
}

pub(super) fn build_native_project_schematic(project: &LoadedNativeProject) -> Result<Schematic> {
    let mut sheets = HashMap::new();

    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let expected_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let native_sheet = load_native_sheet(&path)?;
        if native_sheet.uuid != expected_uuid {
            bail!(
                "sheet UUID mismatch: schematic root key {} does not match {} in {}",
                expected_uuid,
                native_sheet.uuid,
                path.display()
            );
        }
        sheets.insert(expected_uuid, native_sheet_into_engine_sheet(native_sheet));
    }

    Ok(Schematic {
        uuid: project.schematic.uuid,
        sheets,
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::<CheckWaiver>::new(),
    })
}

pub(super) fn build_native_project_board(project: &LoadedNativeProject) -> Result<Board> {
    let stackup_layers = project
        .board
        .stackup
        .layers
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()?;
    let packages = project
        .board
        .packages
        .values()
        .cloned()
        .map(|value| {
            let package: PlacedPackage =
                serde_json::from_value(value).context("failed to parse board component")?;
            Ok((package.uuid, package))
        })
        .collect::<Result<HashMap<Uuid, PlacedPackage>>>()?;
    let pads = project
        .board
        .pads
        .values()
        .cloned()
        .map(|value| {
            let pad: PlacedPad =
                serde_json::from_value(value).context("failed to parse board pad")?;
            Ok((pad.uuid, pad))
        })
        .collect::<Result<HashMap<Uuid, PlacedPad>>>()?;
    let tracks = project
        .board
        .tracks
        .values()
        .cloned()
        .map(|value| {
            let track: Track =
                serde_json::from_value(value).context("failed to parse board track")?;
            Ok((track.uuid, track))
        })
        .collect::<Result<HashMap<Uuid, Track>>>()?;
    let vias = project
        .board
        .vias
        .values()
        .cloned()
        .map(|value| {
            let via: Via = serde_json::from_value(value).context("failed to parse board via")?;
            Ok((via.uuid, via))
        })
        .collect::<Result<HashMap<Uuid, Via>>>()?;
    let zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| {
            let zone: Zone = serde_json::from_value(value).context("failed to parse board zone")?;
            Ok((zone.uuid, zone))
        })
        .collect::<Result<HashMap<Uuid, Zone>>>()?;
    let nets = project
        .board
        .nets
        .values()
        .cloned()
        .map(|value| {
            let net: Net = serde_json::from_value(value).context("failed to parse board net")?;
            Ok((net.uuid, net))
        })
        .collect::<Result<HashMap<Uuid, Net>>>()?;
    let net_classes = project
        .board
        .net_classes
        .values()
        .cloned()
        .map(|value| {
            let net_class: NetClass =
                serde_json::from_value(value).context("failed to parse board net class")?;
            Ok((net_class.uuid, net_class))
        })
        .collect::<Result<HashMap<Uuid, NetClass>>>()?;
    let keepouts = project
        .board
        .keepouts
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board keepout"))
        .collect::<Result<Vec<Keepout>>>()?;
    let dimensions = project
        .board
        .dimensions
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board dimension"))
        .collect::<Result<Vec<Dimension>>>()?;
    let texts = project
        .board
        .texts
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board text"))
        .collect::<Result<Vec<BoardText>>>()?;
    let rules = project
        .rules
        .rules
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board rule"))
        .collect::<Result<Vec<Rule>>>()?;

    Ok(Board {
        uuid: project.board.uuid,
        name: project.board.name.clone(),
        stackup: Stackup {
            layers: stackup_layers,
        },
        outline: Polygon {
            vertices: project
                .board
                .outline
                .vertices
                .iter()
                .map(|point| Point {
                    x: point.x,
                    y: point.y,
                })
                .collect(),
            closed: project.board.outline.closed,
        },
        packages,
        pads,
        tracks,
        vias,
        zones,
        nets,
        net_classes,
        rules,
        keepouts,
        dimensions,
        texts,
    })
}

pub(super) fn summarize_native_schematic_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
    erc_findings: &[ErcFinding],
) -> CheckSummary {
    let mut by_code: BTreeMap<String, usize> = BTreeMap::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    let mut waived = 0usize;

    for diagnostic in diagnostics {
        *by_code.entry(diagnostic.kind.clone()).or_default() += 1;
        match diagnostic.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            _ => infos += 1,
        }
    }

    for finding in erc_findings {
        *by_code.entry(finding.code.to_string()).or_default() += 1;
        if finding.waived {
            waived += 1;
            continue;
        }
        match finding.severity {
            eda_engine::erc::ErcSeverity::Error => errors += 1,
            eda_engine::erc::ErcSeverity::Warning => warnings += 1,
            eda_engine::erc::ErcSeverity::Info => infos += 1,
        }
    }

    let status = if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };

    let mut by_code = by_code
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect::<Vec<_>>();
    by_code.sort_by(|a, b| a.code.cmp(&b.code));

    CheckSummary {
        status,
        errors,
        warnings,
        infos,
        waived,
        by_code,
    }
}

pub(super) fn summarize_native_board_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
) -> CheckSummary {
    let mut by_code: BTreeMap<String, usize> = BTreeMap::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;

    for diagnostic in diagnostics {
        *by_code.entry(diagnostic.kind.clone()).or_default() += 1;
        match diagnostic.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            _ => infos += 1,
        }
    }

    let status = if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };

    let mut by_code = by_code
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect::<Vec<_>>();
    by_code.sort_by(|a, b| a.code.cmp(&b.code));

    CheckSummary {
        status,
        errors,
        warnings,
        infos,
        waived: 0,
        by_code,
    }
}
