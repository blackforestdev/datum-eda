#!/usr/bin/env python3
"""GP-CM05E raw-sample measurement PRODUCER (slice S2).

Drives the real release product surface and assembles raw timing/RSS samples into
the ``measurements.validate_trials`` schema (3 trials x 10 warm-ups + 100 samples,
each a typed sample plus a datum.preferences.measurement-capture.v2 evidence
record; one datum.preferences.measurement-trial.v2 log per trial). It self-
validates each produced measurement with the governed validator.

S2 covers the HEADLESS/CLI budgets, selected by --budget:
  * release-query/{defaults,populated} - timed queries over empty and populated stores.
  * durable/{set,reset}                - standalone human TTY-confirmed durable mutations
                                         (execute(confirmation=True); launch_to_response_ns).
  * project                            - project-genesis + project-validation, three seed modes.
  * storage                            - storage-growth: three Project modes plus the continuous
                                         Preferences generation history (_storage_sample).
durable-mutation /daemon is S3; the GUI budgets (gui-feedback, native-window-open,
window-lifecycle) are S4. The /standalone durable path and the _storage_sample path are the two
new typed pieces; every other variant reuses the blessed timed assembler.

A global pre-warm (prewarm()) runs once before the measured budgets to prime the process
image and filesystem caches, recorded as method provenance so the corpus states it measures
steady state; the one-time cold observation is retained and disclosed separately.

No retries-until-green: a failed or over-budget sample is preserved in its evidence
and surfaced by the validator; it is never re-run to obtain a passing value.
"""
import argparse
import json
import os
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve()
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance import measurements as meas  # noqa: E402
from global_preferences_acceptance import observations as obs  # noqa: E402
from global_preferences_acceptance.capture import Capture  # noqa: E402
from global_preferences_acceptance.inputs import Bundle, canonical, digest, parse, require  # noqa: E402
from global_preferences_acceptance.process import execute  # noqa: E402
from global_preferences_acceptance.storage_capture import snapshot  # noqa: E402

MATRIX_RELPATH = "specs/global_preferences_production_acceptance_matrix.json"
CAPTURE_SCHEMA = "datum.preferences.measurement-capture.v2"
TRIAL_SCHEMA = "datum.preferences.measurement-trial.v2"
STATE_SCHEMA = "datum.preferences.state.v2"
WARMUPS = 10
SAMPLES = 100
PREWARM_ITERATIONS = 5
MOTION_KEY = "datum.accessibility.reduced_motion"
PREWARM_SCHEMA = "datum.preferences.measurement-prewarm.v1"
# Reachable-daemon durable-mutation transport (slice S3).
DAEMON_READY_TIMEOUT = 10.0
MCP_PREPARE_SCHEMA = "datum.preferences.proposal.prepare"
MCP_ACCEPT_SCHEMA = "datum.preferences.proposal.accept_apply"
AUTHORIZE_RESULT_MARKER = '{\n  "proposal_id"'

# Timed headless query variants (release-query budget): verb -> CLI argv tail.
RELEASE_QUERY_ARGV = {
    "describe": ["preferences", "describe"],
    "list": ["preferences", "list"],
    "get": ["preferences", "get", "datum.accessibility.reduced_motion"],
    "search": ["preferences", "search", "motion"],
    "explain": ["preferences", "explain", "datum.accessibility.reduced_motion"],
    "preview-project-units-seed": ["preferences", "preview-project-units-seed", "--source", "global"],
}
PROJECT_MODES = ("factory", "global-missing", "global-populated")
STORAGE_VARIANTS = ("factory", "global-missing", "global-populated", "generation")
# Validator storage virtual roots -> sample byte field (observations.state_manifest
# categorisation; see measurements._storage_sample).
STORAGE_CATEGORIES = {"project": "project_bytes", "generations": "generation_bytes",
                      "receipts": "receipt_bytes", "request-index": "request_index_bytes",
                      "fixed": "fixed_overhead_bytes"}


def _mutation_argv(action, expected, identity, *, key=MOTION_KEY, value=True):
    """Model measure_global_preferences_release.mutation(): one human durable mutation."""
    argv = ["--format", "json", "preferences", action, key, "--expected", json.dumps(expected),
            "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL, identity)),
            "--reason", "GP-CM05E production acceptance"]
    if action == "set":
        argv += ["--value-json", json.dumps(value)]
    return argv


def _run_mutation(binary, action, expected, identity, env, timeout, *, key=MOTION_KEY, value=True):
    """Drive one real TTY-confirmed durable mutation; return (run, product response)."""
    run = execute(Path(binary), _mutation_argv(action, expected, identity, key=key, value=value),
                  env, timeout=timeout, confirmation=True)
    require(run["exit_code"] == 0 and not run["timed_out"] and run["confirmed"],
            f"durable mutation failed/timed-out/unconfirmed: {identity}")
    rendered = run["stdout"].decode("utf-8", "replace").replace("\r", "")
    start = rendered.find('{\n  "ok"')
    require(start >= 0, f"TTY mutation produced no product envelope: {identity}")
    response = parse(rendered[start:].encode(), "TTY product response")
    require(isinstance(response, dict) and response.get("ok") is True, f"mutation refused: {identity}")
    return run, response


