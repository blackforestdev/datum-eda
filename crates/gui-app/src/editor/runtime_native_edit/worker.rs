//! Existing native edit worker owns typed writes, Proposal lifecycle and qualified refresh.
use super::*;
impl EditWork {
    pub(super) fn start(
        &mut self,
        intent: NativeEditIntent,
        request: LiveReviewRequest,
        inputs: native_edit_selection::Inputs,
        previous: (
            Option<std::sync::Arc<eda_engine::substrate::DesignModel>>,
            datum_gui_protocol::CheckRunReviewState,
        ),
    ) -> Result<()> {
        anyhow::ensure!(!self.pending(), "An edit is already pending");
        let (sender, receiver) = mpsc::sync_channel(1);
        let client = self.client.clone();
        let wake = self.wake.clone();
        std::thread::Builder::new()
            .name("datum-native-edit".into())
            .spawn(move || {
                let mut observed_proposal=None;
                let prepared = (|| -> Result<Answer> {
                    let mut owner = client.lock().map_err(|_|anyhow::anyhow!("Native editor worker unavailable"))?;
                    if owner.is_none() {*owner=Some(NativeProjectClient::spawn()?);}
                    let client=owner.as_ref().unwrap();
                    let started=std::time::Instant::now();
                    // An old DesignModel still materializes source shards from disk.
                    // Freeze this basis before the operation can replace the root.
                    let previous_dispositions = previous.0.as_deref()
                        .map(datum_gui_protocol::workspace_loading::disposition_basis)
                        .transpose()?;
                    let operation=client.execute(&intent);
                    observed_proposal=operation.as_ref().ok().and_then(|r|r.proposal.clone());
                    trace_startup_timing(format!("native edit canonical operation {}ms",started.elapsed().as_millis()));
                    // Qualify the entire refresh against both revision and tip,
                    // including an undo/redo cycle that restores the same bytes.
                    // A confirmed receipt already carries a freshly resolved, durable
                    // post-write description. It is the pre-refresh boundary;
                    // failures still require a fresh recovery-qualified description.
                    let recovered;
                    let before=match &operation {
                        Ok(receipt) if !receipt.description.is_null()=>&receipt.description,
                        Ok(_) | Err(_)=>{recovered=client.describe(&intent.project_root)?; &recovered}
                    };
                    let refreshed=std::time::Instant::now();
                    // Populate existing byte-qualified read evidence on the worker;
                    // publishing an empty cache would cold-resolve on the next click.
                    let mut cache=runtime_selection::SelectionReadCache::default();
                    let model=cache.resolve_with_engine_reader(&intent.project_root,|| client.read_model(&intent.project_root,before))?;
                    let mut next=datum_gui_protocol::workspace_loading::project_board_editor_workspace(&request,model.clone())?;
                    if let Some(previous_dispositions) = previous_dispositions {
                        use datum_gui_protocol::workspace_loading::{disposition_basis, refresh_disposition_report, retain_current_check_projection};
                        if previous_dispositions != disposition_basis(&model)? {
                            refresh_disposition_report(&mut next, &request)?;
                        } else {
                            retain_current_check_projection(&mut next, &previous.1);
                        }
                    }
                    let selection=native_edit_selection::prepare_from_model(&mut next,inputs,model,cache)?;
                    trace_startup_timing(format!("native edit complete workspace and selection {}ms",refreshed.elapsed().as_millis()));
                    let after=client.describe(&intent.project_root)?;
                    // Capability discovery (truthful menu enablement): carry the engine's
                    // native-write verb catalog from native.describe into the refreshed
                    // workspace so native-edit menu entries enable exactly for reported verbs.
                    if let Some(backing)=next.backing.as_mut(){backing.native_write_verbs=native_write_verb_ids(&after);}
                    trace_startup_timing(format!("native edit qualified answer {}ms",started.elapsed().as_millis()));
                    let qualified=before.get("model_revision")==after.get("model_revision")
                        && before.get("transaction_tip")==after.get("transaction_tip")
                        && after.get("project_id")==Some(&serde_json::json!(intent.project_id))
                        && next.scene.project_uuid==intent.project_id.to_string()
                        && next.supervision.model_revision==next.scene.source_revision
                        && after.get("model_revision").and_then(serde_json::Value::as_str)==Some(next.scene.source_revision.as_str())
                        && after.get("transaction_tip").and_then(serde_json::Value::as_str)==next.supervision.journal.accepted_transaction_tip.as_deref();
                    let proposal=operation.as_ref().ok().and_then(|receipt|receipt.proposal.clone());
                    let proposal_refusal=operation.as_ref().ok().and_then(|r|r.proposal_refusal.clone());
                    let result=match operation {
                        Ok(receipt) if qualified && receipt.description.get("model_revision")==after.get("model_revision")
                            && receipt.description.get("transaction_tip")==after.get("transaction_tip") =>
                            Ok(if let Some(proposal)=&receipt.proposal {match proposal.status { eda_engine::substrate::ProposalStatus::Draft=>"Proposal ready; Tab accepts, Esc dismisses",eda_engine::substrate::ProposalStatus::Applied=>if proposal.disposition.is_some() {"ERC disposition recorded"} else if proposal.assignment.is_some() {"Part/Gate assignment recorded"} else {"Selected objects deleted"},eda_engine::substrate::ProposalStatus::Rejected=>"Proposal dismissed",_=>"Proposal updated"}} else if receipt.applied {if receipt.reconciled {"Edit committed; recovered receipt confirmed"} else {"Edit committed"}} else {"Project saved"}.into()),
                        Ok(_)=>Err("Edit completed but the Project changed during refresh; reopen before editing".into()),
                        Err(error)=>Err(format!("{error:#}")),
                    };
                    let result=proposal_refusal.map_or(result,Err);
                    Ok(Answer {proposal,intent:intent.clone(),result,next:qualified.then_some(next),selection:qualified.then_some(selection)})
                })();
                let answer=prepared.unwrap_or_else(|error|Answer {proposal:observed_proposal,intent,result:Err(format!("{error:#}; reopen the Project before editing")),next:None,selection:None});
                finish_owned_edit(client, sender, answer, || {
                    let _ = wake.send_event(());
                });
            })?;
        self.pending = Some(receiver);
        Ok(())
    }
}
