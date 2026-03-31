use super::*;

pub(crate) fn import_design_for_query(path: &Path) -> Result<Engine> {
    import_design_for_query_with_libraries(path, &[])
}

fn import_design_for_query_with_libraries(path: &Path, libraries: &[PathBuf]) -> Result<Engine> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    for library in libraries {
        engine
            .import_eagle_library(library)
            .with_context(|| format!("failed to import library {}", library.display()))?;
    }
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    Ok(engine)
}

pub(crate) fn query_summary(path: &Path) -> Result<SummaryView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(summary) => Ok(SummaryView::Board {
            name: summary.name,
            layers: summary.layer_count,
            components: summary.component_count,
            nets: summary.net_count,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            let summary = engine.get_schematic_summary()?;
            Ok(SummaryView::Schematic {
                sheets: summary.sheet_count,
                symbols: summary.symbol_count,
                labels: summary.net_label_count,
                ports: summary.port_count,
            })
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_netlist(path: &Path) -> Result<NetlistView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(NetlistView::Board {
            netlist: engine.get_netlist()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            require_schematic(&engine, path)?;
            Ok(NetlistView::Schematic {
                netlist: engine.get_netlist()?,
            })
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_nets(path: &Path) -> Result<NetListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_net_info() {
        Ok(nets) => Ok(NetListView::Board { nets }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => Ok(NetListView::Schematic {
            nets: engine.get_schematic_net_info()?,
        }),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_schematic_nets(path: &Path) -> Result<NetListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(NetListView::Schematic {
        nets: engine.get_schematic_net_info()?,
    })
}

pub(crate) fn query_components(path: &Path) -> Result<ComponentListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_components() {
        Ok(components) => Ok(ComponentListView::Board { components }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "component query is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn require_schematic(engine: &Engine, path: &Path) -> Result<()> {
    match engine.get_schematic_summary() {
        Ok(_) => Ok(()),
        Err(EngineError::NotFound {
            object_type: "schematic",
            ..
        }) => bail!(
            "query is currently only implemented for schematics for this subcommand in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_sheets(path: &Path) -> Result<SheetListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(SheetListView::Schematic {
        sheets: engine.get_sheets()?,
    })
}

pub(crate) fn query_symbols(path: &Path) -> Result<SymbolListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(SymbolListView::Schematic {
        symbols: engine.get_symbols(None)?,
    })
}

pub(crate) fn query_labels(path: &Path) -> Result<LabelListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(LabelListView::Schematic {
        labels: engine.get_labels(None)?,
    })
}

pub(crate) fn query_ports(path: &Path) -> Result<PortListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(PortListView::Schematic {
        ports: engine.get_ports(None)?,
    })
}

pub(crate) fn query_buses(path: &Path) -> Result<BusListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(BusListView::Schematic {
        buses: engine.get_buses(None)?,
    })
}

pub(crate) fn query_bus_entries(path: &Path) -> Result<BusEntryListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(BusEntryListView::Schematic {
        bus_entries: engine.get_bus_entries(None)?,
    })
}

pub(crate) fn query_noconnects(path: &Path) -> Result<NoConnectListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(NoConnectListView::Schematic {
        noconnects: engine.get_noconnects(None)?,
    })
}

pub(crate) fn query_hierarchy(path: &Path) -> Result<HierarchyView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(HierarchyView::Schematic {
        hierarchy: engine.get_hierarchy()?,
    })
}

pub(crate) fn query_diagnostics(path: &Path) -> Result<DiagnosticsView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DiagnosticsView::Board {
            diagnostics: engine.get_connectivity_diagnostics()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            require_schematic(&engine, path)?;
            Ok(DiagnosticsView::Schematic {
                diagnostics: engine.get_connectivity_diagnostics()?,
            })
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_unrouted(path: &Path) -> Result<UnroutedView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(UnroutedView::Board {
            airwires: engine.get_unrouted()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query unrouted is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_design_rules(path: &Path) -> Result<DesignRuleListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DesignRuleListView::Board {
            rules: engine.get_design_rules()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query design-rules is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_package_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PackageChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_package_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query package-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_part_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PartChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_part_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query part-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_component_replacement_plan(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<ComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_component_replacement_plan(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query component-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn query_scoped_component_replacement_plan(
    path: &Path,
    input: ScopedComponentReplacementPolicyInput,
    edit: ScopedComponentReplacementPlanEdit,
    libraries: &[PathBuf],
) -> Result<ScopedComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => {
            let plan = engine.get_scoped_component_replacement_plan(input)?;
            if edit.exclude_component_uuids.is_empty() && edit.overrides.is_empty() {
                Ok(plan)
            } else {
                Ok(engine.edit_scoped_component_replacement_plan(plan, edit)?)
            }
        }
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query scoped-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}
