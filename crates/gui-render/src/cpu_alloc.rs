//! Scoped Rust-heap ownership for library allocations with private capacities.
//! Native malloc/mappings and allocator-internal slack remain separate from Rust
//! requested layout bytes. No dependency layout or private field is inspected.
#[path = "private_text_call.rs"]
pub mod calls;
#[path = "heap_accounting.rs"]
pub mod heap;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

thread_local! {
    static CURRENT: Cell<*const State> = const { Cell::new(ptr::null()) };
    static OWNER_METADATA: Cell<bool> = const { Cell::new(false) };
}
static OWNER_METADATA_BYTES: AtomicU64 = AtomicU64::new(0);
static INSTALLED: AtomicBool = AtomicBool::new(false);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static OWNERS: Mutex<Vec<Weak<State>>> = Mutex::new(Vec::new());

struct State {
    id: u64,
    label: &'static str,
    payload: AtomicU64,
    overhead: AtomicU64,
    allocations: AtomicU64,
    peak_payload: AtomicU64,
    outputs: AtomicU64,
    output_lifetimes: Mutex<()>,
}

#[derive(Clone, Debug)]
pub struct Usage {
    pub allocator_installed: bool,
    pub owner_id: u64,
    pub label: &'static str,
    pub payload_bytes: u64,
    pub tracking_bytes: u64,
    pub allocations: u64,
    pub peak_payload_bytes: u64,
    /// Scope's own Arc/header allocation, counted once per owner, not per block.
    pub owner_metadata_bytes: u64,
}

/// Scopes are synchronous and nest by call stack; no raw guard can escape or be
/// dropped out of order. Each allocated block holds its owner until deallocation.
#[derive(Clone)]
pub struct Scope(Arc<State>);
impl Scope {
    pub fn new(label: &'static str) -> Self {
        with_current(ptr::null(), || {
            OWNER_METADATA.with(|capture| capture.set(true));
            let state = Arc::new(State {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                label,
                payload: AtomicU64::new(0),
                overhead: AtomicU64::new(0),
                allocations: AtomicU64::new(0),
                peak_payload: AtomicU64::new(0),
                outputs: AtomicU64::new(0),
                output_lifetimes: Mutex::new(()),
            });
            OWNER_METADATA.with(|capture| capture.set(false));
            let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
            owners.retain(|owner| owner.strong_count() != 0);
            owners.push(Arc::downgrade(&state));
            Self(state)
        })
    }
    pub fn with<T>(&self, work: impl FnOnce() -> T) -> T {
        struct RawOwner(*const State);
        impl Drop for RawOwner {
            fn drop(&mut self) {
                // SAFETY: this guard owns the reference created by into_raw below.
                unsafe {
                    drop(Arc::from_raw(self.0));
                }
            }
        }
        let owner = RawOwner(Arc::into_raw(self.0.clone()));
        with_current(owner.0, work)
    }
    pub(crate) fn output_bytes(&self) -> u64 {
        self.0.outputs.load(Ordering::Acquire)
    }
    pub(crate) fn add_output(&self, bytes: u64) {
        self.0.outputs.fetch_add(bytes, Ordering::AcqRel);
    }
    pub(crate) fn remove_output(&self, bytes: u64) {
        self.0.outputs.fetch_sub(bytes, Ordering::AcqRel);
    }
    pub(crate) fn output_lifetimes(&self) -> std::sync::MutexGuard<'_, ()> {
        self.0
            .output_lifetimes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }
    pub fn usage(&self) -> Usage {
        self.0.usage()
    }
}
impl State {
    fn usage(&self) -> Usage {
        Usage {
            allocator_installed: installed(),
            owner_id: self.id,
            label: self.label,
            payload_bytes: self.payload.load(Ordering::Acquire),
            tracking_bytes: self.overhead.load(Ordering::Acquire),
            allocations: self.allocations.load(Ordering::Acquire),
            peak_payload_bytes: self.peak_payload.load(Ordering::Acquire),
            owner_metadata_bytes: OWNER_METADATA_BYTES.load(Ordering::Acquire),
        }
    }
    fn add_payload(&self, bytes: usize) {
        let live = self
            .payload
            .fetch_add(bytes as u64, Ordering::AcqRel)
            .saturating_add(bytes as u64);
        self.peak_payload.fetch_max(live, Ordering::Relaxed);
    }
}
fn with_current<T>(owner: *const State, work: impl FnOnce() -> T) -> T {
    struct Restore(*const State);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = CURRENT.try_with(|current| current.set(self.0));
        }
    }
    let previous = CURRENT
        .try_with(|current| current.replace(owner))
        .unwrap_or(ptr::null());
    let _restore = Restore(previous);
    work()
}

/// Keep diagnostic storage outside the measured payload scope. Its capacity and
/// allocator overhead still belong to the separately reported observer footprint.
pub(crate) fn observation_storage<T>(work: impl FnOnce() -> T) -> T {
    with_current(ptr::null(), work)
}

pub fn installed() -> bool {
    INSTALLED.load(Ordering::Acquire)
}

/// Observations allocate outside tracked scopes and do not retain payloads.
/// Concurrent fields are sampled counters, not a transaction across all threads.
pub fn usage() -> Vec<Usage> {
    with_current(ptr::null(), || {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        let mut result = Vec::new();
        owners.retain(|owner| {
            if let Some(owner) = owner.upgrade() {
                result.push(owner.usage());
                true
            } else {
                false
            }
        });
        if owners.capacity() > owners.len().saturating_mul(4) {
            *owners = std::mem::take(&mut *owners).into_boxed_slice().into_vec();
        }
        result
    })
}

