//! Trusted transport actor validation and stable audit vocabulary.

use std::collections::BTreeMap;

use serde_json::json;

use super::{
    PreferenceActorKindV1, PreferenceActorV1, PreferenceContextV1, PreferenceErrorCodeV1,
    PreferenceErrorV1,
};

pub(super) fn validate_actor(
    actor: &PreferenceActorV1,
    context: &PreferenceContextV1,
) -> Result<(), PreferenceErrorV1> {
    if actor.session_id.trim().is_empty() || actor.local_actor_id.trim().is_empty() {
        return Err(PreferenceErrorV1 {
            code: PreferenceErrorCodeV1::UnauthorizedActor,
            message: "Trusted actor identity is incomplete".to_owned(),
            details: BTreeMap::from([(
                "required_authority".to_owned(),
                json!("trusted transport actor"),
            )]),
            current_context: context.clone(),
            preserved_draft: None,
            preserved_proposal: None,
        });
    }
    Ok(())
}

pub(super) fn actor_kind(actor: &PreferenceActorV1) -> &'static str {
    match actor.kind {
        PreferenceActorKindV1::HumanGui => "human_gui",
        PreferenceActorKindV1::HumanCli => "human_cli",
        PreferenceActorKindV1::McpAgent => "mcp_agent",
        PreferenceActorKindV1::ScriptAgent => "script_agent",
    }
}
