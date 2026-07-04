use anyhow::{Result, anyhow};

pub(super) fn parse_artifact_generate_include(include: &str) -> Result<Vec<String>> {
    let mut scopes = Vec::new();
    for raw_scope in include.split(',') {
        let scope = raw_scope.trim();
        if scope.is_empty() {
            continue;
        }
        let scope = match scope {
            "all" => "manufacturing-set",
            "gerber-set" | "manufacturing-set" | "bom" | "pnp" | "drill" => scope,
            _ => {
                return Err(anyhow!(
                    "unsupported artifact include scope: {scope}; supported scopes: gerber-set, manufacturing-set, bom, pnp, drill, all"
                ));
            }
        };
        if !scopes.iter().any(|existing| existing == scope) {
            scopes.push(scope.to_string());
        }
    }
    if scopes.is_empty() {
        return Err(anyhow!(
            "artifact generate requires at least one include scope"
        ));
    }
    Ok(scopes)
}
