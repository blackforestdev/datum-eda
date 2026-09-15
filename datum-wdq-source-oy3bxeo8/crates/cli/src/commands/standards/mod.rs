// commands/standards — standards-repair proposal generation (copper
// clearance, silkscreen clearance, unique peer process-aperture policy).
//
// Wave 2 move. Files came from the legacy command_project_surface.rs host;
// the re-export below reproduces exactly what that host exported for this
// family (the repair satellites are family-internal, consumed by repairs.rs).

#[allow(unused_imports)]
use super::*;

mod clearance_repairs;
mod peer_aperture;
mod repairs;
mod silk_repairs;

pub(crate) use self::repairs::generate_native_project_standards_repair_proposals;
