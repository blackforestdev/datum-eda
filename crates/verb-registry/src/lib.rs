//! Single-source declarative registry for the Datum user-facing verb surface.
//!
//! Each user-facing verb (MCP tool name == GUI terminal command id) is declared
//! exactly once as a [`VerbSpec`]. Projections (the checked-in
//! `mcp-server/datum_tool_catalog.json` consumed by the MCP Python catalog, and
//! eventually the CLI clap surface, daemon dispatch, and GUI terminal catalog)
//! are generated from this table instead of being mirrored by hand.
//!
//! This crate is a leaf: it depends only on `serde`/`serde_json` and nothing
//! from the workspace, so every surface can consume it without cycles.

mod catalog;
mod verbs_artifact;

pub use catalog::{CATALOG_VERSION, catalog_json, catalog_string};

/// Visibility/lifecycle status of a verb on the public surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbStatus {
    /// Advertised on the public tool surface.
    Public,
    /// Dispatchable for compatibility but hidden from listings.
    Hidden,
    /// No longer dispatchable; kept for tombstone/replacement metadata.
    Retired,
}

impl VerbStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            VerbStatus::Public => "public",
            VerbStatus::Hidden => "hidden",
            VerbStatus::Retired => "retired",
        }
    }
}

/// Retirement metadata for hidden/retired verbs.
#[derive(Debug, Clone, Copy)]
pub struct RetirementNote {
    /// e.g. `retained_until_migration_plan`, `deprecated`, `scheduled_for_removal`.
    pub status: &'static str,
    /// Human-readable retirement criteria.
    pub criteria: &'static str,
}

/// Write-surface classification (decision 004: no private mutation paths).
#[derive(Debug, Clone, Copy)]
pub struct WriteSurface {
    /// e.g. `proposal_metadata_write`, `journaled_route_apply`.
    pub class: &'static str,
    /// Evidence string describing how the write stays on the journaled path.
    pub evidence: &'static str,
}

/// One token of a CLI argv template.
#[derive(Debug, Clone, Copy)]
pub enum ArgvToken {
    /// Literal token emitted verbatim (subcommand names).
    Lit(&'static str),
    /// Positional value of the named parameter.
    Param(&'static str),
    /// `--flag <value>`; omitted at execution time when the named optional
    /// parameter is absent.
    Flag {
        flag: &'static str,
        param: &'static str,
    },
    /// Boolean `--flag`, emitted only when the named parameter is true.
    Switch {
        flag: &'static str,
        param: &'static str,
    },
    /// `--flag <v>` repeated once per element of the named list parameter.
    Repeated {
        flag: &'static str,
        param: &'static str,
    },
}

impl ArgvToken {
    /// Name of the parameter this token consumes, if any.
    pub fn param_name(self) -> Option<&'static str> {
        match self {
            ArgvToken::Lit(_) => None,
            ArgvToken::Param(param)
            | ArgvToken::Flag { param, .. }
            | ArgvToken::Switch { param, .. }
            | ArgvToken::Repeated { param, .. } => Some(param),
        }
    }
}

/// Wire type of one verb parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    Str,
    Int,
    Bool,
    Uuid,
    StrList,
    Json,
}

/// One verb parameter. The order of `VerbSpec::params` IS the positional
/// dispatch-args order used by the MCP runtime (`x_dispatch_args`).
#[derive(Debug, Clone, Copy)]
pub struct ParamSpec {
    pub name: &'static str,
    pub ty: ParamType,
    pub required: bool,
    pub doc: &'static str,
    /// JSON-encoded default injected when the caller omits the parameter.
    pub default_json: Option<&'static str>,
}