def _sha_file(path):
    import hashlib
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _isolated_env(config_root):
    env = {k: v for k, v in os.environ.items()
           if k not in {"XDG_CONFIG_HOME", "DATUM_GUI_PREFERENCES_PATH",
                        "DATUM_ENGINE_SOCKET", "EDA_ENGINE_SOCKET"}}
    env["XDG_CONFIG_HOME"] = str(config_root)
    return env


def _timed_observation(run, *, launch_to_response_ns=None, daemon_rss_kib=None):
    """Assemble the closed timed-sample shape minus its index/evidence placeholders."""
    ok = run["exit_code"] == 0 and not run["timed_out"]
    return {
        "kind": "timed",
        "started_monotonic_ns": run["started_monotonic_ns"],
        "elapsed_ns": run["elapsed_ns"],
        "rss_kib": run["rss_kib"],
        "daemon_rss_kib": daemon_rss_kib,
        "launch_to_response_ns": launch_to_response_ns,
        "presented_elapsed_ns": None,
        "outcome": "success" if ok else "failure",
    }


def _retain(capture, identity, trial_index, phase, index, run, observation):
    """Seal a sample's underlying measurement-capture.v2 record and attach its reference."""
    sample = {"index": index, **observation}
    record = {"schema": CAPTURE_SCHEMA, "measurement": identity, "trial_index": trial_index,
              "phase": phase, "exit_code": run["exit_code"], "timed_out": run["timed_out"],
              "observation": dict(sample)}
    sample["evidence"] = capture.put(record)
    return sample


def _timed_run(capture, identity, trial_index, phase, index, binary, full_argv, env, timeout):
    """Timed sample over an exact argv (no implicit --format json)."""
    run = execute(Path(binary), full_argv, env, timeout=timeout)
    return _retain(capture, identity, trial_index, phase, index, run, _timed_observation(run))


def _timed_sample(capture, identity, trial_index, phase, index, binary, argv, env, timeout):
    return _timed_run(capture, identity, trial_index, phase, index, binary,
                      ["--format", "json", *argv], env, timeout)


def prewarm(capture, binary, candidate_revision, binary_sha256, timeout, scratch):
    """Prime the process image and filesystem caches before the first measured trial so every
    trial reflects STEADY STATE, not first-ever cold start (project-genesis factory/trial0 paid a
    one-time cold-cache cost of ~107 ms while warm trials ran 35-55 ms). This is recorded as method
    provenance and is NOT a measured sample; the per-trial warm-ups still run unchanged. The cold
    observation is retained and disclosed separately in the corpus and is never re-rolled or deleted.
    (GP-CM05E, PM051 measurement-protocol clarification; contract measurement-protocol section.)"""
    root = scratch / "prewarm"
    root.mkdir(parents=True, exist_ok=True)
    env = _isolated_env(root / "config")
    for index in range(PREWARM_ITERATIONS):
        seed = f"prewarm/{index}"
        execute(Path(binary), ["project", "new", "--name", f"Prewarm {index}",
                "--project-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/project")),
                "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request")),
                "--units-source", "factory", "--json", str(root / f"project-{index}")],
                env, timeout=timeout)
        execute(Path(binary), ["--format", "json", "preferences", "describe"], env, timeout=timeout)
    record = {"schema": PREWARM_SCHEMA, "candidate_revision": candidate_revision,
              "binary_sha256": binary_sha256, "iterations": PREWARM_ITERATIONS,
              "operations": ["project.new", "preferences.describe"],
              "note": "Global warm-up before the first measured trial; measurements reflect steady "
                      "state. The first-ever cold observation is retained and disclosed separately "
                      "and is never re-rolled."}
    return capture.put(record)


