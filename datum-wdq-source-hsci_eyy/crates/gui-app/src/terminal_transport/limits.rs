//! Owner-ratified DTC-P04 transport resource limits.

pub(crate) const MAX_OUTPUT_CHUNKS: usize = 256;
pub(crate) const MAX_OUTPUT_CHUNK_BYTES: usize = 16 * 1024;
pub(crate) const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

pub(crate) const MAX_INPUT_REQUESTS: usize = 64;
pub(crate) const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;

pub(crate) const MAX_LIVE_SESSIONS: usize = 16;
pub(crate) const GUI_DRAIN_EVENT_LIMIT: usize = 128;
pub(crate) const GUI_DRAIN_BYTE_LIMIT: usize = 64 * 1024;

pub(crate) const HUP_GRACE_MS: u64 = 2_000;
pub(crate) const TERM_GRACE_MS: u64 = 2_000;
pub(crate) const KILL_VERIFY_MS: u64 = 2_000;
pub(crate) const GLOBAL_SHUTDOWN_MS: u64 = 6_000;
pub(crate) const MAX_SESSION_MEMBERS: usize = 4_096;
pub(crate) const MAX_SESSION_GROUPS: usize = 4_096;

const _: () = assert!(MAX_OUTPUT_CHUNKS * MAX_OUTPUT_CHUNK_BYTES == MAX_OUTPUT_BYTES);
