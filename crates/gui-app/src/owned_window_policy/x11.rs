//! Establish transient ownership on Winit's existing Xlib connection.

use anyhow::{Result, bail, ensure};
use std::ffi::{c_int, c_ulong, c_void};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::window::Window;

type SetTransientForHint = unsafe extern "C" fn(*mut c_void, c_ulong, c_ulong) -> c_int;
type Flush = unsafe extern "C" fn(*mut c_void) -> c_int;

struct LoadedXlib(*mut c_void);

impl Drop for LoadedXlib {
    fn drop(&mut self) {
        // SAFETY: this handle came from a successful dlopen and is closed once,
        // after all synchronous calls through its symbols have completed.
        unsafe { libc::dlclose(self.0) };
    }
}

pub(super) fn establish_owner(owner: &Window, owned: &Window) -> Result<()> {
    let RawWindowHandle::Xlib(owned_handle) = owned.window_handle()?.as_raw() else {
        return Ok(());
    };
    let RawWindowHandle::Xlib(owner_handle) = owner.window_handle()?.as_raw() else {
        bail!("X11 owned window requires an X11 owner");
    };
    let (RawDisplayHandle::Xlib(display), RawDisplayHandle::Xlib(owner_display)) = (
        owned.display_handle()?.as_raw(),
        owner.display_handle()?.as_raw(),
    ) else {
        bail!("X11 ownership requires Winit's Xlib display");
    };
    ensure!(
        display.display == owner_display.display,
        "X11 owner display mismatch"
    );
    let display = display
        .display
        .ok_or_else(|| anyhow::anyhow!("Xlib display unavailable"))?;
    // SAFETY: the static soname is NUL-terminated. RTLD_NOLOAD permits only
    // the Xlib already loaded by Winit, never a new runtime dependency.
    let library = unsafe {
        libc::dlopen(
            c"libX11.so.6".as_ptr(),
            libc::RTLD_LAZY | libc::RTLD_LOCAL | libc::RTLD_NOLOAD,
        )
    };
    ensure!(!library.is_null(), "Winit's loaded Xlib ABI is unavailable");
    let library = LoadedXlib(library);
    // SAFETY: both symbol names are NUL-terminated and the library is live.
    let set_hint = unsafe { libc::dlsym(library.0, c"XSetTransientForHint".as_ptr()) };
    // SAFETY: same live-library lookup as above.
    let flush = unsafe { libc::dlsym(library.0, c"XFlush".as_ptr()) };
    ensure!(
        !set_hint.is_null() && !flush.is_null(),
        "Xlib ownership symbols unavailable"
    );
    // SAFETY: these are the published Xlib signatures, including XID's unsigned
    // long width. Symbols were checked before conversion.
    let set_hint: SetTransientForHint = unsafe { std::mem::transmute(set_hint) };
    // SAFETY: XFlush takes the live Display pointer and returns int.
    let flush: Flush = unsafe { std::mem::transmute(flush) };
    // SAFETY: Winit owns both live windows and their shared Display throughout
    // this synchronous event-thread call. The hint does not reparent or embed
    // the window; it identifies its transient owner to the window manager.
    unsafe {
        ensure!(
            set_hint(display.as_ptr(), owned_handle.window, owner_handle.window) != 0,
            "X11 transient ownership request failed"
        );
        flush(display.as_ptr());
    }
    Ok(())
}