def build_timed_measurement(capture, *, budget_id, variant_id, argv, binary,
                            candidate_revision, binary_sha256, input_manifest_sha256,
                            matrix_sha256, environment_id, timeout, scratch, setup=None):
    identity = {"budget_id": budget_id, "variant_id": variant_id, "environment_id": environment_id,
                "candidate_revision": candidate_revision, "binary_sha256": binary_sha256,
                "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256}
    trials = []
    for trial_index in range(3):
        config_root = scratch / f"{budget_id}-{variant_id.replace('/', '_')}-trial{trial_index}"
        config_root.mkdir(parents=True, exist_ok=True)
        env = _isolated_env(config_root)
        if setup is not None:
            setup(binary, env, trial_index, timeout)  # one-time per-trial state; not a measured sample
        warmups = [_timed_sample(capture, identity, trial_index, "warmups", i, binary, argv, env, timeout)
                   for i in range(WARMUPS)]
        samples = [_timed_sample(capture, identity, trial_index, "samples", i, binary, argv, env, timeout)
                   for i in range(SAMPLES)]
        trial = {"index": trial_index, "warmup_count": WARMUPS, "warmups": warmups,
                 "baseline_rss_kib": None, "baseline_resources": None, "cold_open": None,
                 "samples": samples}
        trial_log = {**trial, "schema": TRIAL_SCHEMA, "measurement": identity}
        trial["log"] = capture.put(trial_log)
        trials.append(trial)
    return {**identity, "trials": trials}


def _seal_trial(capture, identity, trial_index, warmups, samples):
    """Build one trial envelope and seal its measurement-trial.v2 raw log."""
    trial = {"index": trial_index, "warmup_count": WARMUPS, "warmups": warmups,
             "baseline_rss_kib": None, "baseline_resources": None, "cold_open": None,
             "samples": samples}
    trial_log = {**trial, "schema": TRIAL_SCHEMA, "measurement": identity}
    trial["log"] = capture.put(trial_log)
    return trial


def _assemble_trials(capture, identity, per_trial, trials=3):
    """Wrap per-trial producer callables (one per trial) into the validated trial envelope.

    The governed validator requires exactly three trials; ``trials`` is overridden below three
    only by the unscored S3 wire/timing proof, which deliberately does not call validate_trials."""
    return [_seal_trial(capture, identity, trial_index, *per_trial(trial_index))
            for trial_index in range(trials)]


def build_durable_mutation_measurement(capture, *, action, binary, candidate_revision, binary_sha256,
                                       input_manifest_sha256, matrix_sha256, environment_id, timeout, scratch):
    """Standalone human durable set/reset: every sample a real TTY-confirmed committed mutation.

    launch_to_response_ns is the full launch->exit interval; elapsed_ns begins at the automatic
    APPLY submission (process.execute). daemon_rss_kib stays None for the /standalone transport.
    A reset needs a prior override, so each reset sample is preceded by a retained set that is
    itself never scored. No sample is re-run to obtain a passing value.
    """
    variant_id = f"{action}/standalone"
    identity = {"budget_id": "durable-mutation", "variant_id": variant_id, "environment_id": environment_id,
                "candidate_revision": candidate_revision, "binary_sha256": binary_sha256,
                "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256}

    def per_trial(trial_index):
        config_root = scratch / f"durable-{action}-trial{trial_index}"
        config_root.mkdir(parents=True, exist_ok=True)
        env = _isolated_env(config_root)
        state = {"expected": {"kind": "missing"}}

        def one(phase, index):
            if action == "reset":
                _, setup = _run_mutation(binary, "set", state["expected"],
                                         f"durable/{action}/{trial_index}/{phase}/setup/{index}", env, timeout, value=True)
                state["expected"] = {"kind": "generation", "generation": setup["context"]["generation"]}
            run, response = _run_mutation(binary, action, state["expected"],
                                          f"durable/{action}/{trial_index}/{phase}/{index}", env, timeout,
                                          value=index % 2 == 0)
            require(response["result"]["class"] == "mutation" and response["result"]["result"]["changed"] is True,
                    "durable measurement was a no-op rather than a committed mutation")
            state["expected"] = {"kind": "generation", "generation": response["context"]["generation"]}
            return _retain(capture, identity, trial_index, phase, index, run,
                           _timed_observation(run, launch_to_response_ns=run["launch_to_response_ns"]))

        warmups = [one("warmups", i) for i in range(WARMUPS)]
        samples = [one("samples", i) for i in range(SAMPLES)]
        return warmups, samples

    return {**identity, "trials": _assemble_trials(capture, identity, per_trial)}


class _DaemonProcess:
    """A per-trial ``eda-engine-daemon`` owning one isolated Preferences store over a private Unix
    socket. The daemon services one connection at a time, so every round-trip opens, sends one
    JSON-RPC line, reads one response line and closes; its broker/service state persists for the
    daemon lifetime across those connections. Torn down at the end of the trial, pass or fail."""

    def __init__(self, binary, config_root, timeout):
        self.binary = Path(binary)
        self.config_root = Path(config_root)
        self.timeout = timeout
        self.process = None
        # A Unix socket path must satisfy SUN_LEN (~108 bytes), so it cannot live under the deep
        # per-trial scratch tree; keep it in a short private temp dir the daemon owns and cleans up.
        self._socket_dir = Path(tempfile.mkdtemp(prefix="dgp-"))
        self.socket_path = self._socket_dir / "d.sock"

    def __enter__(self):
        self.config_root.mkdir(parents=True, exist_ok=True)
        env = _isolated_env(self.config_root)  # strips inherited socket vars; owns its XDG store
        self.process = subprocess.Popen([str(self.binary), "--socket", str(self.socket_path)],
                                        env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        deadline = time.monotonic() + DAEMON_READY_TIMEOUT
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                detail = self.process.stderr.read().decode("utf-8", "replace")
                raise RuntimeError(f"eda-engine-daemon exited before binding its socket: {detail}")
            if self.socket_path.exists():
                try:
                    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as probe:
                        probe.connect(str(self.socket_path))
                    return self
                except OSError:
                    pass
            time.sleep(0.01)
        self._terminate()
        raise RuntimeError("eda-engine-daemon did not accept its socket within the ready timeout")

    def __exit__(self, *_exc):
        self._terminate()

    def _terminate(self):
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        shutil.rmtree(self._socket_dir, ignore_errors=True)

    def rss_kib(self):
        """Owning daemon resident set, read from /proc/<pid>/status VmRSS (KiB)."""
        require(self.process is not None and self.process.poll() is None,
                "daemon is not running when its RSS was sampled")
        for line in Path(f"/proc/{self.process.pid}/status").read_text().splitlines():
            if line.startswith("VmRSS:"):
                kib = int(line.split()[1])
                require(kib >= 1, "daemon VmRSS must be a positive KiB count")
                return kib
        raise RuntimeError("daemon /proc status reported no VmRSS")

    def request(self, method, params):
        """One JSON-RPC line round-trip against the sequential daemon; return the ``result`` value."""
        wire = {"jsonrpc": "2.0", "id": str(uuid.uuid4()), "method": method, "params": params}
        payload = (json.dumps(wire) + "\n").encode()
        buffer = bytearray()
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(self.timeout)
            client.connect(str(self.socket_path))
            client.sendall(payload)
            while b"\n" not in buffer:
                chunk = client.recv(65536)
                if not chunk:
                    break
                buffer.extend(chunk)
        response = parse(bytes(buffer).split(b"\n", 1)[0], f"daemon {method} response")
        require(isinstance(response, dict) and response.get("error") is None,
                f"daemon {method} refused: {response.get('error') if isinstance(response, dict) else response}")
        return response["result"]


def _mcp_actor(session_id, invocation_id):
    return {"kind": "mcp_agent", "session_id": session_id, "local_actor_id": "gp-cm05e-agent",
            "invocation_id": str(invocation_id)}


def _mcp_proposal_result(daemon, schema, action_request, actor):
    """Drive one ``preferences.mcp_product`` proposal call; return the inner proposal result dict."""
    envelope = daemon.request("preferences.mcp_product", {
        "request": {"schema": {"name": schema, "version": 1},
                    "payload": {"class": "proposal", "request": action_request}},
        "actor": actor})
    require(isinstance(envelope, dict) and envelope.get("ok") is True and envelope.get("error") is None,
            f"MCP product envelope refused: {envelope.get('error') if isinstance(envelope, dict) else envelope}")
    result = envelope["result"]
    require(isinstance(result, dict) and result.get("class") == "proposal",
            "expected a proposal-class product result")
    return result["result"]  # PreferenceProposalResultV1 == {"kind": ..., "result": ...}


def _prepare_proposal(daemon, mutation, rationale, actor):
    prepared = _mcp_proposal_result(daemon, MCP_PREPARE_SCHEMA,
                                    {"kind": "prepare", "mutation": mutation, "rationale": rationale}, actor)
    require(prepared.get("kind") == "prepared", f"proposal was not prepared: {prepared.get('kind')}")
    return prepared["result"]["proposal"]


def _accept_apply(daemon, proposal, actor):
    accepted = _mcp_proposal_result(daemon, MCP_ACCEPT_SCHEMA,
                                    {"kind": "accept_and_apply", "proposal": proposal}, actor)
    require(accepted.get("kind") == "accepted", f"authorized apply was not accepted: {accepted.get('kind')}")
    mutation_result = accepted["result"]["mutation_result"]
    require(mutation_result.get("changed") is True,
            "daemon durable mutation was a no-op rather than a committed change")
    return mutation_result


def _authorize_mcp(binary, proposal_path, mcp_session, socket_path, config_root, timeout):
    """Real human ``preferences proposal authorize-mcp`` over a pty: actual foreground-TTY APPLY,
    actor human_cli, routed to the owning daemon. Returns (process run, authorization result)."""
    env = _isolated_env(config_root)
    env["DATUM_ENGINE_SOCKET"] = str(socket_path)
    argv = ["--format", "json", "preferences", "proposal", "authorize-mcp",
            "--proposal-json", str(proposal_path), "--mcp-session", mcp_session]
    run = execute(Path(binary), argv, env, timeout=timeout, confirmation=True)
    require(run["exit_code"] == 0 and not run["timed_out"] and run["confirmed"],
            f"authorize-mcp failed/timed-out/unconfirmed for session {mcp_session}")
    rendered = run["stdout"].decode("utf-8", "replace").replace("\r", "")
    start = rendered.find(AUTHORIZE_RESULT_MARKER)
    if start < 0:  # fall back to the first object after the APPLY prompt
        prompt = rendered.find("Type APPLY to confirm:")
        start = rendered.find("{", prompt if prompt >= 0 else 0)
    require(start >= 0, "authorize-mcp produced no JSON authorization result")
    result = parse(rendered[start:].encode(), "authorize-mcp result")
    require(isinstance(result, dict) and result.get("authorized") is True,
            "MCP apply was not authorized by the human CLI")
    return run, result


def _daemon_durable_mutation(daemon, binary, action, expected, seed, session,
                             config_root, proposal_dir, timeout, *, value=True, key=MOTION_KEY):
    """One full reachable-daemon durable mutation: MCP prepare -> human authorize-mcp (pty APPLY)
    -> MCP accept_apply commit. Returns the composed timed observation, the authorize process run,
    and the resulting generation for the next expected head. Never re-rolled on failure."""
    request_id = str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request"))
    reason = "GP-CM05E production acceptance"
    if action == "set":
        mutation = {"kind": "set_user", "key": key, "value": value, "expected": expected,
                    "request_id": request_id, "reason": reason}
    else:
        mutation = {"kind": "reset_user", "key": key, "expected": expected,
                    "request_id": request_id, "reason": reason}
    actor = _mcp_actor(session, uuid.uuid5(uuid.NAMESPACE_URL, seed + "/invocation"))
    proposal = _prepare_proposal(daemon, mutation, f"GP-CM05E {action} via daemon", actor)
    proposal_path = Path(proposal_dir) / f"proposal-{request_id}.json"
    proposal_path.write_text(json.dumps(proposal))
    run, _authorization = _authorize_mcp(binary, proposal_path, session, daemon.socket_path,
                                         config_root, timeout)
    mutation_result = _accept_apply(daemon, proposal, actor)
    commit_done = time.monotonic_ns()
    daemon_rss = daemon.rss_kib()
    # Timing composition (contract durable-mutation measurement clause): the human authorize-mcp
    # pty yields the launch (started)
    # and the automatic APPLY-submission instant (started + launch_to_response - elapsed); the only
    # durable response is the accept_apply commit receipt. All on one monotonic clock.
    started = run["started_monotonic_ns"]
    exited = started + run["launch_to_response_ns"]            # authorize-mcp exit
    apply_submitted = exited - run["elapsed_ns"]               # automatic APPLY confirmation
    elapsed_ns = commit_done - apply_submitted
    launch_to_response_ns = commit_done - started
    require(launch_to_response_ns >= elapsed_ns >= 0, "daemon durable timing endpoints are inconsistent")
    generation = mutation_result.get("generation")
    require(generation is not None, "committed daemon mutation returned no generation")
    observation = {"kind": "timed", "started_monotonic_ns": started, "elapsed_ns": elapsed_ns,
                   "rss_kib": run["rss_kib"], "daemon_rss_kib": daemon_rss,
                   "launch_to_response_ns": launch_to_response_ns, "presented_elapsed_ns": None,
                   "outcome": "success"}
    return {"run": run, "observation": observation,
            "expected": {"kind": "generation", "generation": generation}}


def build_daemon_mutation_measurement(capture, *, action, binary, daemon_binary, candidate_revision,
                                      binary_sha256, input_manifest_sha256, matrix_sha256,
                                      environment_id, timeout, scratch, warmups=WARMUPS,
                                      samples=SAMPLES, trials=3):
    """Reachable-daemon durable set/reset (contract durable-mutation measurement clause). Every
    sample is a real human-authorized
    durable mutation committed through the owning daemon: MCP proposes, the human confirms on a
    foreground TTY via ``authorize-mcp``, the agent's authorized accept_apply commits (changed=true).

    elapsed_ns spans the automatic APPLY submission (in the authorize-mcp pty) through the durable
    commit receipt; launch_to_response_ns spans the authorize-mcp launch through that commit and is
    retained separately. rss_kib is the human authorize CLI process; daemon_rss_kib is the owning
    daemon's /proc VmRSS -- present exactly because this is the /daemon transport. The APPLY->commit
    interval includes the agent's apply-invocation IPC, a conservative over-count disclosed as method
    provenance and never subtracted. A reset needs a prior override, so each reset sample is preceded
    by a retained daemon set that is itself never scored. No sample is re-run to obtain a passing value.
    """
    variant_id = f"{action}/daemon"
    identity = {"budget_id": "durable-mutation", "variant_id": variant_id, "environment_id": environment_id,
                "candidate_revision": candidate_revision, "binary_sha256": binary_sha256,
                "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256}

    def per_trial(trial_index):
        trial_dir = scratch / f"daemon-{action}-trial{trial_index}"
        config_root = trial_dir / "config"
        trial_dir.mkdir(parents=True, exist_ok=True)
        session = f"mcp:gp-cm05e:{action}:{trial_index}:{uuid.uuid4()}"
        state = {"expected": {"kind": "missing"}}
        with _DaemonProcess(daemon_binary, config_root, timeout) as daemon:
            def one(phase, index):
                if action == "reset":
                    setup = _daemon_durable_mutation(
                        daemon, binary, "set", state["expected"],
                        f"daemon/{action}/{trial_index}/{phase}/setup/{index}", session,
                        config_root, trial_dir, timeout, value=True)
                    state["expected"] = setup["expected"]  # retained override, not scored
                scored = _daemon_durable_mutation(
                    daemon, binary, action, state["expected"],
                    f"daemon/{action}/{trial_index}/{phase}/{index}", session,
                    config_root, trial_dir, timeout, value=index % 2 == 0)
                state["expected"] = scored["expected"]
                return _retain(capture, identity, trial_index, phase, index,
                               scored["run"], scored["observation"])

            warm = [one("warmups", i) for i in range(warmups)]
            samp = [one("samples", i) for i in range(samples)]
        return warm, samp

    return {**identity, "trials": _assemble_trials(capture, identity, per_trial, trials)}


def build_project_measurements(capture, *, mode, binary, candidate_revision, binary_sha256,
                               input_manifest_sha256, matrix_sha256, environment_id, timeout, scratch):
    """One genesis + validate pass per index yields both project-genesis and project-validation."""
    common = {"environment_id": environment_id, "candidate_revision": candidate_revision,
              "binary_sha256": binary_sha256, "input_manifest_sha256": input_manifest_sha256,
              "matrix_sha256": matrix_sha256}
    genesis_id = {"budget_id": "project-genesis", "variant_id": mode, **common}
    validation_id = {"budget_id": "project-validation", "variant_id": mode, **common}

    def per_trial(trial_index):
        base = scratch / f"project-{mode}-trial{trial_index}"
        base.mkdir(parents=True, exist_ok=True)
        env = _isolated_env(base / "config")
        if mode == "global-populated":
            _run_mutation(binary, "set", {"kind": "missing"}, f"project/{mode}/{trial_index}/setup",
                          env, timeout, key="datum.units.system", value="imperial")
        buckets = {"g": {"warmups": [], "samples": []}, "v": {"warmups": [], "samples": []}}

        def one(phase, index):
            dest = base / f"{phase}-{index}"
            seed = f"datum:gp-cm05e:{trial_index}:{mode}:{phase}:{index}"
            new_argv = ["project", "new", "--name", f"Acceptance {index}",
                        "--project-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/project")),
                        "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request")),
                        "--units-source", "factory" if mode == "factory" else "global",
                        "--json", str(dest)]
            buckets["g"][phase].append(
                _timed_run(capture, genesis_id, trial_index, phase, index, binary, new_argv, env, timeout))
            buckets["v"][phase].append(
                _timed_run(capture, validation_id, trial_index, phase, index, binary,
                           ["project", "validate", str(dest)], env, timeout))

        for i in range(WARMUPS):
            one("warmups", i)
        for i in range(SAMPLES):
            one("samples", i)
        return buckets

    genesis_trials, validation_trials = [], []
    for trial_index in range(3):
        buckets = per_trial(trial_index)
        for ident, trials, key in ((genesis_id, genesis_trials, "g"), (validation_id, validation_trials, "v")):
            trials.append(_seal_trial(capture, ident, trial_index,
                                      buckets[key]["warmups"], buckets[key]["samples"]))
    return [{**genesis_id, "trials": genesis_trials}, {**validation_id, "trials": validation_trials}]