/// How a verb executes.
#[derive(Debug, Clone, Copy)]
pub enum Dispatch {
    /// Executes through the `datum-eda` CLI. `method` is the bridge/legacy
    /// flat method name the MCP runtime dispatches through
    /// (`x_dispatch_method`); `argv` is the CLI argv template the bridge
    /// builds for it.
    Cli {
        method: &'static str,
        argv: &'static [ArgvToken],
    },
    /// Executes as a daemon JSON-RPC method.
    DaemonRpc { method: &'static str },
}

impl Dispatch {
    pub fn method(self) -> &'static str {
        match self {
            Dispatch::Cli { method, .. } | Dispatch::DaemonRpc { method } => method,
        }
    }
}

/// One user-facing verb, declared once.
#[derive(Debug, Clone, Copy)]
pub struct VerbSpec {
    /// Canonical id: MCP tool name == GUI terminal command id.
    pub id: &'static str,
    /// One-sentence description (MCP tool description).
    pub summary: &'static str,
    pub status: VerbStatus,
    /// Canonical replacement verb ids (required for hidden/retired verbs).
    pub replacements: &'static [&'static str],
    pub retirement: Option<RetirementNote>,
    pub dispatch: Dispatch,
    /// Positional order here IS the dispatch-args order.
    pub params: &'static [ParamSpec],
    /// Raw JSON overriding the generated `inputSchema` when the schema cannot
    /// be expressed through `params` alone.
    pub schema_json_override: Option<&'static str>,
    pub write_surface: Option<WriteSurface>,
    /// Advertised in the GUI terminal command catalog.
    pub terminal: bool,
}

impl VerbSpec {
    /// Two-segment family prefix, e.g. `datum.artifact`.
    pub fn prefix(&self) -> &'static str {
        match self.id.match_indices('.').nth(1) {
            Some((index, _)) => &self.id[..index],
            None => self.id,
        }
    }
}

/// The full verb table, assembled from per-family modules, sorted by id.
pub fn verbs() -> &'static [VerbSpec] {
    verbs_artifact::VERBS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verb_ids_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for verb in verbs() {
            assert!(seen.insert(verb.id), "duplicate verb id: {}", verb.id);
        }
    }

    #[test]
    fn assembly_is_sorted_by_id() {
        let ids: Vec<&str> = verbs().iter().map(|verb| verb.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "verbs() must be assembled sorted by id");
    }

    #[test]
    fn hidden_and_retired_verbs_name_replacements() {
        for verb in verbs() {
            if matches!(verb.status, VerbStatus::Hidden | VerbStatus::Retired) {
                assert!(
                    !verb.replacements.is_empty(),
                    "{} is {:?} but names no canonical replacements",
                    verb.id,
                    verb.status
                );
            }
        }
    }

    #[test]
    fn param_names_are_unique_per_verb() {
        for verb in verbs() {
            let mut seen = std::collections::BTreeSet::new();
            for param in verb.params {
                assert!(
                    seen.insert(param.name),
                    "{} declares parameter {} twice",
                    verb.id,
                    param.name
                );
            }
        }
    }

    #[test]
    fn argv_tokens_reference_declared_params() {
        for verb in verbs() {
            if let Dispatch::Cli { argv, .. } = verb.dispatch {
                for token in argv {
                    if let Some(param) = token.param_name() {
                        assert!(
                            verb.params.iter().any(|p| p.name == param),
                            "{} argv references undeclared param {}",
                            verb.id,
                            param
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn defaults_and_overrides_parse_as_json() {
        for verb in verbs() {
            for param in verb.params {
                if let Some(raw) = param.default_json {
                    serde_json::from_str::<serde_json::Value>(raw).unwrap_or_else(|err| {
                        panic!("{} param {} default_json invalid: {err}", verb.id, param.name)
                    });
                }
            }
            if let Some(raw) = verb.schema_json_override {
                serde_json::from_str::<serde_json::Value>(raw).unwrap_or_else(|err| {
                    panic!("{} schema_json_override invalid: {err}", verb.id)
                });
            }
        }
    }

    #[test]
    fn catalog_is_deterministic() {
        assert_eq!(catalog_string(), catalog_string());
        assert!(catalog_string().ends_with('\n'));
    }
}
