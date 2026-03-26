use super::*;

pub(super) fn engine_assign_part_surface_result(cli: &Cli) -> Result<String> {
    let library_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut assign_engine = Engine::new()?;
    assign_engine.import_eagle_library(&library_fixture)?;
    assign_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_assign_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import_eagle_library(&library_fixture)?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let part_uuid = assign_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;
    let assign_part = assign_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid,
    })?;
    let assign_target =
        m3_write_surface_common::unique_temp_path("engine-surface-assign-part-save", "kicad_pcb");
    assign_engine.save(&assign_target)?;
    let assign_saved = std::fs::read_to_string(&assign_target)?;
    if !assign_saved.contains("(footprint \"ALT-3\"") {
        bail!("saved assign_part component did not rewrite expected footprint name");
    }
    let mut reloaded_assign = Engine::new()?;
    reloaded_assign.import_eagle_library(&library_fixture)?;
    reloaded_assign.import(&assign_target)?;
    let reloaded_assign_components = reloaded_assign.get_components()?;
    let reloaded_assign_sig = reloaded_assign
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("reloaded assign_part nets missing SIG"))?;
    let baseline_assign_r1 = baseline_assign_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline assign_part components missing R1"))?;
    let updated_assign_r1 = reloaded_assign_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded assign_part components missing R1"))?;
    if updated_assign_r1.value != "ALTAMP" {
        bail!("saved assign_part component did not reimport expected value");
    }
    if baseline_assign_r1.value == updated_assign_r1.value {
        bail!("assign_part did not change engine follow-up components state");
    }
    if reloaded_assign_sig.pins.len() != 1 {
        bail!("assign_part did not change engine follow-up net-info state");
    }
    let lmv321_part_uuid = assign_engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let mut remap_engine = Engine::new()?;
    remap_engine.import_eagle_library(&library_fixture)?;
    remap_engine.import(&cli.roundtrip_board_fixture_path)?;
    remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let remap_intermediate_sig = remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate assign_part nets missing SIG"))?;
    remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid,
    })?;
    let remap_after_sig = remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("remapped assign_part nets missing SIG"))?;
    if remap_after_sig.pins.len() != remap_intermediate_sig.pins.len() {
        bail!("assign_part logical remap did not preserve engine follow-up net-info state");
    }

    Ok(format!(
        "assign_part={}, assign_saved={}, assign_part_rewrote_footprint=true, assign_part_followup_components_changed=true, assign_part_followup_net_info_changed=true, assign_part_logical_remap_preserved=true",
        assign_part.description,
        assign_target.display()
    ))
}

