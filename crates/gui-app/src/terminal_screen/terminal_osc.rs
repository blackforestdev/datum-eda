use datum_gui_protocol::TerminalLaneState;

pub(super) fn apply_osc_payload(payload: &[u8], state: &mut TerminalLaneState) {
    let payload = String::from_utf8_lossy(payload);
    let Some((command, value)) = payload.split_once(';') else {
        return;
    };
    match command {
        "0" | "1" | "2" => state.title = Some(value.to_string()),
        "7" => {
            if let Some(path) = osc7_file_uri_path(value) {
                state.current_working_directory = Some(path);
            }
        }
        _ => {}
    }
}

fn osc7_file_uri_path(value: &str) -> Option<String> {
    let rest = value.strip_prefix("file://")?;
    let path_start = rest.find('/')?;
    let path = &rest[path_start..];
    if path.is_empty() {
        return None;
    }
    Some(percent_decode(path))
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            out.push((high << 4) | low);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
