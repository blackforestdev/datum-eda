use super::*;

pub(super) fn cli_surface_result(cli: &Cli) -> Result<String> {
    let roundtrip_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--undo")
        .arg("1")
        .arg("--redo")
        .arg("1")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI modify parity probe")?;
    if !roundtrip_output.status.success() {
        bail!(
            "CLI roundtrip parity probe failed with status {:?}: {}",
            roundtrip_output.status.code(),
            String::from_utf8_lossy(&roundtrip_output.stderr).trim()
        );
    }
    let roundtrip: CliModifyReport = serde_json::from_slice(&roundtrip_output.stdout)
        .context("failed to parse CLI roundtrip JSON output")?;

    let expected_actions = vec![
        format!("delete_track {}", cli.track_uuid),
        "undo".to_string(),
        "redo".to_string(),
    ];
    if roundtrip.actions != expected_actions {
        bail!(
            "CLI roundtrip actions mismatch: expected {:?}, got {:?}",
            expected_actions,
            roundtrip.actions
        );
    }
    let expected_last_description = format!("redo delete_track {}", cli.track_uuid);
    if roundtrip
        .last_result
        .as_ref()
        .map(|result| result.description.as_str())
        != Some(expected_last_description.as_str())
    {
        bail!("CLI roundtrip last_result description mismatch");
    }

    let target = unique_temp_path("cli-surface-save", "kicad_pcb");
    let save_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI save parity probe")?;
    if !save_output.status.success() {
        bail!(
            "CLI save parity probe failed with status {:?}: {}",
            save_output.status.code(),
            String::from_utf8_lossy(&save_output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&save_output.stdout)
        .context("failed to parse CLI save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_delete_state(cli)? {
        bail!("CLI save did not persist the current deleted board state");
    }

    let check_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "check",
        ])
        .arg(saved_path)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-track follow-up check")?;
    if !check_output.status.success() {
        bail!(
            "CLI delete-track follow-up check failed with status {:?}: {}",
            check_output.status.code(),
            String::from_utf8_lossy(&check_output.stderr).trim()
        );
    }
    let check_payload: Value = serde_json::from_slice(&check_output.stdout)
        .context("failed to parse CLI delete-track follow-up check JSON")?;
    let diagnostics = check_payload["diagnostics"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-track follow-up check missing diagnostics"))?;
    if !diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "net_without_copper")
    {
        bail!("CLI delete-track follow-up check missing net_without_copper");
    }

    Ok(format!(
        "roundtrip_last={}, saved={}, delete_then_save_persisted=true, delete_track_followup_check_changed=true",
        expected_last_description, saved_path
    ))
}

pub(super) fn cli_via_surface_result(cli: &Cli) -> Result<String> {
    let via_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let target = unique_temp_path("cli-surface-via-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&via_fixture)
        .arg("--delete-via")
        .arg(via_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI via save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI via save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI via save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI via save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let expected = after_delete_via_state(&via_fixture, via_uuid)?;
    if reloaded.get_net_info()? != expected {
        bail!("CLI via save did not persist the current deleted board state");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-via follow-up net query")?;
    if !query_output.status.success() {
        bail!(
            "CLI delete-via follow-up net query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI delete-via follow-up net JSON")?;
    let nets = payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing nets"))?;
    let gnd = nets
        .iter()
        .find(|net| net["name"] == "GND")
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing GND"))?;
    if gnd["vias"] != 0 {
        bail!("CLI delete-via follow-up net query did not reflect removed via");
    }
    Ok(format!(
        "via_saved={}, delete_via_then_save_persisted=true, delete_via_followup_net_info_changed=true",
        saved_path
    ))
}

pub(super) fn cli_component_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-component-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-component save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI delete-component save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI delete-component save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-component save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-component follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI delete-component follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI delete-component follow-up components JSON")?;
    let components = payload["components"].as_array().ok_or_else(|| {
        anyhow::anyhow!("CLI delete-component follow-up query missing components")
    })?;
    if components
        .iter()
        .any(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
    {
        bail!("CLI delete-component follow-up query still included deleted component");
    }
    Ok(format!(
        "component_saved={}, delete_component_then_save_persisted=true, delete_component_followup_components_changed=true",
        saved_path
    ))
}

pub(super) fn cli_rule_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let target = unique_temp_path("cli-surface-rule-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&fixture)
        .arg("--set-clearance-min-nm")
        .arg("125000")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI rule save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI rule save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI rule save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    if reloaded.get_design_rules()?.len() != 1 {
        bail!("CLI rule save did not persist one design rule");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("design-rules")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule follow-up query")?;
    if !query_output.status.success() {
        bail!(
            "CLI rule follow-up query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI rule follow-up query JSON")?;
    let rules = payload["rules"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI rule follow-up query missing rules array"))?;
    if rules.len() != 1 || rules[0]["name"] != "default clearance" {
        bail!("CLI rule follow-up query did not reflect current design-rule state");
    }
    Ok(format!(
        "rule_saved={}, set_design_rule_then_save_persisted=true, rule_followup_query_changed=true",
        saved_path
    ))
}

pub(super) fn cli_value_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-value-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-value")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:22k")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-value save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-value save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-value follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-value follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing components"))?;
    let r1 = components
        .iter()
        .find(|component| component["reference"] == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing R1"))?;
    if r1["value"] != "22k" {
        bail!("CLI set-value follow-up query did not reflect updated component value");
    }
    Ok(format!(
        "value_saved={}, set_value_then_save_persisted=true, set_value_followup_components_changed=true",
        saved_path
    ))
}

pub(super) fn cli_reference_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-reference-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-reference")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:R10")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-reference save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-reference save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-reference follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-reference follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| {
            anyhow::anyhow!("CLI set-reference follow-up query missing target component")
        })?;
    if target_component["reference"] != "R10" {
        bail!("CLI set-reference follow-up query did not reflect updated component reference");
    }
    Ok(format!(
        "reference_saved={}, set_reference_then_save_persisted=true, set_reference_followup_components_changed=true",
        saved_path
    ))
}