def _within_trusted_base(path, base):
    """True iff ``path`` resolves to exactly its own location under the trusted, producer-created
    ``base`` -- i.e. it introduces no symlink below ``base``. A symlinked ``base`` or temp prefix is
    tolerated (portable), while a redirected destination or a symlinked tail component is rejected.
    (GP-CM05E audit fold-in: the prior ``path == path.resolve()`` guard spuriously tripped whenever
    the scratch/tmp prefix was itself a symlink.)"""
    base, path = Path(base), Path(path)
    if not path.is_absolute():
        path = base / path
    try:
        tail = path.relative_to(base)
    except ValueError:
        return False
    return path.resolve() == base.resolve().joinpath(*tail.parts)


def _project_state(destination, capture, root_identity, base):
    """State manifest for a published Project under the validator's ``project/`` virtual root."""
    destination = Path(destination).absolute()
    require(_within_trusted_base(destination, base), "project destination is redirected")
    files = []
    if destination.exists():
        for path in sorted(destination.rglob("*")):
            mode = path.lstat().st_mode
            require(not stat.S_ISLNK(mode), "published Project contains an unexpected symlink")
            if stat.S_ISDIR(mode):
                continue
            require(stat.S_ISREG(mode), "project storage entry is not a regular file")
            raw = path.read_bytes()
            reference = capture.put(raw)
            files.append({"path": "project/" + path.relative_to(destination).as_posix(),
                          "kind": "file", "bytes": len(raw), "sha256": reference["sha256"]})
    state = {"schema": STATE_SCHEMA, "root": root_identity,
             "files": sorted(files, key=lambda entry: entry["path"])}
    return state, capture.put(state)


