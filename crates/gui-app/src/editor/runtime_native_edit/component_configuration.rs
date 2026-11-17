//! Confirmed component tasks compose the existing Library and Proposal intents.
use super::*;
use datum_gui_protocol::library_browser::component_resolution::configuration::ConfigurationPhase;
use eda_engine::api::native_write::library_gui::LibraryDefinitionRequest;
use eda_engine::api::native_write::schematic_delete_proposal::{
    ComponentPropertiesRequest, FieldRowEdit, NativeProposalAction as Proposal,
    NativeProposalResponse,
};

impl Runtime {
    pub(super) fn configuration_owns_intent(&self, action: NativeEditAction) -> bool {
        self.workspace()
            .library
            .configuration
            .as_ref()
            .is_some_and(|c| match (&c.phase, action) {
                (ConfigurationPhase::SavingDefinitions, NativeEditAction::Library)
                | (
                    ConfigurationPhase::PreparingInstance | ConfigurationPhase::ApplyingInstance(_),
                    NativeEditAction::Proposal,
                ) => true,
                _ => false,
            })
    }

    pub(crate) fn apply_component_configuration(&mut self) -> Result<(), String> {
        let configuration = self
            .workspace()
            .library
            .configuration
            .as_ref()
            .ok_or("Open Component Properties or configure a component")?;
        self.verify_component_configuration_context()?;
        if configuration.pending() || self.native_edit.pending() || self.check_work.pending() {
            return Err("Wait for the current component action".into());
        }
        if let Some(target) = &configuration.target {
            target.verify(self.workspace())?;
            let physical_unchanged = configuration.current_component.as_ref().is_some_and(|c| {
                configuration
                    .footprint()
                    .is_ok_and(|f| f.uuid == c.footprint)
                    && configuration.mapping_source.as_ref() == Some(c)
            });
            if physical_unchanged || configuration.footprint_change_refusal.is_some() {
                if configuration.reference == target.original_reference
                    && configuration.value == target.original_value
                {
                    self.session.workspace_mut().library.configuration = None;
                    return Ok(());
                }
                let proposal = Proposal::CreateFieldBatch {
                    proposal_id: eda_engine::substrate::ObjectId::new_v4(),
                    sheet: target.basis.source,
                    instance_path: target.symbol.instance_path.clone(),
                    edits: vec![FieldRowEdit {
                        symbol: target.symbol.id,
                        reference: configuration.reference.clone(),
                        value: configuration.value.clone(),
                    }],
                };
                return self.submit_component_proposal(proposal);
            }
            if let Some(reason) = &configuration.footprint_change_refusal {
                return Err(reason.clone());
            }
        }
        if let Some(component) = configuration.placement_component() {
            component
                .verify(
                    self.workspace()
                        .library
                        .basis
                        .as_ref()
                        .ok_or("Current library unavailable")?,
                )
                .map_err(|e| e.to_string())?;
            return self.finish_configured_definition();
        }
        let request = configuration.request()?;
        let model = self
            .workspace()
            .library
            .basis
            .as_ref()
            .ok_or("Current library unavailable")?;
        if let Some(component) = configuration.current_component.as_ref().filter(|c| {
            c.value == configuration.value.trim()
                && configuration.mapping_source.as_ref() == Some(*c)
                && c.footprint == request.footprint
        }) {
            component.verify(model).map_err(|e| e.to_string())?;
            return self.finish_configured_definition();
        }
        let reusable = eda_engine::api::native_write::library_gui::component_configuration::reusable_configurations(model, &request);
        if let Some(component) = reusable.into_iter().next() {
            self.session
                .workspace_mut()
                .library
                .configuration
                .as_mut()
                .unwrap()
                .current_component = Some(component);
            return self.finish_configured_definition();
        }
        self.session
            .workspace_mut()
            .library
            .configuration
            .as_mut()
            .unwrap()
            .phase = ConfigurationPhase::SavingDefinitions;
        self.queue_native_edit(NativeEditAction::Library);
        if !self.native_edit.pending() {
            self.fail_component_configuration(
                "Component definition was not submitted; no automatic retry was sent",
            );
            return Err(
                "Component definition was not submitted; no automatic retry was sent".into(),
            );
        }
        Ok(())
    }

