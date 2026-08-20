use crate::{
    InputDisposition, InputError, KeyModifiers, MouseEncoding, MouseTracking, TerminalCore,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseAction {
    Press(MouseButton),
    Release(MouseButton),
    Move(Option<MouseButton>),
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MousePosition {
    pub column: i64,
    pub row: i64,
    pub pixel_x: i64,
    pub pixel_y: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MouseInput {
    pub action: MouseAction,
    pub position: MousePosition,
    pub modifiers: KeyModifiers,
    pub local_override: bool,
}

impl TerminalCore {
    pub fn encode_mouse(&self, input: MouseInput) -> Result<InputDisposition, InputError> {
        if input.local_override {
            return Ok(InputDisposition::LocalOnly);
        }
        let tracking = self.state.mouse_tracking;
        if !tracked(tracking, input.action) {
            return Ok(InputDisposition::Ignored);
        }
        self.input_disposition(Some(encode(self, input)))
    }
}

fn tracked(tracking: MouseTracking, action: MouseAction) -> bool {
    match tracking {
        MouseTracking::Off => false,
        MouseTracking::X10 => matches!(action, MouseAction::Press(_)),
        MouseTracking::Button => !matches!(action, MouseAction::Move(_)),
        MouseTracking::Drag => !matches!(action, MouseAction::Move(None)),
        MouseTracking::Any => true,
    }
}

fn encode(core: &TerminalCore, input: MouseInput) -> Vec<u8> {
    let (code, release) = button_code(input.action, input.modifiers);
    let legacy_code = if release {
        3 + 4 * u32::from(input.modifiers.shift)
            + 8 * u32::from(input.modifiers.alt)
            + 16 * u32::from(input.modifiers.control)
    } else {
        code
    };
    let size = core.state.size;
    let cell_x = clip(input.position.column, size.columns.get()) + 1;
    let cell_y = clip(input.position.row, size.rows.get()) + 1;
    match core.state.mouse_encoding {
        MouseEncoding::Sgr => sgr(code, cell_x, cell_y, release),
        MouseEncoding::SgrPixels => {
            let pixel_x = clip_pixels(input.position.pixel_x, size.pixels.width);
            let pixel_y = clip_pixels(input.position.pixel_y, size.pixels.height);
            sgr(code, pixel_x, pixel_y, release)
        }
        MouseEncoding::Urxvt => {
            format!("\x1b[{};{cell_x};{cell_y}M", legacy_code + 32).into_bytes()
        }
        MouseEncoding::Utf8 => {
            let mut bytes = b"\x1b[M".to_vec();
            for value in [legacy_code + 32, cell_x + 32, cell_y + 32] {
                bytes.extend(
                    char::from_u32(value)
                        .unwrap_or('\u{fffd}')
                        .to_string()
                        .as_bytes(),
                );
            }
            bytes
        }
        MouseEncoding::Default => {
            let values = [legacy_code + 32, cell_x.min(223) + 32, cell_y.min(223) + 32];
            let mut bytes = b"\x1b[M".to_vec();
            bytes.extend(values.map(|value| value as u8));
            bytes
        }
    }
}

fn clip(value: i64, maximum: u16) -> u32 {
    value.clamp(0, i64::from(maximum.saturating_sub(1))) as u32
}

fn clip_pixels(value: i64, maximum: u32) -> u32 {
    if maximum == 0 {
        1
    } else {
        value.clamp(0, i64::from(maximum.saturating_sub(1))) as u32 + 1
    }
}

fn sgr(code: u32, x: u32, y: u32, release: bool) -> Vec<u8> {
    format!("\x1b[<{code};{x};{y}{}", if release { 'm' } else { 'M' }).into_bytes()
}

fn button_code(action: MouseAction, modifiers: KeyModifiers) -> (u32, bool) {
    let mut code = match action {
        MouseAction::Press(button) | MouseAction::Release(button) => button_index(button),
        MouseAction::Move(button) => button.map_or(3, button_index) + 32,
        MouseAction::WheelUp => 64,
        MouseAction::WheelDown => 65,
        MouseAction::WheelLeft => 66,
        MouseAction::WheelRight => 67,
    };
    code += 4 * u32::from(modifiers.shift)
        + 8 * u32::from(modifiers.alt)
        + 16 * u32::from(modifiers.control);
    (code, matches!(action, MouseAction::Release(_)))
}

const fn button_index(button: MouseButton) -> u32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}
