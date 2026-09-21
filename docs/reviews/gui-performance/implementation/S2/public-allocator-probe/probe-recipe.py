from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/gpu_uniform_tests.rs');old=p.read_bytes()
probe=r'''
#[test]
#[ignore = "temporary public allocator report probe"]
fn pm045_public_allocator_report_probe() {
    let mut renderer = hardware_renderer(960, 720);
    let device = renderer.device.clone();
    let queue = renderer.queue.clone();
    let report = |stage| {
        let Some(report) = device.generate_allocator_report() else {
            panic!("allocator report unavailable at {stage}");
        };
        eprintln!("REPORT {stage}: allocated={} reserved={} allocations={}",
            report.total_allocated_bytes, report.total_reserved_bytes, report.allocations.len());
        for a in &report.allocations {
            eprintln!("ALLOCATION {stage}: name={:?} offset={} size={}", a.name, a.offset, a.size);
        }
        let atlas: Vec<_> = report.allocations.iter().filter(|a| a.name == "glyphon atlas").collect();
        eprintln!("ATLAS {stage}: count={} bytes={}", atlas.len(), atlas.iter().map(|a|a.size).sum::<u64>());
    };
    report("created");
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let prepared = PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    capture(&mut renderer, &prepared);
    report("drawn");
    drop(renderer);
    queue.submit([]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    report("dropped-drained");
}
'''
try:
 p.write_bytes(old+probe.encode())
 with open('/tmp/pm045-allocator-public-probe.log','w') as log:
  result=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','pm045_public_allocator_report_probe','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
 print('probe exit',result.returncode)
finally:p.write_bytes(old)
