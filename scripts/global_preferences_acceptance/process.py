"""Observed Linux process timing, retained streams, wait4 RSS and bounded human-TTY input."""
import errno
import os
import pty
import select
import signal
import tempfile
import time


def execute(binary, arguments, environment, *, timeout=15, confirmation=False):
    """Measure actual process exit; mutation timing starts at automatic APPLY submission."""
    started = time.monotonic_ns()
    confirmed = None
    output = bytearray()
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        if confirmation:
            pid, descriptor = pty.fork()
        else:
            pid = os.fork()
            descriptor = None
        if pid == 0:
            try:
                if not confirmation:
                    os.setsid()
                    os.dup2(stdout.fileno(), 1)
                    os.dup2(stderr.fileno(), 2)
                os.execve(str(binary), [str(binary), *arguments], environment)
            except BaseException as error:
                os.write(2, (str(error) + '\n').encode())
                os._exit(127)
        if descriptor is not None:
            os.set_blocking(descriptor, False)
        timed_out = False
        reaped = False
        try:
            while True:
                if descriptor is not None:
                    readable, _, _ = select.select([descriptor], [], [], 0.001)
                    if readable:
                        try:
                            chunk = os.read(descriptor, 65536)
                            output.extend(chunk)
                        except OSError as error:
                            if error.errno not in (errno.EIO, errno.EAGAIN):
                                raise
                        if confirmed is None and b'Type APPLY to confirm:' in output:
                            confirmed = time.monotonic_ns()
                            os.write(descriptor, b'APPLY\n')
                ended, status, usage = os.wait4(pid, os.WNOHANG)
                if ended:
                    reaped = True
                    break
                if time.monotonic_ns() - started > timeout * 1_000_000_000:
                    os.killpg(pid, signal.SIGKILL)
                    _, status, usage = os.wait4(pid, 0)
                    reaped = True
                    timed_out = True
                    break
                if descriptor is None:
                    time.sleep(0.001)
            finished = time.monotonic_ns()
            if descriptor is not None:
                while True:
                    try:
                        chunk = os.read(descriptor, 65536)
                    except OSError as error:
                        if error.errno in (errno.EIO, errno.EAGAIN):
                            break
                        raise
                    if not chunk:
                        break
                    output.extend(chunk)
            stdout.seek(0)
            stderr.seek(0)
            return {'command': [str(binary), *arguments], 'started_monotonic_ns': started,
                    'elapsed_ns': finished - (confirmed or started),
                    'launch_to_response_ns': finished - started if confirmation else None,
                    'rss_kib': usage.ru_maxrss, 'exit_code': 124 if timed_out else os.waitstatus_to_exitcode(status),
                    'confirmed': confirmed is not None, 'timed_out': timed_out,
                    'stdout': bytes(output) if confirmation else stdout.read(),
                    'stderr': stderr.read()}
        finally:
            if not reaped:
                try:
                    os.killpg(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                os.wait4(pid, 0)
            if descriptor is not None:
                os.close(descriptor)
