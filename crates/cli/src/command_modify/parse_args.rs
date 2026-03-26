use super::*;

pub(crate) fn parse_move_component_arg(value: &str) -> Result<MoveComponentInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 && parts.len() != 4 {
        bail!("--move-component expects <uuid>:<x_mm>:<y_mm>[:<rotation_deg>]");
    }
    let uuid = Uuid::parse_str(parts[0])?;
    let x_mm = parts[1].parse::<f64>()?;
    let y_mm = parts[2].parse::<f64>()?;
    let rotation = if parts.len() == 4 {
        Some(parts[3].parse::<i32>()?)
    } else {
        None
    };
    Ok(MoveComponentInput {
        uuid,
        position: eda_engine::ir::geometry::Point::new(
            eda_engine::ir::units::mm_to_nm(x_mm),
            eda_engine::ir::units::mm_to_nm(y_mm),
        ),
        rotation,
    })
}

pub(crate) fn parse_set_value_arg(value: &str) -> Result<SetValueInput> {
    let (uuid, component_value) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-value expects <uuid>:<value>"))?;
    Ok(SetValueInput {
        uuid: Uuid::parse_str(uuid)?,
        value: component_value.to_string(),
    })
}

pub(crate) fn parse_rotate_component_arg(value: &str) -> Result<RotateComponentInput> {
    let (uuid, rotation) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--rotate-component expects <uuid>:<rotation_deg>"))?;
    Ok(RotateComponentInput {
        uuid: Uuid::parse_str(uuid)?,
        rotation: rotation.parse::<i32>()?,
    })
}

pub(crate) fn parse_set_reference_arg(value: &str) -> Result<SetReferenceInput> {
    let (uuid, reference) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-reference expects <uuid>:<reference>"))?;
    Ok(SetReferenceInput {
        uuid: Uuid::parse_str(uuid)?,
        reference: reference.to_string(),
    })
}

pub(crate) fn parse_assign_part_arg(value: &str) -> Result<AssignPartInput> {
    let (uuid, part_uuid) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--assign-part expects <uuid>:<part_uuid>"))?;
    Ok(AssignPartInput {
        uuid: Uuid::parse_str(uuid)?,
        part_uuid: Uuid::parse_str(part_uuid)?,
    })
}

pub(crate) fn parse_set_package_arg(value: &str) -> Result<SetPackageInput> {
    let (uuid, package_uuid) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-package expects <uuid>:<package_uuid>"))?;
    Ok(SetPackageInput {
        uuid: Uuid::parse_str(uuid)?,
        package_uuid: Uuid::parse_str(package_uuid)?,
    })
}

pub(crate) fn parse_set_package_with_part_arg(value: &str) -> Result<SetPackageWithPartInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 {
        bail!("--set-package-with-part expects <uuid>:<package_uuid>:<part_uuid>");
    }
    Ok(SetPackageWithPartInput {
        uuid: Uuid::parse_str(parts[0])?,
        package_uuid: Uuid::parse_str(parts[1])?,
        part_uuid: Uuid::parse_str(parts[2])?,
    })
}

pub(crate) fn parse_replace_component_arg(value: &str) -> Result<ReplaceComponentInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 {
        bail!("--replace-component expects <uuid>:<package_uuid>:<part_uuid>");
    }
    Ok(ReplaceComponentInput {
        uuid: Uuid::parse_str(parts[0])?,
        package_uuid: Uuid::parse_str(parts[1])?,
        part_uuid: Uuid::parse_str(parts[2])?,
    })
}

pub(crate) fn parse_apply_replacement_plan_arg(
    value: &str,
) -> Result<PlannedComponentReplacementInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 && parts.len() != 5 {
        bail!(
            "--apply-replacement-plan expects <uuid>:package:<package_uuid> | <uuid>:part:<part_uuid> | <uuid>:package:<package_uuid>:part:<part_uuid>"
        );
    }
    let uuid = Uuid::parse_str(parts[0])?;
    let mut package_uuid = None;
    let mut part_uuid = None;
    let mut index = 1;
    while index + 1 < parts.len() {
        match parts[index] {
            "package" => package_uuid = Some(Uuid::parse_str(parts[index + 1])?),
            "part" => part_uuid = Some(Uuid::parse_str(parts[index + 1])?),
            other => bail!(
                "--apply-replacement-plan selector must be 'package' or 'part', got {other}"
            ),
        }
        index += 2;
    }
    Ok(PlannedComponentReplacementInput {
        uuid,
        package_uuid,
        part_uuid,
    })
}

pub(crate) fn parse_apply_replacement_policy_arg(
    value: &str,
) -> Result<PolicyDrivenComponentReplacementInput> {
    let (uuid, selector) = value.split_once(':').ok_or_else(|| {
        anyhow::anyhow!("--apply-replacement-policy expects <uuid>:package|part")
    })?;
    let policy = match selector {
        "package" => ComponentReplacementPolicy::BestCompatiblePackage,
        "part" => ComponentReplacementPolicy::BestCompatiblePart,
        other => bail!("--apply-replacement-policy selector must be 'package' or 'part', got {other}"),
    };
    Ok(PolicyDrivenComponentReplacementInput {
        uuid: Uuid::parse_str(uuid)?,
        policy,
    })
}

pub(crate) fn parse_apply_scoped_replacement_policy_arg(
    value: &str,
) -> Result<ScopedComponentReplacementPolicyInput> {
    let mut parts = value.split(':');
    let policy = match parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("--apply-scoped-replacement-policy expects a policy"))?
    {
        "package" => ComponentReplacementPolicy::BestCompatiblePackage,
        "part" => ComponentReplacementPolicy::BestCompatiblePart,
        other => bail!(
            "--apply-scoped-replacement-policy policy must be 'package' or 'part', got {other}"
        ),
    };
    let mut scope = ComponentReplacementScope::default();
    for segment in parts {
        let (key, raw_value) = segment.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "--apply-scoped-replacement-policy scope selectors must use key=value segments"
            )
        })?;
        match key {
            "ref_prefix" => scope.reference_prefix = Some(raw_value.to_string()),
            "value" => scope.value_equals = Some(raw_value.to_string()),
            "package_uuid" => scope.current_package_uuid = Some(Uuid::parse_str(raw_value)?),
            "part_uuid" => scope.current_part_uuid = Some(Uuid::parse_str(raw_value)?),
            other => bail!(
                "--apply-scoped-replacement-policy selector must be one of ref_prefix,value,package_uuid,part_uuid; got {other}"
            ),
        }
    }
    Ok(ScopedComponentReplacementPolicyInput { scope, policy })
}

pub(crate) fn parse_set_net_class_arg(value: &str) -> Result<SetNetClassInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 6 && parts.len() != 8 {
        bail!(
            "--set-net-class expects <net_uuid>:<class_name>:<clearance_nm>:<track_width_nm>:<via_drill_nm>:<via_diameter_nm>[:<diffpair_width_nm>:<diffpair_gap_nm>]"
        );
    }
    Ok(SetNetClassInput {
        net_uuid: Uuid::parse_str(parts[0])?,
        class_name: parts[1].to_string(),
        clearance: parts[2].parse::<i64>()?,
        track_width: parts[3].parse::<i64>()?,
        via_drill: parts[4].parse::<i64>()?,
        via_diameter: parts[5].parse::<i64>()?,
        diffpair_width: if parts.len() == 8 {
            parts[6].parse::<i64>()?
        } else {
            0
        },
        diffpair_gap: if parts.len() == 8 {
            parts[7].parse::<i64>()?
        } else {
            0
        },
    })
}
