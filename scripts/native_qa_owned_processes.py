"""Owned native-QA process teardown, never ordinary window-Close proof."""
import os
from pathlib import Path
import signal
import subprocess
import time


def _gui_exited(qa):
    if getattr(qa, '_owned_cleanup_failed', False):
        raise RuntimeError('Prior owned shutdown failed; private runtime must remain retained')
    if qa.process is not None and qa.process.poll() is None:
        raise RuntimeError('GUI exit is unconfirmed; refusing companion/runtime cleanup')


def _retain_runtime(qa, error):
    if not getattr(qa, '_owned_cleanup_failed', False):
        qa.runtime._finalizer.detach()
        qa._owned_cleanup_failed = True
    qa.observations.setdefault('retained_owned_runtime', {
        'path': qa.runtime.name, 'reason': str(error),
        'limit': 'Failed teardown; retained for inspection, not ordinary Close proof'})


def close_owned_companions(qa, result):
    """Reap owned companions, retaining the runtime on every failure path."""
    try:
        _reap_owned_companions(qa, result)
    except BaseException as error:
        _retain_runtime(qa, error)
        raise


def _reap_owned_companions(qa, result):
    """Reap only same-user, exact-binary companions in this private runtime."""
    _gui_exited(qa)
    actions = qa.observations.setdefault(
        'owned_companion_housekeeping', result.get('owned_companion_housekeeping', []))
    result['owned_companion_housekeeping'] = actions
    binary = (qa.binary_dir / 'eda-engine-daemon').resolve()
    for proc in Path('/proc').iterdir():
        if not proc.name.isdecimal():
            continue
        try:
            if proc.stat().st_uid != os.getuid() or (proc / 'exe').resolve() != binary:
                continue
            raw = (proc / 'cmdline').read_bytes()
            argv = [value.decode() for value in raw.split(b'\0') if value]
            if len(argv) != 3 or argv[1] != '--socket':
                continue
            socket = Path(argv[2])
            if (socket.parent.parent != Path(qa.runtime.name) or socket.name != 'engine.sock'
                    or not socket.parent.name.startswith('datum-native-edit-')):
                continue
            pid = int(proc.name)
            fd = os.pidfd_open(pid)
            try:
                if (proc.stat().st_uid != os.getuid() or (proc / 'exe').resolve() != binary
                        or (proc / 'cmdline').read_bytes() != raw):
                    raise RuntimeError(f'Owned companion {pid} identity changed before signalling')
                signal.pidfd_send_signal(fd, signal.SIGTERM)
            finally:
                os.close(fd)
            until = time.monotonic() + 3
            while proc.exists() and time.monotonic() < until:
                time.sleep(.05)
            if proc.exists():
                raise RuntimeError(f'Owned companion {pid} still present after cleanup')
            actions.append({'pid': pid, 'socket': argv[2], 'signal': 'SIGTERM', 'gone': True,
                            'limit': 'Forced owned housekeeping, not ordinary Close/PTY proof'})
        except (FileNotFoundError, ProcessLookupError):
            continue


def stop_owned_gui(qa):
    """Stop this GUI and its companions, retaining the reusable input envelope."""
    try:
        if qa.process is not None:
            if qa.process.poll() is None:
                qa.actions.append({'terminate_owned_app_pid': qa.process.pid, 'signal': 'SIGTERM',
                                   'limit': 'Process termination, not ordinary window Close'})
                qa.process.terminate()
                try:
                    qa.process.wait(timeout=6)
                except subprocess.TimeoutExpired:
                    os.killpg(qa.process.pid, signal.SIGTERM)
                    qa.process.wait(timeout=6)
            _gui_exited(qa)
        close_owned_companions(qa, qa.observations)
        if qa.app_log is not None:
            qa.app_log.close()
        log = Path(qa.env['DATUM_GUI_LOG']) if 'DATUM_GUI_LOG' in qa.env else None
        if log is not None and log.exists():
            qa.observations[qa.phase + '_all_surface_identities'] = [
                line for line in log.read_text().splitlines() if 'surface identity' in line]
        qa.process = None
    except BaseException as error:
        _retain_runtime(qa, error)
        raise


def close_owned_envelope(qa):
    """Remove the private runtime only after every owned shutdown is confirmed."""
    try:
        _gui_exited(qa)
        close_owned_companions(qa, qa.observations)
        if qa.clipboard_writer and qa.clipboard_writer.poll() is None:
            qa.clipboard_writer.terminate()
            qa.clipboard_writer.wait(timeout=5)
        if qa.compositor:
            if qa.compositor.poll() is None:
                qa.compositor.terminate()
                qa.compositor.wait(timeout=10)
            qa.compositor_log.close()
        qa.runtime.cleanup()
    except BaseException as error:
        _retain_runtime(qa, error)
        raise
