//! Bounded diagnostic projection, never a complete inspection or proof payload.
use datum_gui_protocol::{AuthoredSelectionIdentity as Id, SelectionSubject, SelectionTarget};
use serde::Serialize;
use serde_json::{Value, json};

const SAMPLE: usize = 8;
const TEXT: usize = 160;

fn digest(value: &impl Serialize) -> String {
    // Reuse the existing engine SHA-256 utility; no dependency or new hash law.
    eda_engine::import::ids_sidecar::compute_source_hash_bytes(
        &serde_json::to_vec(value).expect("typed diagnostic identities serialize"),
    )
}

fn text(value: &str) -> Value {
    json!({"preview": value.chars().take(TEXT).collect::<String>(),
        "omitted_chars": value.chars().count().saturating_sub(TEXT)})
}

fn identity(id: &Id) -> Value {
    json!({"class": id.class, "id": id.id,
        "instance_path_sample": id.instance_path.iter().take(SAMPLE).collect::<Vec<_>>(),
        "instance_path_count": id.instance_path.len(),
        "instance_path_sha256": digest(&id.instance_path)})
}

pub(super) fn capture(target: &SelectionTarget) -> Value {
    let SelectionTarget::Native(view) = target else {
        return match target {
            SelectionTarget::None => json!({"kind":"none", "member_count":0}),
            SelectionTarget::ReviewAction(id) => {
                json!({"kind":"review_action", "identity":text(id)})
            }
            SelectionTarget::AuthoredObject(id) => {
                json!({"kind":"authored_object", "identity":text(id)})
            }
            SelectionTarget::CheckFinding(id) => {
                json!({"kind":"check_finding", "identity":text(id)})
            }
            SelectionTarget::Native(_) => unreachable!(),
        };
    };
    let subject_identity = match &view.subject {
        SelectionSubject::Object(id) => identity(id),
        SelectionSubject::Run(run) => json!({"origin":identity(&run.origin), "sha256":digest(run)}),
        SelectionSubject::GlobalNet(id) | SelectionSubject::Bus(id) => json!(id),
        SelectionSubject::Proposal(id)
        | SelectionSubject::Review(id)
        | SelectionSubject::Diagnostic(id) => text(id),
        SelectionSubject::None | SelectionSubject::Compound(_) => Value::Null,
    };
    json!({
        "kind":view.subject.kind(), "project":view.project, "revision":view.revision,
        "subject_identity":subject_identity, "focus":view.focus().map(identity), "scope":view.scope,
        "member_count":view.members.len(), "members_sha256":digest(&view.members),
        "member_sample":view.members.iter().take(SAMPLE).map(identity).collect::<Vec<_>>(),
        "omitted_members":view.members.len().saturating_sub(SAMPLE),
        "related_count":view.related.len(), "related_sha256":digest(&view.related),
        "hidden_count":view.hidden.len(), "hidden_sha256":digest(&view.hidden),
        "locked_count":view.locked.len(), "locked_sha256":digest(&view.locked),
        "unprojected_count":view.unprojected.len(), "unprojected_sha256":digest(&view.unprojected),
        "dropped_count":view.dropped.len(), "dropped_sha256":digest(&view.dropped),
        "notice":view.notice.as_deref().map(text), "stale":view.stale,
        "complete_inspection":false
    })
}
