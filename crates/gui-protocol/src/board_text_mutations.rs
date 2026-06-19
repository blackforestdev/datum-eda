//! Board-text mutation surface: native-board JSON edits driven by the
//! workspace board-text editing hit targets.

use super::*;

pub fn toggle_board_text_boolean_field(
    backing: &WorkspaceBacking,
    object_id: &str,
    field: BoardTextBooleanField,
) -> Result<bool> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let field_name = match field {
        BoardTextBooleanField::Mirrored => "mirrored",
        BoardTextBooleanField::KeepUpright => "keep_upright",
        BoardTextBooleanField::Bold => "bold",
    };
    let next = !text
        .get(field_name)
        .and_then(Value::as_bool)
        .unwrap_or(false);
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(field_name.to_string(), Value::Bool(next));
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}

pub fn cycle_board_text_alignment_field(
    backing: &WorkspaceBacking,
    object_id: &str,
    field: BoardTextAlignmentField,
) -> Result<String> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let (field_name, next_value) = match field {
        BoardTextAlignmentField::Horizontal => {
            let current = text
                .get("h_align")
                .and_then(Value::as_str)
                .unwrap_or("left");
            ("h_align", next_h_align(current)?)
        }
        BoardTextAlignmentField::Vertical => {
            let current = text
                .get("v_align")
                .and_then(Value::as_str)
                .unwrap_or("bottom");
            ("v_align", next_v_align(current)?)
        }
    };
    let next = next_value.to_string();
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(field_name.to_string(), Value::String(next.clone()));
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}

pub fn step_board_text_line_spacing_ratio(
    backing: &WorkspaceBacking,
    object_id: &str,
    step: BoardTextLineSpacingStep,
) -> Result<i32> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let current = text
        .get("line_spacing_ratio_ppm")
        .and_then(Value::as_i64)
        .unwrap_or(1_000_000) as i32;
    let next = match step {
        BoardTextLineSpacingStep::Decrease => current - BOARD_TEXT_LINE_SPACING_STEP_PPM,
        BoardTextLineSpacingStep::Increase => current + BOARD_TEXT_LINE_SPACING_STEP_PPM,
    }
    .clamp(
        BOARD_TEXT_LINE_SPACING_MIN_PPM,
        BOARD_TEXT_LINE_SPACING_MAX_PPM,
    );
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(
            "line_spacing_ratio_ppm".to_string(),
            Value::Number(serde_json::Number::from(next)),
        );
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}

pub fn step_board_text_height(
    backing: &WorkspaceBacking,
    object_id: &str,
    step: BoardTextHeightStep,
) -> Result<i64> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    let current_height = text_object
        .get("height_nm")
        .and_then(Value::as_i64)
        .unwrap_or_else(default_board_text_height_nm);
    if current_height <= 0 {
        anyhow::bail!("board text height must be positive before resize: {current_height}");
    }
    let current_stroke = text_object
        .get("stroke_width_nm")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| default_stroke_width_nm(current_height));
    if current_stroke <= 0 {
        anyhow::bail!("board text stroke width must be positive before resize: {current_stroke}");
    }
    let delta = ((current_height as i128 * BOARD_TEXT_HEIGHT_STEP_PPM as i128) / 1_000_000_i128)
        .max(1_i128) as i64;
    let next_height = match step {
        BoardTextHeightStep::Decrease => current_height - delta,
        BoardTextHeightStep::Increase => current_height + delta,
    }
    .clamp(BOARD_TEXT_HEIGHT_MIN_NM, BOARD_TEXT_HEIGHT_MAX_NM);
    let next_stroke = ((current_stroke as i128 * next_height as i128
        + (current_height as i128 / 2_i128))
        / current_height as i128)
        .max(1_i128) as i64;
    text_object.insert(
        "height_nm".to_string(),
        Value::Number(serde_json::Number::from(next_height)),
    );
    text_object.insert(
        "stroke_width_nm".to_string(),
        Value::Number(serde_json::Number::from(next_stroke)),
    );
    write_json_file(&backing.board_path, &board)?;
    Ok(next_height)
}