#[repr(C)]
struct Header {
    owner: *const State,
}
fn extended(layout: Layout) -> Option<(Layout, usize)> {
    Layout::new::<Header>().extend(layout).ok()
}

/// Install once at the executable boundary. Unscoped allocations carry only a
/// null owner header; tracked blocks keep attribution through realloc/free on any
/// thread. Header alignment/padding is reported separately from requested payload.
pub struct Allocator;
// SAFETY: every successful allocation uses the extended System layout and a
// matching prefix. Deallocation/reallocation reconstruct that same layout from
// GlobalAlloc's caller-supplied original layout. Each non-null header owns one
// Arc strong reference, transferred on realloc and consumed exactly once on free.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { allocate(layout, false) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { allocate(layout, true) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        let Some((combined, offset)) = extended(layout) else {
            return;
        };
        // SAFETY: pointer/layout came from this allocator; prefix is initialized.
        unsafe {
            let base = pointer.sub(offset);
            let owner = (*base.cast::<Header>()).owner;
            System.dealloc(base, combined);
            if !owner.is_null() {
                let owner = Arc::from_raw(owner);
                calls::transaction(|ledger| {
                    owner
                        .payload
                        .fetch_sub(layout.size() as u64, Ordering::AcqRel);
                    owner.overhead.fetch_sub(offset as u64, Ordering::AcqRel);
                    owner.allocations.fetch_sub(1, Ordering::AcqRel);
                    ledger.allocation(owner.id, 0, (layout.size() + offset) as u64);
                });
            }
        }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Some((combined, offset)) = extended(layout) else {
            return ptr::null_mut();
        };
        let Ok(next) = Layout::from_size_align(new_size, layout.align()) else {
            return ptr::null_mut();
        };
        let Some((next, next_offset)) = extended(next) else {
            return ptr::null_mut();
        };
        let _ = next_offset; // Same payload alignment gives the same prefix offset.
        // SAFETY: realloc preserves the prefix, alignment and its one Arc owner.
        // Failure leaves the original block and accounting untouched.
        unsafe {
            let base = pointer.sub(offset);
            let owner = (*base.cast::<Header>()).owner;
            let resized = System.realloc(base, combined, next.size());
            if resized.is_null() {
                return resized;
            }
            if !owner.is_null() {
                calls::transaction(|ledger| {
                    if new_size >= layout.size() {
                        (*owner).add_payload(new_size - layout.size());
                    } else {
                        (*owner)
                            .payload
                            .fetch_sub((layout.size() - new_size) as u64, Ordering::AcqRel);
                    }
                    ledger.allocation((*owner).id, new_size as u64, layout.size() as u64);
                });
            }
            resized.add(offset)
        }
    }
}
unsafe fn allocate(layout: Layout, zeroed: bool) -> *mut u8 {
    if !INSTALLED.load(Ordering::Relaxed) {
        INSTALLED.store(true, Ordering::Release);
    }
    let Some((combined, offset)) = extended(layout) else {
        return ptr::null_mut();
    };
    // SAFETY: both System entry points receive a valid layout. The payload starts
    // at Layout::extend's aligned offset; zero initialization covers the payload.
    unsafe {
        let base = if zeroed {
            System.alloc_zeroed(combined)
        } else {
            System.alloc(combined)
        };
        if base.is_null() {
            return base;
        }
        if OWNER_METADATA.try_with(Cell::get).unwrap_or(false) {
            OWNER_METADATA_BYTES.store(combined.size() as u64, Ordering::Release);
        }
        let owner = CURRENT.try_with(Cell::get).unwrap_or(ptr::null());
        if !owner.is_null() {
            Arc::increment_strong_count(owner);
            calls::transaction(|ledger| {
                (*owner).add_payload(layout.size());
                (*owner).overhead.fetch_add(offset as u64, Ordering::AcqRel);
                (*owner).allocations.fetch_add(1, Ordering::AcqRel);
                ledger.allocation((*owner).id, (layout.size() + offset) as u64, 0);
            });
        }
        base.cast::<Header>().write(Header { owner });
        base.add(offset)
    }
}

impl crate::Renderer {
    /// Rust allocations attributed to synchronous text preparation. Font ownership
    /// is reported by font_cpu_usage; nested layout/raster scopes by usage().
    /// Public cache-capacity reports overlap these scopes; do not sum the two views.
    pub fn text_cpu_usage(&self) -> Usage {
        self.text_cpu.usage()
    }
}

/// Shared coordination storage is distinct from scoped payload and reported
/// once per process. The call ledger is static, with no construction-time heap.
#[derive(Debug)]
pub struct RegistryUsage {
    pub registry_heap_bytes: usize,
    /// All live and weak-retained scope Arc allocations, counted once here.
    /// Usage::owner_metadata_bytes describes a subset, not an additional total.
    pub scope_owner_bytes: u64,
    pub call_slots_static_bytes: usize,
}
pub fn registry_metadata_bytes() -> RegistryUsage {
    with_current(ptr::null(), || {
        let owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        RegistryUsage {
            registry_heap_bytes: heap::capacity_bytes::<Weak<State>>(owners.capacity()),
            scope_owner_bytes: owners.len() as u64 * OWNER_METADATA_BYTES.load(Ordering::Acquire),
            call_slots_static_bytes: calls::registry_metadata_bytes(),
        }
    })
}

#[cfg(test)]
#[path = "cpu_alloc_tests.rs"]
mod tests;