def _category_totals(state):
    sizes = {field: 0 for field in STORAGE_CATEGORIES.values()}
    for entry in state["files"]:
        sizes[STORAGE_CATEGORIES[entry["path"].split("/", 1)[0]]] += entry["bytes"]
    return sizes


def _storage_sample(capture, identity, trial_index, phase, index, run, before_ref, after_ref,
                    before_state, after_state):
    """Storage sample whose byte fields are derived from the retained before/after manifests."""
    before_totals, after_totals = _category_totals(before_state), _category_totals(after_state)
    growth = {field: after_totals[field] - before_totals[field] for field in STORAGE_CATEGORIES.values()}
    ok = run["exit_code"] == 0 and not run["timed_out"]
    sample = {"kind": "storage", "index": index, "started_monotonic_ns": run["started_monotonic_ns"],
              "project_bytes": growth["project_bytes"], "generation_bytes": growth["generation_bytes"],
              "receipt_bytes": growth["receipt_bytes"], "request_index_bytes": growth["request_index_bytes"],
              "fixed_overhead_bytes": growth["fixed_overhead_bytes"],
              "before": before_ref, "after": after_ref, "outcome": "success" if ok else "failure"}
    record = {"schema": CAPTURE_SCHEMA, "measurement": identity, "trial_index": trial_index,
              "phase": phase, "exit_code": run["exit_code"], "timed_out": run["timed_out"],
              "observation": dict(sample)}
    sample["evidence"] = capture.put(record)
    return sample


