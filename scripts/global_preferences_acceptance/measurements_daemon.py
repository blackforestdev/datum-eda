#!/usr/bin/env python3
"""GP-CM05E reachable-daemon durable-mutation measurement orchestration (slice S3).

Owns the multi-actor /daemon transport for the durable-mutation budget: launch a per-trial
eda-engine-daemon over a private Unix socket, drive MCP prepare/accept_apply over it as the
mcp_agent, perform the human foreground-TTY APPLY via `preferences proposal authorize-mcp`, and
assemble each committed sample (daemon_rss_kib read from the daemon's /proc VmRSS).

Extracted from measurements_producer.py under PM022 source-health (SH-007 real ownership: this module
owns the daemon orchestration; the producer owns budget selection and the headless builders). The
move is behaviour-preserving. The shared sample primitives (_isolated_env, _retain, _assemble_trials)
and the WARMUPS/SAMPLES/MOTION_KEY constants live in measurements_producer and are imported here;
measurements_producer imports build_daemon_mutation_measurement lazily (only for daemon/* budgets), so
there is no import cycle in either direction.
"""
import json
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve()
sys.path.insert(0, str(HERE.parents[1]))

from global_preferences_acceptance.inputs import parse, require  # noqa: E402
from global_preferences_acceptance.process import execute  # noqa: E402
from global_preferences_acceptance.measurements_producer import (  # noqa: E402
    MOTION_KEY,
    SAMPLES,
    WARMUPS,
    _assemble_trials,
    _isolated_env,
    _retain,
)

DAEMON_READY_TIMEOUT = 10.0
MCP_PREPARE_SCHEMA = "datum.preferences.proposal.prepare"
MCP_ACCEPT_SCHEMA = "datum.preferences.proposal.accept_apply"
AUTHORIZE_RESULT_MARKER = '{\n  "proposal_id"'


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
