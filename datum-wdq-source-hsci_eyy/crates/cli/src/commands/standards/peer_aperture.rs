use std::collections::BTreeMap;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPad;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PadProcessAperturePolicy {
    mask_layers: Vec<i32>,
    paste_layers: Vec<i32>,
    solder_mask_margin_nm: i64,
    solder_paste_margin_nm: i64,
    solder_paste_margin_ratio_ppm: i32,
}

impl PadProcessAperturePolicy {
    fn from_pad(pad: &PlacedPad) -> Self {
        let mut mask_layers = pad.mask_layers.clone();
        mask_layers.sort();
        let mut paste_layers = pad.paste_layers.clone();
        paste_layers.sort();
        Self {
            mask_layers,
            paste_layers,
            solder_mask_margin_nm: pad.solder_mask_margin_nm,
            solder_paste_margin_nm: pad.solder_paste_margin_nm,
            solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
        }
    }

    fn apply_to_pad(&self, pad: &mut PlacedPad) -> bool {
        let changed = PadProcessAperturePolicy::from_pad(pad) != *self;
        pad.mask_layers = self.mask_layers.clone();
        pad.paste_layers = self.paste_layers.clone();
        pad.solder_mask_margin_nm = self.solder_mask_margin_nm;
        pad.solder_paste_margin_nm = self.solder_paste_margin_nm;
        pad.solder_paste_margin_ratio_ppm = self.solder_paste_margin_ratio_ppm;
        changed
    }
}

pub(super) fn apply_unique_peer_process_aperture_policy(
    project: &super::LoadedNativeProject,
    target_pad: &mut PlacedPad,
) -> Result<bool> {
    let mut policy_counts = BTreeMap::<PadProcessAperturePolicy, usize>::new();
    for pad_value in project.board.pads.values().cloned() {
        let pad: PlacedPad =
            serde_json::from_value(pad_value).context("failed to parse peer pad")?;
        if pad.package != target_pad.package {
            continue;
        }
        *policy_counts
            .entry(PadProcessAperturePolicy::from_pad(&pad))
            .or_default() += 1;
    }
    if policy_counts.len() < 2 {
        return Ok(false);
    }
    let mut policies = policy_counts.into_iter().collect::<Vec<_>>();
    policies.sort_by(|(left_policy, left_count), (right_policy, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_policy.cmp(right_policy))
    });
    if policies.len() > 1 && policies[0].1 == policies[1].1 {
        return Ok(false);
    }
    let expected = policies.remove(0).0;
    if expected == PadProcessAperturePolicy::from_pad(target_pad) {
        return Ok(false);
    }
    Ok(expected.apply_to_pad(target_pad))
}
