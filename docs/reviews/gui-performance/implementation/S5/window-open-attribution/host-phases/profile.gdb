set pagination off
set confirm off
set print thread-events off
python
import gdb,time,json,os
pid=gdb.selected_inferior().pid
hz=os.sysconf('SC_CLK_TCK')
def cpu():
 with open('/proc/%s/stat'%pid) as f:s=f.read().rsplit(')',1)[1].split()
 return (int(s[11])+int(s[12]))/hz
class Finish(gdb.FinishBreakpoint):
 def __init__(self,name):
  super().__init__(internal=True);self.name=name;self.initial=cpu();self.start=time.monotonic()
 def stop(self):
  print('PROFILE '+json.dumps(dict(phase=self.name,cpu_ms=1000*(cpu()-self.initial),wall_ms=1000*(time.monotonic()-self.start))),flush=True)
  return False
class Begin(gdb.Breakpoint):
 def __init__(self,name):
  super().__init__(name,internal=True);self.name=name
 def stop(self):
  Finish(self.name);return False
for name in ['<eda_engine::substrate::ProjectResolver>::resolve','<datum_gui::project_preferences_runtime::ProjectPreferencesCoordinator>::new','<datum_gui::project_preferences_runtime::ProjectPreferencesCoordinator>::open_dialog','<datum_gui_render::renderer_state::Renderer>::new','<datum_gui_render::renderer_state::Renderer>::prepare_native_preferences_scrolled','<datum_gui_render::renderer_state::Renderer>::prepare_native_new_project_scrolled']:
 Begin(name)
print('PROFILE_READY',flush=True)
end
continue