    fn submit_component_proposal(&mut self, proposal: Proposal) -> Result<(), String> {
        let configuration = self
            .session
            .workspace_mut()
            .library
            .configuration
            .as_mut()
            .ok_or("Component context unavailable")?;
        configuration.phase = ConfigurationPhase::PreparingInstance;
        configuration.pending_proposal = Some(proposal);
        self.queue_native_edit(NativeEditAction::Proposal);
        if !self.native_edit.pending() {
            self.fail_component_configuration(
                "Component action was not submitted; no automatic retry was sent",
            );
            return Err("Component action was not submitted; no automatic retry was sent".into());
        }
        Ok(())
    }

    fn finish_configured_definition(&mut self) -> Result<(), String> {
        let configuration = self
            .workspace()
            .library
            .configuration
            .as_ref()
            .ok_or("Component context unavailable")?;
        let component = configuration
            .current_component
            .clone()
            .ok_or("Saved component qualification unavailable")?;
        component
            .verify(
                self.workspace()
                    .library
                    .basis
                    .as_ref()
                    .ok_or("Current library unavailable")?,
            )
            .map_err(|e| e.to_string())?;
        if let Some(target) = &configuration.target {
            target.verify(self.workspace())?;
            let proposal = Proposal::CreateComponentProperties(ComponentPropertiesRequest {
                proposal_id: eda_engine::substrate::ObjectId::new_v4(),
                symbol: target.symbol.id,
                instance_path: target.symbol.instance_path.clone(),
                reference: configuration.reference.clone(),
                value: component.value.clone(),
                component,
            });
            self.submit_component_proposal(proposal)
        } else {
            self.resume_configured_component()?;
            Ok(())
        }
    }

    /// Advance only a confirmed response to the explicit Apply/Place action.
    /// A transport refusal never repeats Library creation or Proposal mutation.
    pub(super) fn complete_component_configuration(
        &mut self,
        intent: &NativeEditIntent,
        result: &Result<String, String>,
        response: Option<&NativeProposalResponse>,
    ) {
        let Some(configuration) = self.workspace().library.configuration.clone() else {
            return;
        };
        if !configuration.pending()
            || !matches!(
                intent.action,
                NativeEditAction::Library | NativeEditAction::Proposal
            )
        {
            return;
        }
        let phase = configuration.phase.clone();
        let completed = (|| -> Result<(), String> {
            result.as_ref().map_err(Clone::clone)?;
            if phase == ConfigurationPhase::SavingDefinitions {
                let model = self
                    .workspace()
                    .library
                    .basis
                    .as_ref()
                    .ok_or("Definitions may be saved; current library unavailable")?;
                let request = configuration.request()?;
                let component = eda_engine::api::native_write::library_gui::component_configuration::reusable_configurations(model,&request).into_iter().find(|c|c.part.part==configuration.part_id).ok_or("Definitions may be saved; exact configured component unavailable. Refresh before continuing")?;
                self.session
                    .workspace_mut()
                    .library
                    .configuration
                    .as_mut()
                    .unwrap()
                    .current_component = Some(component);
                return self.finish_configured_definition();
            }
            let response = response.ok_or(
                "Component Proposal response unavailable; inspect history before continuing",
            )?;
            let pending = configuration
                .pending_proposal
                .as_ref()
                .ok_or("Component Proposal identity unavailable")?;
            if response.proposal_id != pending.proposal_id() {
                return Err("Component Proposal identity changed".into());
            }
            if phase == ConfigurationPhase::PreparingInstance {
                if response.status != eda_engine::substrate::ProposalStatus::Draft {
                    return Err("Component Proposal is not a confirmed draft".into());
                }
                configuration
                    .target
                    .as_ref()
                    .ok_or("Component target unavailable")?
                    .verify(self.workspace())?;
                let configuration = self
                    .session
                    .workspace_mut()
                    .library
                    .configuration
                    .as_mut()
                    .unwrap();
                configuration.phase = ConfigurationPhase::ApplyingInstance(response.proposal_id);
                configuration.pending_proposal = Some(Proposal::AcceptApply {
                    proposal_id: response.proposal_id,
                });
                self.queue_native_edit(NativeEditAction::Proposal);
                if !self.native_edit.pending() {
                    return Err("Component draft retained; Apply was not submitted".into());
                }
            } else if phase == ConfigurationPhase::ApplyingInstance(response.proposal_id) {
                if response.status != eda_engine::substrate::ProposalStatus::Applied {
                    return Err("Component application not confirmed; inspect its Proposal before continuing".into());
                }
                let return_to_transfer = self
                    .workspace()
                    .library
                    .configuration
                    .as_ref()
                    .unwrap()
                    .return_to_transfer;
                self.session.workspace_mut().library.configuration = None;
                if return_to_transfer {
                    self.begin_board_transfer();
                }
            }
            Ok(())
        })();
        if let Err(reason) = completed {
            self.fail_component_configuration(&reason);
        }
    }

