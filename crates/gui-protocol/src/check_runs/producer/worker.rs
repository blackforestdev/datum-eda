//! One owned guarded ERC process; only actual producer acceptance means Running.
use super::*;
use anyhow::{Context, Result};
use std::io::Read;
use std::process::{Command, Stdio};

/// Called on a worker. The callback runs as soon as the admitted producer emits
/// its acknowledgement, before waiting for computation or persisted readback.
pub fn run(
    request: &CheckProducerRequest,
    accepted: impl FnMut(CheckProducerAcceptance),
) -> Result<CheckRunReviewState> {
    let cli = crate::cli_bridge::cli_prefix()?;
    run_with_cli(&cli, request, accepted)
}

pub(super) fn produce_candidate_with_cli(
    cli: &[String],
    request: &CheckProducerRequest,
    mut accepted: impl FnMut(CheckProducerAcceptance),
) -> Result<super::CheckProducerCandidate> {
    let (program, prefix) = cli.split_first().context("Check producer is unavailable")?;
    let mut child = Command::new(program)
        // This request comes from the explicit manual GUI action and contains
        // no agent launch identity. Ambient parent metadata cannot name it.
        // Agent CLI/MCP provenance and commit-source policy remain unchanged.
        .env_remove("DATUM_AGENT_PROVENANCE")
        .args(prefix)
        .args(["--format", "json", "check", "run"])
        .arg(&request.project_root)
        .args(["--profile", "erc", "--expected-project"])
        .arg(request.project_id.to_string())
        .arg("--expected-revision")
        .arg(&request.model_revision)
        .arg("--progress-job")
        .arg(request.job_id.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Start guarded ERC producer")?;
    let mut stdout = child.stdout.take().context("ERC output is unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("ERC acknowledgement stream is unavailable")?;
    let (status, output, receipt) = std::thread::scope(|scope| {
        // Both pipes must drain while the process runs; large finding output
        // cannot prevent the acceptance stream or the producer from progressing.
        let output = scope.spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        let receipt = super::acceptance_stream::read(stderr, request, &mut accepted);
        let status = child.wait();
        let output = output.join();
        (status, output, receipt)
    });
    let status = status.context("Wait for guarded ERC producer")?;
    let output = output.map_err(|_| anyhow::anyhow!("ERC output reader failed"))??;
    let mut receipt = receipt.context("Read ERC acknowledgement stream")?;
    anyhow::ensure!(
        status.success(),
        "Guarded ERC producer exited with {status}; {} stderr bytes drained; diagnostic tail: {}",
        receipt.stderr_bytes,
        String::from_utf8_lossy(receipt.diagnostic_tail.make_contiguous())
            .trim()
            .escape_debug()
    );
    anyhow::ensure!(
        receipt.error.is_none(),
        "ERC acceptance failed: {}",
        receipt.error.unwrap_or_default()
    );
    anyhow::ensure!(
        receipt.accepted,
        "ERC producer did not acknowledge the requested basis"
    );
    let produced = crate::check_run_review_state_from_json(std::str::from_utf8(&output)?)?;
    anyhow::ensure!(
        request.admits_run(&produced),
        "ERC result does not match the admitted job"
    );
    Ok(super::CheckProducerCandidate { run: produced })
}

// Preserve the strict public transport API for existing consumers. The GUI
// instead consumes the candidate through one serialized canonical readback.
pub(super) fn run_with_cli(
    cli: &[String],
    request: &CheckProducerRequest,
    accepted: impl FnMut(CheckProducerAcceptance),
) -> Result<CheckRunReviewState> {
    let produced = produce_candidate_with_cli(cli, request, accepted)?.run;
    let id = produced
        .check_run_id
        .as_deref()
        .context("ERC result has no persisted identity")?;
    let root = request
        .project_root
        .to_str()
        .context("Project root is not UTF-8")?;
    let record: serde_json::Value =
        crate::run_cli_json(cli, &["check", "show", root, "--check-run", id])?;
    anyhow::ensure!(
        record.get("contract").and_then(serde_json::Value::as_str) == Some("check_run_record_v1")
            && record.get("project_id").and_then(serde_json::Value::as_str)
                == Some(request.project_id.to_string().as_str())
            && record
                .get("model_revision")
                .and_then(serde_json::Value::as_str)
                == Some(request.model_revision.as_str()),
        "Project changed before persisted ERC readback"
    );
    let persisted = crate::check_run_review_state_from_json(&serde_json::to_string(&record)?)?;
    anyhow::ensure!(
        request.admits_run(&persisted) && persisted == produced,
        "Persisted ERC record differs from its admitted producer result"
    );
    Ok(persisted)
}
