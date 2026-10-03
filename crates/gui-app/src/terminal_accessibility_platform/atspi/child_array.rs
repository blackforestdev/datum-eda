//! Exact preflight for complete read-only child arrays, shared by selection and menus.
use super::*;
use crate::terminal_accessibility_platform::dbus::MAX_MESSAGE_BYTES;
fn align(n: usize, alignment: usize) -> Option<usize> {
    n.checked_add(alignment - 1).map(|n| n & !(alignment - 1))
}
pub(super) fn preflight(
    paths: impl Iterator<Item = String>,
    bus: &str,
    header: usize,
    limit: usize,
) -> Option<usize> {
    let mut bytes = 8usize; // array length followed by struct alignment
    for path in paths {
        bytes = align(bytes, 8)?.checked_add(5)?.checked_add(bus.len())?;
        bytes = align(bytes, 4)?.checked_add(5)?.checked_add(path.len())?;
        if header.checked_add(bytes)? > limit {
            return None;
        }
    }
    (header.checked_add(bytes)? <= limit).then_some(bytes)
}
pub(super) fn reply<'a>(
    state: &ServiceState,
    call: &Message,
    make_paths: impl Fn() -> Box<dyn Iterator<Item = String> + 'a>,
) -> DispatchResult {
    let header = Message::method_return(
        1,
        call.serial,
        call.header.sender.as_deref(),
        "a(so)",
        Vec::new(),
    )
    .encode()
    .map_err(|_| limits())?
    .len();
    preflight(make_paths(), &state.bus_name, header, MAX_MESSAGE_BYTES).ok_or_else(limits)?;
    let mut body = BodyWriter::new();
    body.array(8, |body| {
        for path in make_paths() {
            body.structure(|body| {
                body.string(&state.bus_name);
                body.object_path(&path);
            });
        }
    });
    Ok(("a(so)", body.finish()))
}
fn limits() -> (&'static str, &'static str) {
    (
        "org.freedesktop.DBus.Error.LimitsExceeded",
        "Complete child array exceeds the existing D-Bus frame limit; use ChildCount and GetChildAtIndex. Membership is not truncated.",
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_accessibility_children_preflight_matches_exact_encoded_layout() {
        for bus in ["", ":1.7", ":1.12345"] {
            for paths in [
                Vec::new(),
                vec!["/short".into(), "/longer/qualified_identity".into()],
            ] {
                let body = object_array_body(bus, &paths);
                let size = preflight(paths.clone().into_iter(), bus, 72, usize::MAX).unwrap();
                assert_eq!(size, body.len());
                assert_eq!(
                    preflight(paths.clone().into_iter(), bus, 72, 72 + size),
                    Some(size)
                );
                assert_eq!(preflight(paths.into_iter(), bus, 72, 71 + size), None);
            }
        }
    }
}
