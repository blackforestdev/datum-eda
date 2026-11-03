#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkingMenuState {
    pub menu_key: String,
    pub target_object_id: Option<String>,
    pub anchor_x_px: i32,
    pub anchor_y_px: i32,
    pub preview_slot: Option<String>,
    pub gesture_dx_px: i32,
    pub gesture_dy_px: i32,
    pub selection: Option<crate::selection_menu::SelectionMenu>,
    /// Open nested sub-wheel path within the active marking menu's `submenus`
    /// (empty = the top-level wheel; each entry names a submenu one level deeper,
    /// depth-2 cap). Navigation-only: traversing it never invokes an operation.
    pub submenu_path: Vec<String>,
}

impl MarkingMenuState {
    /// A typed action owns one captured authored identity, never the prior selection.
    pub fn schematic_context_target(&self) -> Option<&crate::AuthoredSelectionIdentity> {
        let menu = self.selection.as_ref().filter(|m| m.schematic)?;
        let mut candidates = menu.overlap_candidates();
        let target = candidates.next()?;
        if candidates.next().is_some()
            || !menu
                .preview_keys
                .get(target)?
                .contains(self.target_object_id.as_ref()?)
        {
            return None;
        }
        Some(target)
    }

    pub fn schematic_context_key(&self) -> &'static str {
        use crate::AuthoredSelectionClass as C;
        match self.schematic_context_target().map(|id| id.class) {
            Some(C::Symbol) => "schematic.symbol",
            Some(C::Wire) => "schematic.wire",
            Some(C::Label) => "schematic.label",
            _ if self.target_object_id.is_none()
                && self
                    .selection
                    .as_ref()
                    .is_some_and(|m| m.schematic && m.overlap_candidates().next().is_none()) =>
            {
                "schematic.empty"
            }
            _ => "schematic.selection",
        }
    }

    /// Shared paint/invocation admission for the existing schematic wheel leaves.
    /// Capability and clicked source identity are independent of selected Inspector state.
    pub fn schematic_action_unavailable_reason(
        &self,
        item: &crate::GuiMenuItem,
        state: &crate::ReviewWorkspaceState,
    ) -> Option<String> {
        self.schematic_action_admission(item, state).err()
    }

    fn schematic_action_admission(
        &self,
        item: &crate::GuiMenuItem,
        state: &crate::ReviewWorkspaceState,
    ) -> Result<(), String> {
        use crate::{AuthoredSelectionClass as C, GuiMenuBinding as B};
        let unavailable = "This action is not invoked in this build.";
        if !self.menu_key.starts_with("schematic.") {
            return Err(unavailable.into());
        }
        let menu = self.selection.as_ref().ok_or("Reopen the context menu")?;
        let basis = crate::AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        let scene = crate::camera_scene_for_pane(state, basis.pane)
            .ok_or("Schematic source unavailable")?;
        if !menu.schematic
            || basis.content != crate::PaneContent::Schematic
            || basis.project != menu.project
            || basis.revision != menu.revision
            || scene.scene_id != menu.source_scene
            || state.ui.filters != menu.filters
            || self.menu_key != self.schematic_context_key()
        {
            return Err("Context source changed; reopen the menu".into());
        }
        let target = self.schematic_context_target();
        if let Some(target) = target {
            if basis
                .document
                .as_ref()
                .is_none_or(|d| d.instance_path != target.instance_path)
            {
                return Err("Context occurrence changed; reopen the menu".into());
            }
            let sheet = state
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&basis.source))
                .ok_or("Schematic source unavailable")?;
            let present = match target.class {
                C::Symbol => sheet.symbols.contains_key(&target.id),
                C::Wire => sheet.wires.contains_key(&target.id),
                C::Label => sheet.labels.contains_key(&target.id),
                _ => false,
            };
            if !present {
                return Err("Context object changed; reopen the menu".into());
            }
        }
        let authoring = &state.schematic_authoring;
        if authoring.delete_review.is_some() {
            return Err("Accept or dismiss the deletion proposal first".into());
        }
        if authoring.active() || authoring.picker || authoring.creation.is_some() {
            return Err("Finish or cancel the current edit first".into());
        }
        let class = target.map(|id| id.class);
        let verb = match item.binding() {
            B::Verb("datum.schematic.set_symbol_part") if class == Some(C::Symbol) => {
                crate::schematic_authoring::assignment::AssignmentEvidence::admit_target(
                    state,
                    target.unwrap(),
                )?;
                return Ok(());
            }
            B::Verb("datum.schematic.move_symbol") if class == Some(C::Symbol) => {
                if let crate::SelectionTarget::Native(view) = &state.selection
                    && !view.stale
                    && view.project == menu.project
                    && view.revision.0 == menu.revision
                    && crate::footprint_move_preview::native::group::subject_contains(
                        &view.subject,
                        target.unwrap(),
                    )
                {
                    if !view.locked.is_empty() {
                        return Err(
                            "Move cannot change a selection containing locked objects".into()
                        );
                    }
                    if matches!(&view.subject, crate::SelectionSubject::Compound(_)) {
                        crate::footprint_move_preview::native::group::symbol_members(&view.subject)
                            .map_err(|e| e.to_string())?;
                    }
                }
                "datum.schematic.move_symbol"
            }
            B::Verb("datum.schematic.rename_label") if class == Some(C::Label) => {
                "datum.schematic.rename_label"
            }
            B::Verb(
                verb
                @ ("datum.schematic.set_symbol_reference" | "datum.schematic.set_symbol_value"),
            ) if class == Some(C::Symbol) => verb,
            B::GuiLocal("schematic.symbol_fields.open") if class == Some(C::Symbol) => {
                "datum.schematic.set_symbol_reference"
            }
            B::Verb(verb @ ("datum.schematic.place_label" | "datum.schematic.place_junction"))
                if class == Some(C::Wire) || self.menu_key == "schematic.empty" =>
            {
                verb
            }
            B::Verb(
                verb @ ("datum.schematic.place_text"
                | "datum.schematic.place_symbol"
                | "datum.schematic.draw_wire"),
            ) if self.menu_key == "schematic.empty" => verb,
            B::GuiLocal("schematic.power_port.open") if self.menu_key == "schematic.empty" => {
                "datum.schematic.place_label"
            }
            _ => return Err(item.not_built.as_deref().unwrap_or(unavailable).into()),
        };
        crate::gui_menu_model::native_edit_unavailable_reason(state, verb)
            .map_or(Ok(()), |reason| Err(reason.into()))
    }
}
