from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/screen_buffer.rs');candidate=p.read_bytes();baseline=subprocess.check_output(['git','show','HEAD:'+str(p)])
probe='''
        let mut moving = vec![[1.0_f32; 5]; 4096];
        let mut measured = ScreenBuffer::default();
        measured.sync(&device, &queue, "moving", &moving);
        queue.submit([]);
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let mut times = Vec::new();
        for round in 0..12 {
            for vertex in &mut moving { vertex[0] = round as f32 + 20.0; }
            let started = std::time::Instant::now();
            let bytes = measured.sync(&device, &queue, "moving", &moving);
            let elapsed = started.elapsed().as_micros();
            queue.submit([]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            times.push(elapsed);
            eprintln!("MOVING round={round} sync_us={elapsed} bytes={bytes}");
        }
        times.sort();
        eprintln!("MOVING median_sync_us={}", times[times.len()/2]);
'''
try:
 for name,source in [('trimmed-spans',candidate)]:
  s=source.decode();i=s.rfind('\n    }\n}');assert i>0;s=s[:i]+probe+s[i:];p.write_text(s)
  with open('/tmp/pm045-s2-dirty-words-'+name+'-timing.log','w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','screen_upload_reuses_exact_content','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode==0,name
finally:p.write_bytes(candidate)
