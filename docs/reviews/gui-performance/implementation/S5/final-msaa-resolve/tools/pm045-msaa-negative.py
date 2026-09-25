from pathlib import Path
import subprocess,json
p=Path('crates/gui-render/src/render/gpu_frame.rs');before=p.read_bytes();s=before.decode();needle='(final_resolve == ResolvePass::TerminalForeground).then_some(&view)';assert s.count(needle)==1
try:
 p.write_text(s.replace(needle,'None /* negative control: omit the final foreground resolve */'))
 with open('/tmp/pm045-msaa-negative.log','w') as log:
  rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','-p','datum-gui-render','--features','visual','--lib','terminal_last_layer_matches_invisible_later_passes','--offline','--','--ignored','--test-threads=1'],stdout=log,stderr=subprocess.STDOUT).returncode
 text=Path('/tmp/pm045-msaa-negative.log').read_text();assert rc!=0 and 'test result: FAILED' in text and 'invisible later menu passes must preserve terminal samples' in text,(rc,text[-2000:])
 Path('/tmp/pm045-msaa-negative-result.json').write_text(json.dumps({'returncode':rc,'expected_failure':True,'mutation':needle+' -> None','assertion':'invisible later menu passes must preserve terminal samples'},indent=2)+'\n')
finally:p.write_bytes(before)
assert p.read_bytes()==before