pub(super) fn engine_set_package_surface_result(cli: &Cli) -> Result<String> {
    let library_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut package_engine = Engine::new()?;
    package_engine.import_eagle_library(&library_fixture)?;
    package_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_package_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import_eagle_library(&library_fixture)?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let package_uuid = package_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;
    let set_package = package_engine.set_package(SetPackageInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        package_uuid,
    })?;
    let package_target =
        m3_write_surface_common::unique_temp_path("engine-surface-set-package-save", "kicad_pcb");
    package_engine.save(&package_target)?;
    let package_saved = std::fs::read_to_string(&package_target)?;
    if !package_saved.contains("(footprint \"ALT-3\"") {
        bail!("saved set_package component did not rewrite expected footprint name");
    }
    let mut reloaded_package = Engine::new()?;
    reloaded_package.import_eagle_library(&library_fixture)?;
    reloaded_package.import(&package_target)?;
    let reloaded_package_components = reloaded_package.get_components()?;
    let baseline_package_r1 = baseline_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline set_package components missing R1"))?;
    let updated_package_r1 = reloaded_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded set_package components missing R1"))?;
    if updated_package_r1.package_uuid != package_uuid {
        bail!("saved set_package component did not reimport expected package uuid");
    }
    if baseline_package_r1.package_uuid == updated_package_r1.package_uuid {
        bail!("set_package did not change engine follow-up components state");
    }
    let lmv321_part_uuid = package_engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let mut package_remap_engine = Engine::new()?;
    package_remap_engine.import_eagle_library(&library_fixture)?;
    package_remap_engine.import(&cli.roundtrip_board_fixture_path)?;
    package_remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let package_remap_intermediate_sig = package_remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate set_package nets missing SIG"))?;
    package_remap_engine.set_package(SetPackageInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        package_uuid,
    })?;
    let package_remap_after_sig = package_remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("remapped set_package nets missing SIG"))?;
    if package_remap_after_sig.pins.len() != package_remap_intermediate_sig.pins.len() {
        bail!("set_package logical remap did not preserve engine follow-up net-info state");
    }
    let altamp_part_uuid = package_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;
    let mut explicit_package_engine = Engine::new()?;
    explicit_package_engine.import_eagle_library(&library_fixture)?;
    explicit_package_engine.import(&cli.roundtrip_board_fixture_path)?;
    explicit_package_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let explicit_package_intermediate_sig = explicit_package_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate explicit set_package nets missing SIG"))?;
    let explicit_package = explicit_package_engine.set_package_with_part(
        eda_engine::api::SetPackageWithPartInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            package_uuid,
            part_uuid: altamp_part_uuid,
        },
    )?;
    let explicit_package_target = m3_write_surface_common::unique_temp_path(
        "engine-surface-set-package-with-part-save",
        "kicad_pcb",
    );
    explicit_package_engine.save(&explicit_package_target)?;
    let mut reloaded_explicit_package = Engine::new()?;
    reloaded_explicit_package.import_eagle_library(&library_fixture)?;
    reloaded_explicit_package.import(&explicit_package_target)?;
    let reloaded_explicit_package_components = reloaded_explicit_package.get_components()?;
    let explicit_package_component = reloaded_explicit_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded explicit package components missing R1"))?;
    let explicit_package_after_sig = explicit_package_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("explicit set_package nets missing SIG"))?;
    if explicit_package_component.package_uuid != package_uuid {
        bail!("set_package_with_part did not persist expected package uuid");
    }
    if explicit_package_component.value != "ALTAMP" {
        bail!("set_package_with_part did not persist expected explicit part value");
    }
    if explicit_package_after_sig.pins.len() != explicit_package_intermediate_sig.pins.len() {
        bail!("set_package_with_part did not preserve engine follow-up net-info state");
    }

    Ok(format!(
        "set_package={}, package_saved={}, set_package_followup_components_changed=true, set_package_followup_net_info_changed=true, set_package_logical_remap_preserved=true, set_package_with_part={}, explicit_package_saved={}, set_package_with_part_followup_net_info_changed=true",
        set_package.description,
        package_target.display(),
        explicit_package.description,
        explicit_package_target.display()
    ))
}

pub(super) fn engine_set_net_class_surface_result(cli: &Cli) -> Result<String> {
    let net_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let mut net_class_engine = Engine::new()?;
    net_class_engine.import(&net_fixture)?;
    let baseline_net_info = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&net_fixture)?;
        baseline_engine.get_net_info()?
    };
    let gnd_uuid = baseline_net_info
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("baseline net info missing GND"))?
        .uuid;
    let set_net_class = net_class_engine.set_net_class(SetNetClassInput {
        net_uuid: gnd_uuid,
        class_name: "power".to_string(),
        clearance: 125_000,
        track_width: 250_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 0,
        diffpair_gap: 0,
    })?;
    let net_class_target =
        m3_write_surface_common::unique_temp_path("engine-surface-net-class-save", "kicad_pcb");
    net_class_engine.save(&net_class_target)?;
    let mut reloaded_net_class = Engine::new()?;
    reloaded_net_class.import(&net_class_target)?;
    let reloaded_net_info = reloaded_net_class.get_net_info()?;
    let baseline_gnd = baseline_net_info
        .iter()
        .find(|net| net.uuid == gnd_uuid)
        .ok_or_else(|| anyhow::anyhow!("baseline net info missing GND uuid"))?;
    let updated_gnd = reloaded_net_info
        .iter()
        .find(|net| net.uuid == gnd_uuid)
        .ok_or_else(|| anyhow::anyhow!("reloaded net info missing GND uuid"))?;
    if updated_gnd.class != "power" {
        bail!("saved set_net_class net did not reimport expected class");
    }
    if baseline_gnd.class == updated_gnd.class {
        bail!("set_net_class did not change engine follow-up net-info state");
    }

    Ok(format!(
        "set_net_class={}, net_class_saved={}, set_net_class_followup_net_info_changed=true",
        set_net_class.description,
        net_class_target.display()
    ))
}
