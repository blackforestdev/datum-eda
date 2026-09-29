//! Bounded accepted/rejected GPU evidence. Invalid samples never enter the accepted stream.
use anyhow::{Result, ensure};
use datum_gui_render::GpuFrameSample;
use serde_json::json;
use std::io::Write;

fn validate(sample: &GpuFrameSample, workload_required: bool) -> Result<()> {
    ensure!(
        sample.submission_manifest.len() <= 6
            && sample.raw_ticks.len() <= 32
            && sample.passes_ns.len() <= 16,
        "native GPU sample exceeds bounded record"
    );
    ensure!(
        !sample.submission_manifest.is_empty()
            && sample
                .submission_manifest
                .last()
                .is_some_and(|s| s.submission == sample.submission)
            && sample
                .submission_manifest
                .iter()
                .all(|s| s.attempt[..4].iter().all(|v| *v != 0)),
        "native GPU sample has missing submission or render-attempt lineage"
    );
    ensure!(
        !workload_required
            || sample.submission_manifest.iter().all(|s| {
                let tag = s.workload;
                tag[0] != 0
                    && tag[1] != 0
                    && tag[1] & !63 == 0
                    && (0..6).all(|phase| tag[1] & (1 << phase) == 0 || tag[phase + 2] != 0)
            }),
        "native GPU sample has missing workload demand identity"
    );
    Ok(())
}
pub(super) fn write(
    sample: &GpuFrameSample,
    workload_required: bool,
    mut file: impl Write,
) -> Result<()> {
    let checked = validate(sample, workload_required);
    let error = checked.as_ref().err().map(ToString::to_string);
    let record = json!({"host":sample.host,"device_epoch":sample.device_epoch,
        "frame":sample.frame,"submission":sample.submission,"timestamp_period_ns":sample.period_ns,
        "raw_ticks":sample.raw_ticks.iter().take(32).collect::<Vec<_>>(),"scene_marker_ticks":sample.scene_marker_ticks,
        "submission_manifest":sample.submission_manifest.iter().take(6).map(|s|json!({"submission":s.submission,"kind":s.kind,"attempt":s.attempt,"workload":s.workload,"first_tick":s.first_tick,"last_tick":s.last_tick,"transfer_first_tick":s.transfer_first_tick,"transfer_last_tick":s.transfer_last_tick,"span_ns":s.span_ns,"transfer_interval_ns":s.transfer_interval_ns})).collect::<Vec<_>>(),
        "passes_ns":sample.passes_ns.iter().take(16).collect::<Vec<_>>(),"own_pass_sum_ns":sample.own_pass_sum_ns,
        "frame_span_ns":sample.frame_span_ns,"rejection":error,
        "truncated":sample.submission_manifest.len()>6 || sample.raw_ticks.len()>32 || sample.passes_ns.len()>16});
    let prefix = if checked.is_ok() {
        "gpu_measurement"
    } else {
        "gpu_measurement_rejected"
    };
    if checked.is_err() {
        eprintln!("{prefix} {record}");
    }
    let written = writeln!(file, "{prefix} {record}");
    // Preserve the semantic rejection even if logging also fails.
    if let Err(error) = checked {
        if let Err(write_error) = written {
            return Err(error.context(format!("rejected sample log failed: {write_error}")));
        }
        return Err(error);
    }
    written?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_gpu_sample_is_preserved_before_error_without_acceptance() {
        let sample = GpuFrameSample {
            submission_manifest: vec![],
            host: 1,
            device_epoch: 2,
            frame: 3,
            submission: 4,
            period_ns: 1.,
            raw_ticks: vec![7; 33],
            scene_marker_ticks: None,
            passes_ns: vec![],
            own_pass_sum_ns: 0.,
            frame_span_ns: 0.,
        };
        let mut output = Vec::new();
        assert!(write(&sample, true, &mut output).is_err());
        let line = String::from_utf8(output).unwrap();
        assert!(line.starts_with("gpu_measurement_rejected "));
        let value: serde_json::Value =
            serde_json::from_str(line.trim().split_once(' ').unwrap().1).unwrap();
        assert_eq!(value["frame"], 3);
        assert_eq!(value["truncated"], true);
        assert_eq!(value["raw_ticks"].as_array().unwrap().len(), 32);
        assert!(write(&sample, true, std::io::sink()).is_err());
    }
}
