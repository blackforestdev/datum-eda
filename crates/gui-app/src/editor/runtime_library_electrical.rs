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
            let is_graphic = graphic.is_some();
            let result = if let Some(result) = graphic {
                result
            } else if pin {
                d.switch_pin(index, control == C::RemovePin)
            } else {
                d.switch_anchor(index, control == C::RemoveAnchor)
            };
            match result {
                Ok(()) => {
                    let key = if is_graphic {
                        d.electrical.graphics.focus_key()
                    } else if pin {
                        "pin_name"
                    } else if d.create
                        || d.electrical.new_anchor
                        || d.electrical.anchor_id.is_some()
                    {
                        "anchor_pin"
                    } else {
                        "anchor_x"
                    };
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
