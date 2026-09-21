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
 for name,source in [('batched-copy-probe',candidate)]:
  s=source.decode()
  a=s.index('    let mut start = None;');b=s.index('\n    uploaded\n}',a)
  s=s[:a]+'''    let mut spans = Vec::new();
    let mut compact = Vec::new();
    let mut begin = None;
    for offset in (0..new.len()).step_by(alignment) {
        if old.get(offset..offset+alignment) != Some(&new[offset..offset+alignment]) {
            begin.get_or_insert(offset);
        } else if let Some(start) = begin.take() { spans.push(start..offset); }
    }
    if let Some(start) = begin { spans.push(start..new.len()); }
    for span in &spans { compact.extend_from_slice(&new[span.clone()]); }
    use wgpu::util::DeviceExt;
    let stage = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("packed-copy-probe"), contents: &compact, usage: wgpu::BufferUsages::COPY_SRC,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    let mut offset = 0;
    for span in spans {
        encoder.copy_buffer_to_buffer(&stage, offset, buffer, span.start as u64, span.len() as u64);
        offset += span.len() as u64;
    }
    queue.submit([encoder.finish()]);
    let uploaded = compact.len();''' +s[b:]
  s=s.replace('write_dirty_ranges(queue, buffer,', 'write_dirty_ranges(device, queue, buffer,').replace('fn write_dirty_ranges(', 'fn write_dirty_ranges(device: &wgpu::Device, ')
  i=s.rfind('\n    }\n}');assert i>0;s=s[:i]+probe+s[i:];p.write_text(s)
  with open('/tmp/pm045-s2-dirty-words-'+name+'-timing.log','w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','screen_upload_reuses_exact_content','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode==0,name
finally:p.write_bytes(candidate)