def build_storage_measurement(capture, *, variant, binary, candidate_revision, binary_sha256,
                              input_manifest_sha256, matrix_sha256, environment_id, timeout, scratch):
    """Storage growth. ``generation`` = one continuous Preferences history of committed mutations;
    the three project modes = independent Project genesis byte growth (before empty, after published)."""
    identity = {"budget_id": "storage-growth", "variant_id": variant, "environment_id": environment_id,
                "candidate_revision": candidate_revision, "binary_sha256": binary_sha256,
                "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256}

    def per_trial(trial_index):
        base = scratch / f"storage-{variant}-trial{trial_index}"
        base.mkdir(parents=True, exist_ok=True)

        if variant == "generation":
            env = _isolated_env(base / "config")
            repository = (base / "config/datum/preferences")
            root_identity = f"generation-storage/trial-{trial_index}"
            carry = {"expected": {"kind": "missing"}}
            before_state, before_ref = snapshot(repository, capture, root_identity)

            def one(phase, index):
                run, response = _run_mutation(binary, "set", carry["expected"],
                                              f"storage/generation/{trial_index}/{phase}/{index}",
                                              env, timeout, value=index % 2 == 0)
                require(response["result"]["class"] == "mutation" and response["result"]["result"]["changed"] is True,
                        "storage mutation did not retain a new generation")
                carry["expected"] = {"kind": "generation", "generation": response["context"]["generation"]}
                after_state, after_ref = snapshot(repository, capture, root_identity)
                sample = _storage_sample(capture, identity, trial_index, phase, index, run,
                                         carry["before_ref"], after_ref, carry["before_state"], after_state)
                carry["before_ref"], carry["before_state"] = after_ref, after_state
                return sample

            carry["before_ref"], carry["before_state"] = before_ref, before_state
            warmups = [one("warmups", i) for i in range(WARMUPS)]
            samples = [one("samples", i) for i in range(SAMPLES)]
            return warmups, samples

        env = _isolated_env(base / "config")
        root_identity = f"project-storage/{variant}/trial-{trial_index}"
        before_state, before_ref = _project_state(base / ".absent", capture, root_identity, base)
        if variant == "global-populated":
            _run_mutation(binary, "set", {"kind": "missing"}, f"storage/{variant}/{trial_index}/setup",
                          env, timeout, key="datum.units.system", value="imperial")

        def one(phase, index):
            dest = base / f"{phase}-{index}"
            seed = f"datum:gp-cm05e:storage:{trial_index}:{variant}:{phase}:{index}"
            new_argv = ["project", "new", "--name", f"Acceptance {index}",
                        "--project-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/project")),
                        "--request-id", str(uuid.uuid5(uuid.NAMESPACE_URL, seed + "/request")),
                        "--units-source", "factory" if variant == "factory" else "global",
                        "--json", str(dest)]
            run = execute(Path(binary), new_argv, env, timeout=timeout)
            require(run["exit_code"] == 0 and not run["timed_out"], f"project genesis failed: {seed}")
            after_state, after_ref = _project_state(dest, capture, root_identity, base)
            return _storage_sample(capture, identity, trial_index, phase, index, run,
                                   before_ref, after_ref, before_state, after_state)

        warmups = [one("warmups", i) for i in range(WARMUPS)]
        samples = [one("samples", i) for i in range(SAMPLES)]
        if variant != "global-populated":
            require(not (base / "config/datum/preferences").exists(),
                    "factory/global-missing genesis created a Preferences repository")
        return warmups, samples

    return {**identity, "trials": _assemble_trials(capture, identity, per_trial)}


