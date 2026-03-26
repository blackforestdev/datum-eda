use super::*;

pub(super) fn cli_set_net_class_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let mut engine = Engine::new()?;
    engine.import(&fixture)?;
    let net_uuid = engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("GND net missing from CLI set-net-class fixture"))?
        .uuid;

    let target = unique_temp_path("cli-surface-net-class-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&fixture)
        .arg("--set-net-class")
        .arg(format!(
            "{}:power:125000:250000:300000:600000",
            net_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-net-class save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-net-class save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-net-class save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-net-class follow-up net query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-net-class follow-up net query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-net-class follow-up net JSON")?;
    let nets = payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class follow-up query missing nets"))?;
    let gnd = nets
        .iter()
        .find(|net| net["uuid"] == net_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class follow-up query missing target net"))?;
    if gnd["class"] != "power" {
        bail!("CLI set-net-class follow-up query did not reflect updated class");
    }
    Ok(format!(
        "net_class_saved={}, set_net_class_then_save_persisted=true, set_net_class_followup_net_info_changed=true",
        saved_path
    ))
}

pub(super) fn cli_move_surface_result(cli: &Cli) -> Result<String> {
    let baseline_distance =
        cli_unrouted_distance(&cli.repo_root, &cli.roundtrip_board_fixture_path)?;
    let target = unique_temp_path("cli-surface-move-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--move-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:15:12:90")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI move save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI move save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI move save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI move save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let moved = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI move save missing R1 after reimport"))?;
    if moved.position.x != 15_000_000 || moved.position.y != 12_000_000 || moved.rotation != 90 {
        bail!("CLI move save did not persist expected moved component state");
    }
    let moved_distance = cli_unrouted_distance(&cli.repo_root, Path::new(saved_path))?;
    if moved_distance == baseline_distance {
        bail!("CLI follow-up query did not reflect moved-component derived state");
    }
    Ok(format!(
        "move_saved={}, move_component_then_save_persisted=true, cli_followup_unrouted_changed=true",
        saved_path
    ))
}

pub(super) fn cli_rotate_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-rotate-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--rotate-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:180")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rotate-component save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI rotate-component save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI rotate-component save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rotate-component follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI rotate-component follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI rotate-component follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component follow-up query missing target component"))?;
    if target_component["rotation"] != 180 {
        bail!("CLI rotate-component follow-up query did not reflect updated rotation");
    }
    Ok(format!(
        "rotate_saved={}, rotate_component_then_save_persisted=true, rotate_component_followup_components_changed=true",
        saved_path
    ))
}
