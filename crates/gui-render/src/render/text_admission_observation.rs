//! Opt-in counts from the same shaped buffers consumed by production drawing.
//! Scratch uses existing admission; failure invalidates observation, not rendering.
use super::*;
use crate::text_gpu::staging_vec::StagingVec;
use glyphon::CacheKey;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextAdmissionGroup {
    pub runs: usize,
    pub layout_rows: usize,
    pub shaped_instances: usize,
    pub unique_raster_keys: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextAdmissionObservation {
    pub preparation_serial: u64,
    pub font_owner_id: u64,
    pub cache_revision: u64,
    pub workspace: TextAdmissionGroup,
    pub overlay: TextAdmissionGroup,
    pub union_unique_raster_keys: usize,
    pub scratch_bytes: u64,
}
pub(crate) struct Observer {
    scope: Option<crate::cpu_alloc::Scope>,
    serial: u64,
    pub latest: Option<TextAdmissionObservation>,
    pub failed: bool,
}
impl Observer {
    pub fn new(enabled: bool) -> Self {
        Self {
            scope: enabled.then(|| crate::cpu_alloc::Scope::new("text-admission-observer")),
            serial: 0,
            latest: None,
            failed: false,
        }
    }
    pub fn serial(&self) -> u64 {
        self.serial
    }
    pub fn begin(&mut self) {
        self.latest = None;
        self.failed = false;
        if self.scope.is_some() {
            if let Some(serial) = self.serial.checked_add(1) {
                self.serial = serial;
            } else {
                self.failed = true;
            }
        }
    }
    pub fn usage(&self) -> Option<crate::cpu_alloc::Usage> {
        self.scope.as_ref().map(crate::cpu_alloc::Scope::usage)
    }
}
fn unique(keys: &mut [CacheKey]) -> usize {
    keys.sort_unstable();
    usize::from(!keys.is_empty()) + keys.windows(2).filter(|pair| pair[0] != pair[1]).count()
}
fn group(
    entries: &[text_buffer_cache::CachedTextBuffer],
    indices: &[usize],
    runs: &[TextRun],
) -> anyhow::Result<TextAdmissionGroup> {
    anyhow::ensure!(
        indices.len() == runs.len(),
        "text admission index/run mismatch"
    );
    let mut counts = TextAdmissionGroup {
        runs: runs.len(),
        ..Default::default()
    };
    for area in text_buffer_cache::build_text_areas(entries, indices, runs) {
        for row in area.rows {
            counts.layout_rows += 1;
            counts.shaped_instances = counts
                .shaped_instances
                .checked_add(row.glyphs.len())
                .ok_or_else(|| anyhow::anyhow!("text admission count overflow"))?;
        }
    }
    Ok(counts)
}
impl Renderer {
    pub(crate) fn observe_text_admission(
        &mut self,
        workspace: &[usize],
        workspace_runs: &[TextRun],
        overlay: &[usize],
        overlay_runs: &[TextRun],
    ) {
        let Some(scope) = self.text_admission.scope.clone() else {
            return;
        };
        if self.text_admission.failed {
            return;
        }
        let result = scope.with(|| -> anyhow::Result<TextAdmissionObservation> {
            let entries = self.text_buffers.entries();
            let mut a = group(entries, workspace, workspace_runs)?;
            let mut b = group(entries, overlay, overlay_runs)?;
            let count = a
                .shaped_instances
                .checked_add(b.shaped_instances)
                .ok_or_else(|| anyhow::anyhow!("text admission count overflow"))?;
            let mut keys = StagingVec::new(count, &self.atlas.staging_budget)?;
            for (indices, runs) in [(workspace, workspace_runs), (overlay, overlay_runs)] {
                for area in text_buffer_cache::build_text_areas(entries, indices, runs) {
                    for row in area.rows {
                        for glyph in row.glyphs {
                            keys.push(glyph.physical((area.left, area.top), area.scale).cache_key);
                        }
                    }
                }
            }
            a.unique_raster_keys = unique(&mut keys[..a.shaped_instances]);
            b.unique_raster_keys = unique(&mut keys[a.shaped_instances..]);
            Ok(TextAdmissionObservation {
                preparation_serial: self.text_admission.serial,
                font_owner_id: self.font_system.usage().allocation.owner_id,
                cache_revision: self.text_buffers.revision(),
                workspace: a,
                overlay: b,
                union_unique_raster_keys: unique(&mut keys),
                scratch_bytes: keys.allocated_bytes(),
            })
        });
        self.text_admission.failed = result.is_err();
        self.text_admission.latest = result.ok();
    }
    pub fn text_admission_observation(&self) -> Option<TextAdmissionObservation> {
        self.text_admission.latest
    }
    pub fn text_admission_observer_usage(&self) -> Option<crate::cpu_alloc::Usage> {
        self.text_admission.usage()
    }
    pub fn text_admission_observation_failed(&self) -> bool {
        self.text_admission.failed
    }
}