def _populate_setup(binary, timeout):
    def setup(bin_path, env, trial_index, to):
        _run_mutation(bin_path, "set", {"kind": "missing"}, f"populate/trial-{trial_index}", env, to)
    return setup


def _produce(budget, capture, binary, common, timeout, scratch, *, daemon_binary=None,
             warmups=WARMUPS, samples=SAMPLES, trials=3):
    """Return the list of measurements for one headless budget selector."""
    if budget in ("release-query/defaults", "release-query/populated"):
        store = budget.split("/", 1)[1]
        setup = _populate_setup(binary, timeout) if store == "populated" else None
        return [build_timed_measurement(
            capture, budget_id="release-query", variant_id=f"{verb}/{store}",
            argv=RELEASE_QUERY_ARGV[verb], binary=binary, timeout=timeout, scratch=scratch,
            setup=setup, **common) for verb in RELEASE_QUERY_ARGV]
    if budget in ("durable/set", "durable/reset"):
        return [build_durable_mutation_measurement(
            capture, action=budget.split("/", 1)[1], binary=binary, timeout=timeout,
            scratch=scratch, **common)]
    if budget in ("daemon/set", "daemon/reset"):
        require(daemon_binary is not None, "daemon budgets require an executable daemon binary")
        return [build_daemon_mutation_measurement(
            capture, action=budget.split("/", 1)[1], binary=binary, daemon_binary=daemon_binary,
            timeout=timeout, scratch=scratch, warmups=warmups, samples=samples, trials=trials,
            **common)]
    if budget == "project":
        produced = []
        for mode in PROJECT_MODES:
            produced.extend(build_project_measurements(
                capture, mode=mode, binary=binary, timeout=timeout, scratch=scratch, **common))
        return produced
    if budget == "storage":
        return [build_storage_measurement(
            capture, variant=variant, binary=binary, timeout=timeout, scratch=scratch, **common)
            for variant in STORAGE_VARIANTS]
    raise ValueError(f"unknown budget selector {budget!r}")


def _report(measurement):
    budget, variant = measurement["budget_id"], measurement["variant_id"]
    if budget == "storage-growth":
        per_trial = [{"project": max(s["project_bytes"] for s in t["samples"]),
                      "generation": max(s["generation_bytes"] + s["receipt_bytes"] + s["request_index_bytes"]
                                        for s in t["samples"])} for t in measurement["trials"]]
        print(f"{budget} {variant}: validate_trials ACCEPTED; per-trial max bytes = {per_trial}")
    else:
        limit = meas.BUDGETS[budget]
        p95 = [meas.percentile([s["elapsed_ns"] for s in t["samples"]]) for t in measurement["trials"]]
        print(f"{budget} {variant}: validate_trials ACCEPTED; per-trial p95 ns = {p95} "
              f"(budget p95 <= {limit['p95_ms']}ms)")


