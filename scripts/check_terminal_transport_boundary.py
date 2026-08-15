#!/usr/bin/env python3
"""Enforce the Datum-owned PTY transport ownership boundary."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_SRC = Path("crates/gui-app/src")
TRANSPORT = APP_SRC / "terminal_transport"

OWNERS = {
    "libc::posix_openpt": TRANSPORT / "linux/pty.rs",
    "libc::grantpt": TRANSPORT / "linux/pty.rs",
    "libc::unlockpt": TRANSPORT / "linux/pty.rs",
    "libc::ptsname_r": TRANSPORT / "linux/pty.rs",
    "libc::setsid": TRANSPORT / "linux/spawn.rs",
    "libc::TIOCSCTTY": TRANSPORT / "linux/spawn.rs",
    "libc::dup2": TRANSPORT / "linux/spawn.rs",
    "libc::TIOCSWINSZ": TRANSPORT / "linux/job_control.rs",
    "libc::kill": TRANSPORT / "linux/job_control.rs",
}

IDENTIFIER_OWNERS = {
    "posix_openpt": {TRANSPORT / "linux/pty.rs"},
    "grantpt": {TRANSPORT / "linux/pty.rs"},
    "unlockpt": {TRANSPORT / "linux/pty.rs"},
    "ptsname_r": {TRANSPORT / "linux/pty.rs"},
    "setsid": {TRANSPORT / "linux/spawn.rs"},
    "pre_exec": {TRANSPORT / "linux/spawn.rs"},
    "TIOCSCTTY": {TRANSPORT / "linux/spawn.rs"},
    "dup2": {TRANSPORT / "linux/spawn.rs"},
    "TIOCSWINSZ": {TRANSPORT / "linux/job_control.rs"},
    "kill": {TRANSPORT / "linux/job_control.rs"},
    "open_pty_pair": {TRANSPORT / "linux/pty.rs", TRANSPORT / "mod.rs"},
    "attach_child_pty": {TRANSPORT / "linux/spawn.rs", TRANSPORT / "mod.rs"},
    "signal_process_group": {
        TRANSPORT / "linux/job_control.rs", TRANSPORT / "session_handle.rs",
    },
}

REQUIRED_FILES = {
    "mod.rs": "prepare_terminal_transport",
    "request.rs": "struct TerminalTransportRequest",
    "event.rs": "enum TerminalTransportEvent",
    "reader.rs": "spawn_event_threads",
    "session_handle.rs": "struct TerminalTransportSession",
    "wake.rs": "struct TerminalWakeGate",
    "linux/pty.rs": "open_pty_pair",
    "linux/spawn.rs": "attach_child_pty",
    "linux/job_control.rs": "signal_process_group",
}

SEMANTIC_TOKENS = (
    "TerminalScreen", "TerminalLaneState", "pty_grid_mut", "apply_bytes",
    "datum_terminal_core", "DesignModel", "Operation", "commit(", "journal",
    "Clipboard", "gui_protocol", "datum_gui_protocol", "wgpu", "Renderer",
)

THIRD_PARTY_TOKENS = (
    "libghostty", "ghostty_vt", "alacritty_terminal", "portable_pty",
    "portable-pty", "libloading", "dlopen", "include!",
)


def rust_sources(root: Path) -> list[Path]:
    return sorted(
        path for path in (root / APP_SRC).rglob("*.rs")
        if not path.name.endswith("_tests.rs")
    )


def check(root: Path) -> list[str]:
    failures: list[str] = []
    transport_root = root / TRANSPORT
    sources = rust_sources(root)
    source_text = {
        path.relative_to(root): path.read_text(encoding="utf-8") for path in sources
    }

    for relative, marker in REQUIRED_FILES.items():
        path = transport_root / relative
        if not path.is_file():
            failures.append(f"terminal transport module is missing: {TRANSPORT / relative}")
            continue
        if marker not in path.read_text(encoding="utf-8"):
            failures.append(f"terminal transport module lacks owned marker {marker}: {path.relative_to(root)}")

    for marker, owner in OWNERS.items():
        locations = [path for path, text in source_text.items() if marker in text]
        if locations != [owner]:
            failures.append(
                f"{marker} must occur only in {owner} (found: "
                f"{', '.join(map(str, locations)) or 'none'})"
            )

    for marker, owners in IDENTIFIER_OWNERS.items():
        pattern = re.compile(rf"\b{re.escape(marker)}\b")
        escaped = [
            path for path, text in source_text.items()
            if pattern.search(text) and path not in owners
        ]
        if escaped:
            failures.append(
                f"terminal transport ownership escaped for {marker}: "
                + ", ".join(map(str, escaped))
            )

    transport_sources = {
        path: text for path, text in source_text.items() if TRANSPORT in path.parents
    }
    joined_transport = "\n".join(transport_sources.values())
    for marker in SEMANTIC_TOKENS + THIRD_PARTY_TOKENS:
        if marker in joined_transport:
            failures.append(f"terminal transport contains forbidden authority marker: {marker}")

    outside_transport = {
        path: text for path, text in source_text.items() if TRANSPORT not in path.parents
    }
    raw_ownership = (
        "openpty", "forkpty", "login_tty", "master_fd", "slave_fd",
        "RawFd", "OwnedFd", "AsRawFd", "FromRawFd",
    )
    for marker in raw_ownership:
        escaped = [path for path, text in outside_transport.items() if marker in text]
        if escaped:
            failures.append(
                f"terminal transport ownership escaped for {marker}: "
                + ", ".join(map(str, escaped))
            )

    handle = transport_sources.get(TRANSPORT / "session_handle.rs", "")
    if re.search(r"#\[derive\([^]]*Clone", handle):
        failures.append("terminal transport owning handle must not be Clone")
    for struct_name in ("PreparedTerminalTransport", "TerminalTransportSession"):
        body_match = re.search(
            rf"struct\s+{struct_name}\s*\{{(?P<body>.*?)\n\}}", handle, re.DOTALL
        )
        if not body_match:
            failures.append(f"terminal transport handle is missing: {struct_name}")
            continue
        if re.search(r"(?m)^\s*pub(?:\([^)]*\))?\s+\w+\s*:", body_match.group("body")):
            failures.append(f"terminal transport handle fields must all be private: {struct_name}")

    allowed_methods = {
        "process_group_id", "start", "try_recv_event", "recv_event_timeout",
        "write_bytes", "interrupt", "terminate", "resize",
    }
    public_functions = list(re.finditer(
        r"(?P<visibility>pub(?:\([^)]*\))?)\s+fn\s+(?P<name>\w+)\s*\(", handle
    ))
    constructors = [match for match in public_functions if match.group("name") == "new"]
    if len(constructors) != 2 or any(
        match.group("visibility") != "pub(super)" for match in constructors
    ):
        failures.append("terminal transport must have exactly two parent-private constructors")
    for match in public_functions:
        name = match.group("name")
        visibility = match.group("visibility")
        if name == "new":
            continue
        if name not in allowed_methods or visibility != "pub(crate)":
            failures.append(f"terminal transport exposes unapproved handle API: {visibility} fn {name}")
        signature_end = handle.find("{", match.end())
        signature = handle[match.start():signature_end if signature_end >= 0 else match.end()]
        if any(token in signature for token in ("File", "RawFd", "OwnedFd", "Receiver", "Arc<Mutex<File>>")):
            failures.append(f"terminal transport handle API exposes raw ownership: {name}")

    root_module = transport_sources.get(TRANSPORT / "mod.rs", "")
    if not re.search(r"(?m)^mod linux;$", root_module):
        failures.append("terminal transport Linux module must be private to its root")
    linux_sources = {
        path: text for path, text in transport_sources.items()
        if TRANSPORT / "linux" in path.parents
    }
    for path, text in linux_sources.items():
        if "pub(crate)" in text:
            failures.append(f"raw Linux transport API is broader than its root: {path}")
    for marker in ("prepare_terminal_transport", "open_pty_pair", "into_command", ".spawn()"):
        if marker not in root_module:
            failures.append(f"terminal transport root lacks real orchestration marker: {marker}")

    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        for failure in failures:
            print(f"terminal transport boundary: FAIL: {failure}", file=sys.stderr)
        return 1
    print("terminal transport boundary: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