pub fn set_board_text_height(
    backing: &WorkspaceBacking,
    object_id: &str,
    height_nm: i64,
) -> Result<i64> {
    if !(BOARD_TEXT_HEIGHT_MIN_NM..=BOARD_TEXT_HEIGHT_MAX_NM).contains(&height_nm) {
        anyhow::bail!(
            "board text height must be between {} nm and {} nm",
            BOARD_TEXT_HEIGHT_MIN_NM,
            BOARD_TEXT_HEIGHT_MAX_NM
        );
    }
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    let current_height = text_object
        .get("height_nm")
        .and_then(Value::as_i64)
        .unwrap_or_else(default_board_text_height_nm);
    if current_height <= 0 {
        anyhow::bail!("board text height must be positive before resize: {current_height}");
    }
    let current_stroke = text_object
        .get("stroke_width_nm")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| default_stroke_width_nm(current_height));
    if current_stroke <= 0 {
        anyhow::bail!("board text stroke width must be positive before resize: {current_stroke}");
    }
    let next_stroke = ((current_stroke as i128 * height_nm as i128
        + (current_height as i128 / 2_i128))
        / current_height as i128)
        .max(1_i128) as i64;
    text_object.insert(
        "height_nm".to_string(),
        Value::Number(serde_json::Number::from(height_nm)),
    );
    text_object.insert(
        "stroke_width_nm".to_string(),
        Value::Number(serde_json::Number::from(next_stroke)),
    );
    write_json_file(&backing.board_path, &board)?;
    Ok(height_nm)
}

pub fn step_board_text_rotation(
    backing: &WorkspaceBacking,
    object_id: &str,
    step: BoardTextRotationStep,
) -> Result<i32> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    let current = text_object
        .get("rotation")
        .and_then(Value::as_i64)
        .unwrap_or(0) as i32;
    let delta = match step {
        BoardTextRotationStep::CounterClockwise90 => -90,
        BoardTextRotationStep::Clockwise90 => 90,
    };
    let next = (current + delta).rem_euclid(360);
    text_object.insert(
        "rotation".to_string(),
        Value::Number(serde_json::Number::from(next)),
    );
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}

pub fn set_board_text_rotation(
    backing: &WorkspaceBacking,
    object_id: &str,
    rotation_degrees: i32,
) -> Result<i32> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let next = rotation_degrees.rem_euclid(360);
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(
            "rotation".to_string(),
            Value::Number(serde_json::Number::from(next)),
        );
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}

pub fn set_board_text_line_spacing_ratio(
    backing: &WorkspaceBacking,
    object_id: &str,
    ratio_ppm: i32,
) -> Result<i32> {
    if !(BOARD_TEXT_LINE_SPACING_MIN_PPM..=BOARD_TEXT_LINE_SPACING_MAX_PPM).contains(&ratio_ppm) {
        anyhow::bail!(
            "board text line spacing ratio must be between {} ppm and {} ppm",
            BOARD_TEXT_LINE_SPACING_MIN_PPM,
            BOARD_TEXT_LINE_SPACING_MAX_PPM
        );
    }
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(
            "line_spacing_ratio_ppm".to_string(),
            Value::Number(serde_json::Number::from(ratio_ppm)),
        );
    write_json_file(&backing.board_path, &board)?;
    Ok(ratio_ppm)
}

pub fn set_board_text_content(
    backing: &WorkspaceBacking,
    object_id: &str,
    content: &str,
) -> Result<String> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert("text".to_string(), Value::String(content.to_string()));
    write_json_file(&backing.board_path, &board)?;
    Ok(content.to_string())
}

pub fn set_board_text_alignment(
    backing: &WorkspaceBacking,
    object_id: &str,
    h_align: &str,
    v_align: &str,
) -> Result<(String, String)> {
    validate_h_align(h_align)?;
    validate_v_align(v_align)?;
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    text_object.insert("h_align".to_string(), Value::String(h_align.to_string()));
    text_object.insert("v_align".to_string(), Value::String(v_align.to_string()));
    write_json_file(&backing.board_path, &board)?;
    Ok((h_align.to_string(), v_align.to_string()))
}

