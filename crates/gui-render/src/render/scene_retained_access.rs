use super::*;

pub(crate) fn finish_retained_draw_commands(
    commands: &mut Vec<RetainedDrawCommand>,
    layer_id: Option<String>,
    quad_start: usize,
    quad_end: usize,
    stroke_start: usize,
    stroke_end: usize,
) {
    if quad_end > quad_start {
        append_retained_draw_command(commands, RetainedDrawCommand::Quads {
            layer_id: layer_id.clone(),
            range: (quad_start * 6) as u32..(quad_end * 6) as u32,
        });
    }
    if stroke_end > stroke_start {
        append_retained_draw_command(commands, RetainedDrawCommand::Strokes {
            layer_id,
            range: stroke_start as u32..stroke_end as u32,
        });
    }
}

fn append_retained_draw_command(
    commands: &mut Vec<RetainedDrawCommand>,
    command: RetainedDrawCommand,
) {
    let merged = match (commands.last_mut(), &command) {
        (
            Some(RetainedDrawCommand::Quads { layer_id: previous_layer, range: previous }),
            RetainedDrawCommand::Quads { layer_id, range },
        ) | (
            Some(RetainedDrawCommand::Strokes { layer_id: previous_layer, range: previous }),
            RetainedDrawCommand::Strokes { layer_id, range },
        ) if previous_layer == layer_id && previous.end == range.start => {
            previous.end = range.end;
            true
        }
        _ => false,
    };
    if !merged {
        commands.push(command);
    }
}

pub(crate) fn sort_retained_draw_commands(
    commands: &mut [RetainedDrawCommand],
    layers: &[datum_gui_protocol::SceneLayer],
) {
    commands.sort_by_key(|command| {
        let layer_id = match command {
            RetainedDrawCommand::Quads { layer_id, .. }
            | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id.as_deref(),
        };
        layer_id
            .map(|id| scene_layer_stack_priority(id, layers))
            .unwrap_or(u32::MAX)
    });
}

impl RetainedScene {
    pub fn world_vertices(&self) -> &[Vertex] { &self.world_vertices }
    pub(crate) fn world_strokes(&self) -> &[WorldStrokeInstance] { &self.world_strokes }

    pub(crate) fn visible_draw_commands(&self, state: &ReviewWorkspaceState) -> Vec<RetainedDrawCommand> {
        if !authored_visible(state) { return Vec::new(); }
        self.draw_commands.iter()
            .filter(|command| match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                | RetainedDrawCommand::Strokes { layer_id, .. } =>
                    layer_id.as_deref().is_none_or(|id| layer_visible(state, id)),
            })
            .cloned()
            .collect()
    }

    pub(crate) fn all_draw_commands(&self) -> &[RetainedDrawCommand] { &self.draw_commands }

}
