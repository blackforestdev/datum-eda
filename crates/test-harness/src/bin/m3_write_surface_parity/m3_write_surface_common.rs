use super::*;

pub(super) fn after_delete_state(cli: &Cli) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    engine.delete_track(&cli.track_uuid)?;
    Ok(engine.get_net_info()?)
}

pub(super) fn after_delete_via_state(
    fixture: &Path,
    via_uuid: Uuid,
) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(fixture)?;
    engine.delete_via(&via_uuid)?;
    Ok(engine.get_net_info()?)
}

pub(super) fn run_command_checked(command: &mut Command, label: &str) -> Result<String> {
    let output = command
        .output()
        .with_context(|| format!("failed to execute {label}"))?;
    if !output.status.success() {
        bail!(
            "{label} failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(super) fn cli_unrouted_distance(repo_root: &Path, board_path: &Path) -> Result<i64> {
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(board_path)
        .arg("unrouted")
        .current_dir(repo_root)
        .output()
        .context("failed to run CLI unrouted query")?;
    if !output.status.success() {
        bail!(
            "CLI unrouted query failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI unrouted JSON output")?;
    payload["airwires"][0]["distance_nm"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("CLI unrouted JSON missing first airwire distance"))
}

pub(super) fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{unique}.{extension}",
        std::process::id()
    ))
}
