//! Local path/OSC7 decoding shared by inheritance and host-path protection.
//! This owner performs no filesystem access or fallback policy.
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

pub(crate) fn decode_local_path(value: &str) -> Option<PathBuf> {
    if value.starts_with('/') {
        // A plain path's percent/query characters are literal filename bytes.
        return (!value.contains('\0')).then(|| PathBuf::from(value));
    }
    let uri = value.strip_prefix("file://")?;
    let slash = uri.find('/')?;
    let host = &uri[..slash];
    if !host.is_empty()
        && !host.eq_ignore_ascii_case("localhost")
        && !local_hostname().is_some_and(|local| host.eq_ignore_ascii_case(local))
    {
        return None;
    }
    let encoded = &uri[slash..];
    if encoded.contains(['?', '#']) {
        return None;
    }
    let bytes = percent_decode_path(encoded.as_bytes())?;
    let path = PathBuf::from(OsString::from_vec(bytes));
    path.is_absolute().then_some(path)
}

fn local_hostname() -> Option<&'static str> {
    static HOSTNAME: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    HOSTNAME
        .get_or_init(|| {
            let mut name = std::mem::MaybeUninit::<libc::utsname>::zeroed();
            if unsafe { libc::uname(name.as_mut_ptr()) } != 0 {
                return None;
            }
            let name = unsafe { name.assume_init() };
            unsafe { std::ffi::CStr::from_ptr(name.nodename.as_ptr()) }
                .to_str()
                .ok()
                .map(str::to_owned)
        })
        .as_deref()
}

fn percent_decode_path(encoded: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut index = 0;
    while index < encoded.len() {
        if encoded[index] != b'%' {
            if encoded[index] == 0 {
                return None;
            }
            decoded.push(encoded[index]);
            index += 1;
            continue;
        }
        let high = *encoded.get(index + 1)?;
        let low = *encoded.get(index + 2)?;
        let byte = hex_value(high)?
            .checked_mul(16)?
            .checked_add(hex_value(low)?)?;
        if byte == 0 {
            return None;
        }
        decoded.push(byte);
        index += 3;
    }
    Some(decoded)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_local_path_preserves_plain_percent_and_decodes_only_local_uri() {
        if let Some(host) = local_hostname() {
            assert_eq!(
                decode_local_path(&format!("file://{host}/tmp/own%20cwd")),
                Some(PathBuf::from("/tmp/own cwd"))
            );
            assert_eq!(
                decode_local_path(&format!("file://{host}.spoof.invalid/tmp/a")),
                None
            );
        }
        assert_eq!(
            decode_local_path("/tmp/100% literal%20name?#"),
            Some(PathBuf::from("/tmp/100% literal%20name?#"))
        );
        assert_eq!(
            decode_local_path("file:///tmp/exact%20name"),
            Some(PathBuf::from("/tmp/exact name"))
        );
        assert_eq!(
            decode_local_path("file://localhost/tmp/a%25b"),
            Some(PathBuf::from("/tmp/a%b"))
        );
        for uri in [
            "file://remote/tmp/a",
            "file:///tmp/%00",
            "file:///tmp/%xy",
            "file:///tmp/a?query",
            "relative/a",
        ] {
            assert_eq!(decode_local_path(uri), None, "{uri}");
        }
    }
}
