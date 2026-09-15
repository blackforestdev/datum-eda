use anyhow::{Result, bail};
use eda_engine::substrate::ArtifactKind;
use uuid::Uuid;

pub(super) fn output_job_id_for_includes(
    project_id: Uuid,
    prefix: &str,
    include: &[ArtifactKind],
) -> Uuid {
    if let [kind] = include {
        return output_job_id(project_id, prefix, *kind);
    }
    let scopes = include
        .iter()
        .map(|kind| output_job_kind_scope(*kind))
        .collect::<Vec<_>>()
        .join(",");
    Uuid::new_v5(
        &project_id,
        format!("datum-eda:output-job:{scopes}:{prefix}").as_bytes(),
    )
}

pub(super) fn output_job_id(project_id: Uuid, prefix: &str, kind: ArtifactKind) -> Uuid {
    Uuid::new_v5(
        &project_id,
        format!(
            "datum-eda:output-job:{}:{prefix}",
            output_job_kind_scope(kind)
        )
        .as_bytes(),
    )
}

pub(super) fn parse_output_job_include(include: &str) -> Result<Vec<ArtifactKind>> {
    let mut parsed = Vec::new();
    for raw_scope in include.split(',') {
        let scope = raw_scope.trim();
        if scope.is_empty() {
            continue;
        }
        let kind = match scope {
            "gerber-set" => ArtifactKind::GerberSet,
            "manufacturing-set" | "all" => ArtifactKind::ManufacturingSet,
            "bom" => ArtifactKind::Bom,
            "pnp" => ArtifactKind::Pnp,
            "drill" => ArtifactKind::Drill,
            _ => bail!(
                "unsupported output job include scope: {scope}; supported scopes: gerber-set, manufacturing-set, bom, pnp, drill, all"
            ),
        };
        if !parsed.contains(&kind) {
            parsed.push(kind);
        }
    }
    if parsed.is_empty() {
        bail!("output job requires at least one include scope");
    }
    Ok(parsed)
}

pub(super) fn output_job_include_label(include: &[ArtifactKind]) -> String {
    include
        .iter()
        .map(|kind| output_job_kind_label(*kind))
        .collect::<Vec<_>>()
        .join(" + ")
}

pub(super) fn output_job_kind_scope(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::GerberSet => "gerber-set",
        ArtifactKind::ManufacturingSet => "manufacturing-set",
        ArtifactKind::Bom => "bom",
        ArtifactKind::Pnp => "pnp",
        ArtifactKind::Drill => "drill",
    }
}

fn output_job_kind_label(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::GerberSet => "Gerber set",
        ArtifactKind::ManufacturingSet => "Manufacturing set",
        ArtifactKind::Bom => "BOM",
        ArtifactKind::Pnp => "PnP",
        ArtifactKind::Drill => "Drill",
    }
}