def _report_proof(measurement):
    """Bounded-diagnostic summary for the S3 daemon mechanism (not a scored/validated result).

    Asserts, first-hand, the invariants the governed validator will enforce at the S6 campaign:
    every sample carries a positive daemon RSS (because the variant is /daemon), the launch-to-
    response interval is no shorter than the measured elapsed, and no sample is a failure."""
    budget, variant = measurement["budget_id"], measurement["variant_id"]
    samples = [s for trial in measurement["trials"] for s in trial["warmups"] + trial["samples"]]
    for s in samples:
        require(s["kind"] == "timed" and s["outcome"] == "success", "proof sample is not a successful timed sample")
        require(isinstance(s["daemon_rss_kib"], int) and s["daemon_rss_kib"] >= 1,
                "daemon sample is missing its positive /proc VmRSS")
        require(s["launch_to_response_ns"] >= s["elapsed_ns"] >= 0, "proof sample timing endpoints are inconsistent")
        require(s["presented_elapsed_ns"] is None, "headless daemon sample must not present a frame")
    per_trial_p95 = [meas.percentile([s["elapsed_ns"] for s in t["samples"]]) if t["samples"] else None
                     for t in measurement["trials"]]
    daemon_rss = [s["daemon_rss_kib"] for s in samples]
    limit = meas.BUDGETS[budget]
    print(f"[S3 PROOF] {budget} {variant}: {len(measurement['trials'])} trial(s), {len(samples)} samples; "
          f"per-trial elapsed p95 ns = {per_trial_p95} (budget p95 <= {limit['p95_ms']}ms); "
          f"daemon RSS KiB range = [{min(daemon_rss)}, {max(daemon_rss)}] (<= {limit['max_rss_mib'] * 1024}); "
          f"committed, daemon-accounted, APPLY->commit-spanned. Unscored diagnostic.")


def main(argv=None):
    import development_workspace as workspace
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-root", required=True, type=Path)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/datum-eda")
    parser.add_argument("--budget", default="release-query/defaults",
                        choices=["release-query/defaults", "release-query/populated",
                                 "durable/set", "durable/reset", "daemon/set", "daemon/reset",
                                 "project", "storage"],
                        help="headless S2/S3 budget selector")
    parser.add_argument("--daemon-binary", type=Path, default=ROOT / "target/release/eda-engine-daemon",
                        help="eda-engine-daemon release binary (daemon/* budgets only)")
    parser.add_argument("--timeout", type=float, default=15)
    parser.add_argument("--scratch", type=Path, default=workspace.scratch("gp-cm05e-measurement-roots"))
    # Trial/sample counts apply to the daemon/* budgets only; defaults are the governed full shape.
    # The bounded S3 wire/timing proof drives them below the validator's required (3, 10, 100) and
    # therefore pairs with --no-validate; the scored corpus runs at the S6 milestone campaign.
    parser.add_argument("--daemon-trials", type=int, default=3)
    parser.add_argument("--daemon-warmups", type=int, default=WARMUPS)
    parser.add_argument("--daemon-samples", type=int, default=SAMPLES)
    parser.add_argument("--no-validate", action="store_true",
                        help="skip the governed validator (bounded diagnostic proof of the mechanism only)")
    args = parser.parse_args(argv)

    binary = args.binary.resolve()
    if not (binary.is_file() and os.access(binary, os.X_OK)):
        parser.error("release binary must be executable")
    daemon_binary = None
    if args.budget.startswith("daemon/"):
        daemon_binary = args.daemon_binary.resolve()
        if not (daemon_binary.is_file() and os.access(daemon_binary, os.X_OK)):
            parser.error("daemon binary must be executable for daemon/* budgets")

    args.bundle_root.mkdir(parents=True, exist_ok=True)
    scratch = args.scratch.resolve()
    scratch.mkdir(parents=True, exist_ok=True)
    capture = Capture(args.bundle_root.resolve())

    import subprocess
    revision = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "HEAD"],
                              capture_output=True, text=True, check=True).stdout.strip()
    binary_sha256 = _sha_file(binary)
    matrix_sha256 = digest(canonical(parse((ROOT / MATRIX_RELPATH).read_bytes(), "matrix")))
    input_manifest_sha256 = digest(canonical([{"path": str(binary), "sha256": binary_sha256}]))
    common = {"candidate_revision": revision, "binary_sha256": binary_sha256,
              "input_manifest_sha256": input_manifest_sha256, "matrix_sha256": matrix_sha256,
              "environment_id": "headless-linux-x86_64"}

    prewarm_ref = prewarm(capture, binary, revision, binary_sha256, args.timeout, scratch)
    print(f"global pre-warm: {PREWARM_ITERATIONS} iterations recorded at {prewarm_ref['path']} "
          f"(method provenance; measurements reflect steady state)")

    produced = _produce(args.budget, capture, binary, common, args.timeout, scratch,
                        daemon_binary=daemon_binary, warmups=args.daemon_warmups,
                        samples=args.daemon_samples, trials=args.daemon_trials)

    bundle = Bundle(args.bundle_root.resolve(), list(capture.inventory.values()))
    for measurement in produced:
        if args.no_validate:
            _report_proof(measurement)
            continue
        try:
            meas.validate_trials(measurement, bundle)
        except Exception as error:  # preserve + diagnose; never re-roll
            print(f"MEASUREMENT VALIDATION FAILED ({measurement['budget_id']} "
                  f"{measurement['variant_id']}, preserved): {error}", file=sys.stderr)
            return 1
        _report(measurement)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
