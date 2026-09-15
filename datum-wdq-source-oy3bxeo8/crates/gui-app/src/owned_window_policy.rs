//! Shared focus and stacking policy for application-owned native windows.
//!
//! Datum establishes native ownership where the backend exposes it and keeps
//! focus redirection as a defensive fallback. Owned settings windows are never
//! made globally always-on-top.

use anyhow::Result;
use std::sync::Arc;
use winit::window::{UserAttentionType, Window};

#[cfg(target_os = "linux")]
use anyhow::bail;
#[cfg(target_os = "linux")]
use std::ffi::{CStr, c_char, c_void};
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowExtWayland;

#[cfg(target_os = "linux")]
const XDG_TOPLEVEL_SET_PARENT: u32 = 1;

/// Establish the compositor-level parent/child relationship for an owned
/// settings window.
///
/// Winit exposes each Wayland `xdg_toplevel` but does not wrap
/// `xdg_toplevel.set_parent`. Datum uses that exposed platform object and the
/// already-loaded Wayland client ABI to issue the standard request. No library
/// is loaded here: `RTLD_NOLOAD` makes absence a hard error rather than silently
/// adding a runtime dependency.
pub(super) fn establish_native_owner(owner: &Window, owned: &Window) -> Result<()> {
    #[cfg(target_os = "linux")]
    if let (Some(owner), Some(owned)) = (owner.xdg_toplevel(), owned.xdg_toplevel()) {
        wayland::set_xdg_toplevel_parent(owned.as_ptr(), owner.as_ptr())?;
    }
    Ok(())
}

pub(super) const fn owner_activation_raises_owned_window(
    owner_focused: bool,
    owned_window_open: bool,
) -> bool {
    owner_focused && owned_window_open
}

pub(super) fn redirect_owner_activation(
    owner_focused: bool,
    owned_window: Option<&Arc<Window>>,
) -> bool {
    if !owner_activation_raises_owned_window(owner_focused, owned_window.is_some()) {
        return false;
    }
    let window = owned_window.expect("open owned window checked above");
    raise_owned_window(window);
    true
}

pub(super) fn show_owned_window(window: &Window) {
    window.set_visible(true);
    window.focus_window();
    window.request_redraw();
}

pub(super) fn raise_owned_window(window: &Window) {
    show_owned_window(window);
    window.request_user_attention(Some(UserAttentionType::Informational));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_owner_activation_with_an_open_owned_window_redirects_focus() {
        assert!(owner_activation_raises_owned_window(true, true));
        assert!(!owner_activation_raises_owned_window(false, true));
        assert!(!owner_activation_raises_owned_window(true, false));
        assert!(!owner_activation_raises_owned_window(false, false));
    }
}

#[cfg(target_os = "linux")]
mod wayland {
    use super::*;

    type WlProxyGetVersion = unsafe extern "C" fn(*mut c_void) -> u32;
    type WlProxyMarshalFlags =
        unsafe extern "C" fn(*mut c_void, u32, *const c_void, u32, u32, ...) -> *mut c_void;

    struct LoadedWaylandClient(*mut c_void);

    impl LoadedWaylandClient {
        fn existing() -> Result<Self> {
            let name = c"libwayland-client.so.0";
            // SAFETY: `name` is a static NUL-terminated soname. `RTLD_NOLOAD`
            // restricts this lookup to the library Winit's active Wayland
            // backend has already loaded.
            let handle = unsafe {
                libc::dlopen(
                    name.as_ptr(),
                    libc::RTLD_LAZY | libc::RTLD_LOCAL | libc::RTLD_NOLOAD,
                )
            };
            if handle.is_null() {
                bail!("Wayland client ABI is unavailable: {}", dlerror_message());
            }
            Ok(Self(handle))
        }

        fn symbol(&self, name: &CStr) -> Result<*mut c_void> {
            // SAFETY: `self.0` is a live `dlopen` handle and `name` is
            // NUL-terminated. The result is checked before conversion.
            let symbol = unsafe { libc::dlsym(self.0, name.as_ptr()) };
            if symbol.is_null() {
                bail!(
                    "Wayland client symbol {} is unavailable: {}",
                    name.to_string_lossy(),
                    dlerror_message()
                );
            }
            Ok(symbol)
        }
    }

    impl Drop for LoadedWaylandClient {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful `dlopen` call and is
            // closed exactly once here, after the synchronous protocol call.
            unsafe {
                libc::dlclose(self.0);
            }
        }
    }

    pub(super) fn set_xdg_toplevel_parent(owned: *mut c_void, owner: *mut c_void) -> Result<()> {
        let library = LoadedWaylandClient::existing()?;
        let get_version_symbol = library.symbol(c"wl_proxy_get_version")?;
        let marshal_flags_symbol = library.symbol(c"wl_proxy_marshal_flags")?;

        // SAFETY: the symbols are resolved by their stable Wayland client ABI
        // names and converted to their published function signatures.
        let get_version: WlProxyGetVersion = unsafe { std::mem::transmute(get_version_symbol) };
        // SAFETY: see above; this is the published variadic marshal signature.
        let marshal_flags: WlProxyMarshalFlags =
            unsafe { std::mem::transmute(marshal_flags_symbol) };
        // SAFETY: Winit supplied both live `xdg_toplevel` proxies from the same
        // active Wayland connection. Opcode 1 is `xdg_toplevel.set_parent` in
        // the stable xdg-shell protocol and its only argument is the parent
        // `xdg_toplevel` proxy.
        unsafe {
            marshal_flags(
                owned,
                XDG_TOPLEVEL_SET_PARENT,
                ptr::null(),
                get_version(owned),
                0,
                owner,
            );
        }
        Ok(())
    }

    fn dlerror_message() -> String {
        // SAFETY: `dlerror` returns either null or a NUL-terminated diagnostic
        // owned by the dynamic loader and valid until its next loader call.
        let error = unsafe { libc::dlerror() };
        if error.is_null() {
            return "unknown dynamic-loader error".to_string();
        }
        // SAFETY: non-null `dlerror` results are valid C strings by contract.
        unsafe { CStr::from_ptr(error.cast::<c_char>()) }
            .to_string_lossy()
            .into_owned()
    }
}
