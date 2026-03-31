use super::*;

impl Engine {
    pub fn get_board_summary(&self) -> Result<BoardSummary, EngineError> {
        Ok(self.require_board()?.summary())
    }

    pub fn get_components(&self) -> Result<Vec<ComponentInfo>, EngineError> {
        Ok(self.require_board()?.components())
    }

    pub fn get_net_info(&self) -> Result<Vec<BoardNetInfo>, EngineError> {
        Ok(self.require_board()?.net_info())
    }

    pub fn get_stackup(&self) -> Result<StackupInfo, EngineError> {
        Ok(self.require_board()?.stackup_info())
    }

    pub fn get_unrouted(&self) -> Result<Vec<Airwire>, EngineError> {
        Ok(self.require_board()?.unrouted())
    }

    pub fn get_schematic_summary(&self) -> Result<SchematicSummary, EngineError> {
        Ok(self.require_schematic()?.summary())
    }

    pub fn get_sheets(&self) -> Result<Vec<SheetSummary>, EngineError> {
        Ok(self.require_schematic()?.sheet_summaries())
    }

    pub fn get_labels(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<LabelInfo>, EngineError> {
        Ok(self.require_schematic()?.labels(sheet))
    }

    pub fn get_symbols(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<SymbolInfo>, EngineError> {
        Ok(self.require_schematic()?.symbols(sheet))
    }

    pub fn get_symbol_fields(
        &self,
        symbol_uuid: &uuid::Uuid,
    ) -> Result<Vec<SymbolFieldInfo>, EngineError> {
        let schematic = self.require_schematic()?;
        for sheet in schematic.sheets.values() {
            if let Some(symbol) = sheet.symbols.get(symbol_uuid) {
                let mut fields: Vec<_> = symbol
                    .fields
                    .iter()
                    .map(|field| SymbolFieldInfo {
                        uuid: field.uuid,
                        symbol: symbol.uuid,
                        key: field.key.clone(),
                        value: field.value.clone(),
                        visible: field.visible,
                        position: field.position,
                    })
                    .collect();
                fields.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.uuid.cmp(&b.uuid)));
                return Ok(fields);
            }
        }
        Err(EngineError::NotFound {
            object_type: "symbol",
            uuid: *symbol_uuid,
        })
    }

    pub fn get_ports(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<PortInfo>, EngineError> {
        Ok(self.require_schematic()?.ports(sheet))
    }

    pub fn get_buses(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<BusInfo>, EngineError> {
        Ok(self.require_schematic()?.buses(sheet))
    }

    pub fn get_bus_entries(
        &self,
        sheet: Option<&uuid::Uuid>,
    ) -> Result<Vec<BusEntryInfo>, EngineError> {
        Ok(self.require_schematic()?.bus_entries(sheet))
    }

    pub fn get_noconnects(
        &self,
        sheet: Option<&uuid::Uuid>,
    ) -> Result<Vec<NoConnectInfo>, EngineError> {
        Ok(self.require_schematic()?.noconnects(sheet))
    }

    pub fn get_hierarchy(&self) -> Result<HierarchyInfo, EngineError> {
        Ok(self.require_schematic()?.hierarchy())
    }

    pub fn get_schematic_net_info(&self) -> Result<Vec<SchematicNetInfo>, EngineError> {
        Ok(connectivity::schematic_net_info(self.require_schematic()?))
    }

    pub fn get_netlist(&self) -> Result<Vec<NetlistNet>, EngineError> {
        let design = self.require_design()?;
        if let Some(board) = design.board.as_ref() {
            return Ok(board
                .net_info()
                .into_iter()
                .map(|net| NetlistNet {
                    uuid: net.uuid,
                    name: net.name,
                    class: Some(net.class),
                    pins: net
                        .pins
                        .into_iter()
                        .map(|pin| NetlistPin {
                            component: pin.component,
                            pin: pin.pin,
                        })
                        .collect(),
                    routed_pct: Some(net.routed_pct),
                    labels: None,
                    ports: None,
                    sheets: None,
                    semantic_class: None,
                })
                .collect());
        }

        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(connectivity::schematic_net_info(schematic)
            .into_iter()
            .map(|net| NetlistNet {
                uuid: net.uuid,
                name: net.name,
                class: net.class,
                pins: net
                    .pins
                    .into_iter()
                    .map(|pin| NetlistPin {
                        component: pin.component,
                        pin: pin.pin,
                    })
                    .collect(),
                routed_pct: None,
                labels: Some(net.labels),
                ports: Some(net.ports),
                sheets: Some(net.sheets),
                semantic_class: net.semantic_class,
            })
            .collect())
    }

    pub fn get_connectivity_diagnostics(
        &self,
    ) -> Result<Vec<ConnectivityDiagnosticInfo>, EngineError> {
        let design = self.require_design()?;
        if let Some(board) = design.board.as_ref() {
            return Ok(board.diagnostics());
        }
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(connectivity::schematic_diagnostics(schematic))
    }

    pub fn get_check_report(&self) -> Result<CheckReport, EngineError> {
        let design = self.require_design()?;
        if let Some(board) = design.board.as_ref() {
            let diagnostics = board.diagnostics();
            return Ok(CheckReport::Board {
                summary: summarize_diagnostics(&diagnostics),
                diagnostics,
            });
        }
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        let diagnostics = connectivity::schematic_diagnostics(schematic);
        let erc = erc::run_prechecks(schematic);
        Ok(CheckReport::Schematic {
            summary: summarize_schematic_checks(&diagnostics, &erc),
            diagnostics,
            erc,
            drc: Vec::new(),
        })
    }

    pub fn run_erc_prechecks(&self) -> Result<Vec<ErcFinding>, EngineError> {
        Ok(erc::run_prechecks(self.require_schematic()?))
    }

    pub fn run_erc_prechecks_with_config(
        &self,
        config: &ErcConfig,
    ) -> Result<Vec<ErcFinding>, EngineError> {
        Ok(erc::run_prechecks_with_config(
            self.require_schematic()?,
            config,
        ))
    }

    pub fn run_erc_prechecks_with_config_and_waivers(
        &self,
        config: &ErcConfig,
        waivers: &[CheckWaiver],
    ) -> Result<Vec<ErcFinding>, EngineError> {
        let schematic = self.require_schematic()?;
        let mut effective_waivers = schematic.waivers.clone();
        effective_waivers.extend_from_slice(waivers);

        Ok(erc::run_prechecks_with_config_and_waivers(
            schematic,
            config,
            &effective_waivers,
        ))
    }

    pub fn run_drc(&self, rule_types: &[RuleType]) -> Result<DrcReport, EngineError> {
        let waivers = self
            .design
            .as_ref()
            .and_then(|design| design.schematic.as_ref())
            .map(|schematic| schematic.waivers.as_slice())
            .unwrap_or(&[]);
        Ok(drc::run_with_waivers(
            self.require_board()?,
            rule_types,
            waivers,
        ))
    }

    pub fn get_design_rules(&self) -> Result<Vec<Rule>, EngineError> {
        Ok(self.require_board()?.rules.clone())
    }

    fn require_design(&self) -> Result<&Design, EngineError> {
        self.design.as_ref().ok_or(EngineError::NoProjectOpen)
    }

    fn require_board(&self) -> Result<&Board, EngineError> {
        self.require_design()?
            .board
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "board",
                uuid: uuid::Uuid::nil(),
            })
    }

    fn require_schematic(&self) -> Result<&Schematic, EngineError> {
        self.require_design()?
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })
    }
}