    pub(super) fn fail_component_configuration(&mut self, reason: &str) {
        if let Some(configuration) = &mut self.session.workspace_mut().library.configuration {
            configuration.phase = ConfigurationPhase::Editing;
            configuration.pending_proposal = None;
            configuration.refusal = Some(format!(
                "{reason}. Saved library definitions remain independent; the instance action is not retried."
            ));
        }
    }

    pub(super) fn component_edit_worker_stopped(&mut self) {
        let reason = "Edit worker stopped; reopen the Project before editing";
        self.fail_component_configuration(reason);
        if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
            a.refusal = Some(reason.into());
        }
        self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
    }
    pub(super) fn prepare_component_library_intent(
        &mut self,
        intent: &mut NativeEditIntent,
    ) -> Result<(), String> {
        if self.workspace().library.revision_workflow.is_some() {
            return self.prepare_library_revision_intent(intent);
        }
        if let Some(configuration) = self
            .workspace()
            .library
            .configuration
            .as_ref()
            .filter(|c| c.phase == ConfigurationPhase::SavingDefinitions)
        {
            self.verify_component_configuration_context()?;
            if let Some(target) = &configuration.target {
                target.verify(self.workspace())?;
            }
            let request = configuration.request()?;
            intent.library = Some(
                datum_gui_protocol::native_edit_client::NativeLibraryEdit::Definition {
                    create: true,
                    request: LibraryDefinitionRequest {
                        kind: "parts".into(),
                        object_id: configuration.part_id,
                        pool_path: "pool".into(),
                        fields: serde_json::Map::from_iter([(
                            "component_configuration".into(),
                            serde_json::to_value(request).map_err(|e| e.to_string())?,
                        )]),
                    },
                },
            );
            return Ok(());
        }
        let draft = self
            .workspace()
            .library
            .draft
            .as_ref()
            .ok_or("Open a Library definition to commit")?;
        intent.library = Some(
            datum_gui_protocol::native_edit_client::NativeLibraryEdit::Definition {
                create: draft.create,
                request: draft.request()?,
            },
        );
        intent.expected_revision = draft.revision.clone();
        intent.expected_tip = draft.tip;
        Ok(())
    }

    pub(super) fn prepare_component_proposal_intent(
        &self,
        intent: &mut NativeEditIntent,
    ) -> Result<(), String> {
        if let Some(configuration) = self
            .workspace()
            .library
            .configuration
            .as_ref()
            .filter(|c| c.pending_proposal.is_some())
        {
            configuration
                .target
                .as_ref()
                .ok_or("Component Properties target unavailable")?
                .verify(self.workspace())?;
            intent.proposal = configuration.pending_proposal.clone();
            return Ok(());
        }
        let review = self
            .workspace()
            .schematic_authoring
            .delete_review
            .as_ref()
            .ok_or("Open a Proposal review")?;
        intent.proposal = review.pending.clone();
        intent.expected_revision = review.revision.clone();
        intent.expected_tip = review.tip;
        Ok(())
    }
}
