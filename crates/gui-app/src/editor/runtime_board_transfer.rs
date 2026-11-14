//! Root-sheet transfer review; the engine owns eligibility and creation.
use super::*;
use datum_gui_protocol::board_transfer::{BoardTransferAction as Action, BoardTransferReview};
use eda_engine::api::native_write::schematic_board_transfer::{
    self as transfer, TransferChoice, TransferRequest,
};
impl Runtime {
    pub(crate) fn begin_board_transfer(&mut self) -> bool {
        let result = (|| -> anyhow::Result<BoardTransferReview> {
            self.selected_schematic_idle().map_err(anyhow::Error::msg)?;
            let basis = datum_gui_protocol::AuthoringViewBasis::capture_unique_root_schematic(
                self.workspace(),
                self.workspace().ui.layout.focused,
            )
            .map_err(anyhow::Error::msg)?;
            anyhow::ensure!(
                self.ensure_selection_authority()?,
                "Open a native Project first"
            );
            let model = &self
                .native_selection
                .as_ref()
                .context("Native source unavailable")?
                .model;
            let request = TransferRequest {
                sheet: basis.source,
                choices: vec![],
                edit_seed: eda_engine::substrate::ObjectId::new_v4(),
                origin: eda_engine::ir::geometry::Point::zero(),
                pitch_nm: 5_000_000,
                layer: 1,
            };
            let plan = transfer::plan_transfer(model, &request)?;
            Ok(BoardTransferReview {
                request,
                plan,
                first_row: 0,
                field_changes: transfer::field_revision::preview(model, basis.source, &[])
                    .map(|revision| revision.changes.len())
                    .map_err(|error| error.to_string()),
                connectivity_changes: transfer::connectivity_revision::preview(model, basis.source)
                    .map(|revision| {
                        revision
                            .changes
                            .iter()
                            .map(|change| change.component_instance)
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    })
                    .map_err(|error| error.to_string()),
                refusal: None,
            })
        })();
        match result {
            Ok(review) => self.session.workspace_mut().ui.board_transfer = Some(review),
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        self.invalidate_frame();
        true
    }
    pub(crate) fn handle_board_transfer(&mut self, action: Action) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let Some(mut review) = self.workspace().ui.board_transfer.clone() else {
            return false;
        };
        match action {
            Action::Surface => {}
            Action::Cancel => {
                self.session.workspace_mut().ui.board_transfer = None;
            }
            Action::Previous | Action::Next => {
                review.first_row = if action == Action::Previous {
                    review.first_row.saturating_sub(1)
                } else {
                    (review.first_row + 1).min(review.plan.components.len().saturating_sub(1))
                };
                self.session.workspace_mut().ui.board_transfer = Some(review);
            }
            Action::ChooseMap(index) => {
                let result = (|| -> anyhow::Result<()> {
                    let row = review
                        .plan
                        .components
                        .get(index)
                        .context("Component row unavailable")?;
                    anyhow::ensure!(
                        !row.component_choice,
                        "The component already declares its physical choice; review a component update to change it"
                    );
                    anyhow::ensure!(
                        self.ensure_selection_authority()?,
                        "Native source unavailable"
                    );
                    let current = &self.native_selection.as_ref().unwrap().model;
                    anyhow::ensure!(
                        current.project.project_id == review.plan.project_id
                            && current.model_revision == review.plan.expected_model_revision
                            && eda_engine::revision::transaction_tip(&current.journal)
                                == review.plan.expected_tip,
                        "Design changed; reopen the component transfer review before choosing a map"
                    );
                    let ci = row
                        .component_instance
                        .context("Component identity unresolved")?;
                    anyhow::ensure!(
                        row.existing_packages.is_empty(),
                        "Existing counterparts are unchanged"
                    );
                    let options = &row.map_options;
                    anyhow::ensure!(
                        !options.is_empty(),
                        "No valid explicit Part/Footprint/Map choice"
                    );
                    let current = review
                        .request
                        .choices
                        .iter()
                        .find(|c| c.component_instance == ci);
                    let next = current
                        .and_then(|c| options.iter().position(|o| o.pin_pad_map == c.pin_pad_map))
                        .map_or(0, |i| (i + 1) % options.len());
                    let map = options[next].pin_pad_map;
                    review
                        .request
                        .choices
                        .retain(|c| c.component_instance != ci);
                    review.request.choices.push(TransferChoice {
                        component_instance: ci,
                        pin_pad_map: map,
                    });
                    anyhow::ensure!(
                        self.ensure_selection_authority()?,
                        "Native source unavailable"
                    );
                    review.plan = transfer::plan_transfer(
                        &self.native_selection.as_ref().unwrap().model,
                        &review.request,
                    )?;
                    Ok(())
                })();
                review.refusal = result.err().map(|e| e.to_string());
                self.session.workspace_mut().ui.board_transfer = Some(review);
            }
            Action::PrepareSources => {
                self.queue_native_edit(NativeEditAction::PrepareBoardSources {
                    board: review.plan.board_id,
                });
            }
            Action::ReviewFields | Action::ReviewConnectivity => {
                let result = (|| -> anyhow::Result<_> {
                    anyhow::ensure!(
                        self.ensure_selection_authority()?,
                        "Native source unavailable"
                    );
                    let model = &self.native_selection.as_ref().unwrap().model;
                    anyhow::ensure!(
                        model.project.project_id == review.plan.project_id
                            && model.model_revision == review.plan.expected_model_revision
                            && eda_engine::revision::transaction_tip(&model.journal)
                                == review.plan.expected_tip,
                        "Design changed; reopen the PCB update review"
                    );
                    if action == Action::ReviewConnectivity {
                        datum_gui_protocol::schematic_authoring::delete_review::DeleteReview::capture_board_connectivity_revision(self.workspace())
                    } else {
                        datum_gui_protocol::schematic_authoring::delete_review::DeleteReview::capture_board_field_revision(self.workspace())
                    }.map_err(anyhow::Error::msg)
                })();
                match result {
                    Ok(fields) => {
                        self.session.workspace_mut().ui.board_transfer = None;
                        self.session
                            .workspace_mut()
                            .schematic_authoring
                            .delete_review = Some(fields);
                        self.queue_native_edit(NativeEditAction::Proposal);
                    }
                    Err(error) => {
                        review.refusal = Some(error.to_string());
                        self.session.workspace_mut().ui.board_transfer = Some(review);
                    }
                }
            }
            Action::Accept => {
                if review.plan.requires_profile_adoption || review.plan.ready_count() == 0 {
                    review.refusal = Some(
                        "Choose valid maps and explicitly prepare footprint sources first".into(),
                    );
                    self.session.workspace_mut().ui.board_transfer = Some(review);
                } else {
                    self.queue_native_edit(NativeEditAction::BoardTransfer);
                }
            }
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn restore_board_transfer(
        &mut self,
        review: Option<BoardTransferReview>,
        result: &Result<String, String>,
    ) {
        let Some(mut review) = review else {
            return;
        };
        review.refusal = result.as_ref().err().cloned();
        if let Some(native) = &self.native_selection {
            review.field_changes =
                transfer::field_revision::preview(&native.model, review.request.sheet, &[])
                    .map(|revision| revision.changes.len())
                    .map_err(|error| error.to_string());
            review.connectivity_changes =
                transfer::connectivity_revision::preview(&native.model, review.request.sheet)
                    .map(|revision| {
                        revision
                            .changes
                            .iter()
                            .map(|change| change.component_instance)
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                    })
                    .map_err(|error| error.to_string());
            match transfer::plan_transfer(&native.model, &review.request) {
                Ok(plan) => review.plan = plan,
                Err(error) => review.refusal = Some(error.to_string()),
            }
        }
        self.session.workspace_mut().ui.board_transfer = Some(review);
    }
}
