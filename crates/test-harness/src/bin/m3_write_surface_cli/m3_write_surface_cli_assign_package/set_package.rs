use super::*;

pub(crate) fn cli_set_package_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let package_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;

    let target = unique_temp_path("cli-surface-set-package-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--set-package")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            package_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package save report missing saved_path"))?;
    let saved_contents =
        std::fs::read_to_string(saved_path).context("failed to read CLI set-package saved board")?;
    if !saved_contents.contains("(footprint \"ALT-3\"") {
        bail!("CLI set-package save did not rewrite expected footprint name");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-package follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-package follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up query missing target component"))?;
    if target_component["package_uuid"] != package_uuid.to_string() {
        bail!("CLI set-package follow-up query did not reflect updated package assignment");
    }
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(1) {
        bail!(
            "CLI set-package follow-up net query did not reflect regenerated package connectivity"
        );
    }
    Ok(format!(
        "package_saved={}, set_package_then_save_persisted=true, set_package_rewrote_footprint=true, set_package_followup_components_changed=true, set_package_followup_net_info_changed=true",
        saved_path
    ))
}

pub(crate) fn cli_set_package_remap_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let altamp_package_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;

    let target = unique_temp_path("cli-surface-set-package-remap-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            lmv321_part_uuid
        ))
        .arg("--set-package")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            altamp_package_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package remap save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package remap save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package remap save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap save report missing saved_path"))?;
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package remap follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package remap follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package remap follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(2) {
        bail!("CLI set-package remap did not preserve logical net connectivity");
    }
    Ok(format!(
        "package_remap_saved={}, set_package_logical_remap_preserved=true",
        saved_path
    ))
}

pub(crate) fn cli_set_package_with_part_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let altamp = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?;

    let target = unique_temp_path("cli-surface-set-package-with-part-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            lmv321_part_uuid
        ))
        .arg("--set-package-with-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}:{}",
            altamp.package_uuid, altamp.uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package-with-part save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package-with-part save JSON output")?;
    let saved_path = save.saved_path.as_deref().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part save report missing saved_path")
    })?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-package-with-part follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-package-with-part follow-up components JSON")?;
    let components = payload["components"].as_array().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part follow-up query missing components")
    })?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package-with-part query missing target component"))?;
    if target_component["package_uuid"] != altamp.package_uuid.to_string() {
        bail!("CLI set-package-with-part follow-up query did not reflect updated package");
    }
    if target_component["value"] != "ALTAMP" {
        bail!("CLI set-package-with-part follow-up query did not reflect explicit part value");
    }
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package-with-part follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package-with-part follow-up net JSON")?;
    let nets = net_payload["nets"].as_array().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part follow-up query missing nets")
    })?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package-with-part follow-up query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(2) {
        bail!("CLI set-package-with-part did not preserve logical net connectivity");
    }
    Ok(format!(
        "package_with_part_saved={}, set_package_with_part_followup_components_changed=true, set_package_with_part_followup_net_info_changed=true",
        saved_path
    ))
}
