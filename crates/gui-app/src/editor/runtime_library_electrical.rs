//! Complete electrical row dispatch, refusal and focus over the shared form.
use super::*;
impl Runtime {
    pub(super) fn activate_electrical(&mut self, control: C) {
        let browser = &mut self.session.workspace_mut().library;
        if let Some(d) = &mut browser.draft {
            let pin = matches!(control, C::Pin(_) | C::AddPin | C::RemovePin);
            let index = match control {
                C::Pin(i) | C::Anchor(i) => Some(i),
                _ => None,
            };
            let graphic = d.switch_graphic_control(&control);
            let result = if let Some(result) = d.switch_symbol_field_control(&control) {
                result
            } else if let Some(result) = graphic {
                result.map(|()| d.electrical.graphics.focus_key())
            } else if pin {
                d.switch_pin(index, control == C::RemovePin)
                    .map(|()| "pin_name")
            } else {
                d.switch_anchor(index, control == C::RemoveAnchor)
                    .map(|()| {
                        if d.create || d.electrical.new_anchor || d.electrical.anchor_id.is_some() {
                            "anchor_pin"
                        } else {
                            "anchor_x"
                        }
                    })
            };
            match result {
                Ok(key) => {
                    if let Some(focused) = d.fields.iter().position(|f| f.key == key) {
                        d.focused = focused;
                        d.offset = focused;
                        browser.focus = Some(C::Field(focused));
                    }
                }
                Err(error) => d.refusal = Some(error),
            }
        }
    }
}