pub fn set_board_text_h_align(
    backing: &WorkspaceBacking,
    object_id: &str,
    h_align: &str,
) -> Result<String> {
    validate_h_align(h_align)?;
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert("h_align".to_string(), Value::String(h_align.to_string()));
    write_json_file(&backing.board_path, &board)?;
    Ok(h_align.to_string())
}

pub fn set_board_text_v_align(
    backing: &WorkspaceBacking,
    object_id: &str,
    v_align: &str,
) -> Result<String> {
    validate_v_align(v_align)?;
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert("v_align".to_string(), Value::String(v_align.to_string()));
    write_json_file(&backing.board_path, &board)?;
    Ok(v_align.to_string())
}

pub fn set_board_text_render_intent(
    backing: &WorkspaceBacking,
    object_id: &str,
    render_intent: &str,
) -> Result<String> {
    validate_render_intent(render_intent)?;
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    text.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?
        .insert(
            "render_intent".to_string(),
            Value::String(render_intent.to_string()),
        );
    write_json_file(&backing.board_path, &board)?;
    Ok(render_intent.to_string())
}

pub fn set_board_text_font_family(
    backing: &WorkspaceBacking,
    object_id: &str,
    family: &str,
) -> Result<String> {
    validate_font_family(family)?;
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    let family_id = TextFamilyId(family.to_string());
    text_object.insert("family".to_string(), Value::String(family.to_string()));
    text_object.insert(
        "family_source".to_string(),
        Value::String(family_source_to_string(TextFamilySource::Explicit).to_string()),
    );
    text_object.insert(
        "style".to_string(),
        Value::String(default_style_for_family(&family_id).0),
    );
    write_json_file(&backing.board_path, &board)?;
    Ok(family.to_string())
}

pub fn cycle_board_text_field(
    backing: &WorkspaceBacking,
    object_id: &str,
    field: BoardTextCycleField,
) -> Result<String> {
    let text_uuid = object_id
        .strip_prefix("board-text:")
        .ok_or_else(|| anyhow::anyhow!("selected object is not board text: {object_id}"))?;
    let mut board: Value = serde_json::from_str(
        &std::fs::read_to_string(&backing.board_path)
            .with_context(|| format!("failed to read {}", backing.board_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", backing.board_path.display()))?;
    let texts = board
        .get_mut("texts")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow::anyhow!("board JSON missing mutable texts array"))?;
    let text = texts
        .iter_mut()
        .find(|entry| entry.get("uuid").and_then(Value::as_str) == Some(text_uuid))
        .ok_or_else(|| anyhow::anyhow!("board text not found: {text_uuid}"))?;
    let text_object = text
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("board text entry is not an object: {text_uuid}"))?;
    let next = match field {
        BoardTextCycleField::RenderIntent => {
            let current = text_object
                .get("render_intent")
                .and_then(Value::as_str)
                .unwrap_or("manufacturing");
            let next = next_render_intent(current)?;
            text_object.insert("render_intent".to_string(), Value::String(next.to_string()));
            next.to_string()
        }
        BoardTextCycleField::Family => {
            let current = text_object
                .get("family")
                .and_then(Value::as_str)
                .unwrap_or(FAMILY_NEWSTROKE);
            let next = next_font_family(current)?;
            let family = TextFamilyId(next.to_string());
            text_object.insert("family".to_string(), Value::String(next.to_string()));
            text_object.insert(
                "family_source".to_string(),
                Value::String(family_source_to_string(TextFamilySource::Explicit).to_string()),
            );
            text_object.insert(
                "style".to_string(),
                Value::String(default_style_for_family(&family).0),
            );
            next.to_string()
        }
    };
    write_json_file(&backing.board_path, &board)?;
    Ok(next)
}
