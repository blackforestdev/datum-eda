# A1 scale-corrected relaunch: blocked before execution

The launcher now explicitly sets `WINIT_X11_SCALE_FACTOR=1`, following
`scripts/gpu_r3_native_trial.py`. It clears inherited Datum diagnostic flags,
Wayland selection and loader injection, and prepares isolated XDG directories.
The existing diagnostic test command, Vulkan backend, fixture and original
300-second / 646-observation / 2,200-submission limits remain unchanged.

Preflight stopped at its first prerequisite: the approved executable
`target/release/deps/datum_gui-2b88501e8cc7fcea` no longer exists. Its required
SHA-256 is `48b46328d255bc117d30936985c60386fcae75b399da2cce1ab51a5c475f2678`.
The original successful build record names that path. A read-only search of
48 existing GUI/attribution ELF executables under `target` found no matching
copy. The cause of its disappearance is not established.

Zero additional diagnostic processes, observations, builds or renderer changes
occurred. The original failed launch archive, source patch and execution record
retain their recorded hashes. The one authorized relaunch remains unconsumed.
The corrected launcher is syntax checked, but subsequent environment/input
preconditions and runtime scale behavior cannot be reported as verified: the
fail-closed executable check prevented reaching them. It refuses to reuse a
reserved run directory and has no retry or build path.

Resumption requires an existing executable matching the approved hash. The
source archive cannot substitute for that artifact under the explicit no-rebuild
boundary. No replacement study or reconstruction is authorized by this result.

The overall GPU disposition is unchanged: prior renderer corrections and exact
R4 output remain credited; R4's measured p95 6.700167 ms and p99 10.205084 ms fail
the 4/8 ms budgets. Backend-operation cost, timing-marker effects, avoidable
Datum overhead and hardware/economic feasibility remain unquantified by A1.
Full DRM lifetime/duty, broader recovery/endurance and independent replay remain
separate qualification gaps. This artifact-availability failure is not a new
renderer defect or evidence of hardware infeasibility. S4/S5 and the main issue
remain open; CPU work stays deferred.
