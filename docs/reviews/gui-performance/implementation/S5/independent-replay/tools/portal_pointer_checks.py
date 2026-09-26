"""Offline schedule and cancellation checks; never opens a portal or display."""
import contextlib
import importlib.util
import io
import json
import math
import os
from pathlib import Path
import sys
import tempfile
import types

spec = importlib.util.spec_from_file_location('pointer', Path(__file__).with_name('portal_pointer.py'))
pointer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pointer)


def check_schedule():
    rows = pointer.schedule()
    assert len(rows) == 3600
    for scale in (1, 1.5, 2):
        previous = (300, 150)
        for index, row in enumerate(rows):
            assert row['index'] == index
            assert row['offset_ns'] == round(index*10**9/120)
            position = tuple(row['physical_position'])
            assert position != previous
            assert 300 <= position[0] <= 700 and 150 <= position[1] <= 390
            expected = (400 if index//900 % 2 == 0 else 240)/900
            assert math.isclose(math.hypot(*row['physical_delta']), expected, abs_tol=1e-12)
            previous = position
        assert previous == (300, 150)
        assert len({tuple(round(v/scale*256) for v in r['physical_position']) for r in rows}) == 3600
    assert [rows[i]['physical_position'] for i in (899, 1799, 2699, 3599)] == [
        [700, 150], [700, 390], [300, 390], [300, 150]]
    for axis in (0, 1):
        assert abs(sum(r['physical_delta'][axis] for r in rows)) < 1e-9


class Clock:
    now = 0

    def monotonic(self):
        return self.now/10**9

    def monotonic_ns(self):
        return self.now

    time_ns = monotonic_ns

    def sleep(self, seconds):
        self.now += round(seconds*10**9)


class Reply:
    def __init__(self, value):
        self.value = value

    def unpack(self):
        return self.value


def check_service(mode):
    clock = Clock()
    read_fd, write_fd = os.pipe()
    input_stream = os.fdopen(read_fd, 'r')
    original_select = pointer.select.select
    original_time, original_stdin = pointer.time, sys.stdin
    original_dumps = pointer.json.dumps
    old_repository = sys.modules.get('gi.repository')
    calls = []
    session_closed = []

    class Connection:
        callback = None

        def signal_subscribe(self, *args):
            self.callback = args[-2]
            return 1

        def signal_unsubscribe(self, token):
            assert token == 1

        def call_sync(self, dest, target, iface, method, arguments, *args):
            if method == 'Close':
                session_closed.append(target)
                return None
            if method == 'NotifyPointerMotion':
                calls.append(arguments)
                if mode == 'dbus_timeout':
                    clock.sleep(.1)
                    raise TimeoutError('mock D-Bus timeout exception')
                if mode == 'slow':
                    clock.sleep(.02)
                if (mode == 'cancel' and len(calls) == 1) or len(calls) == 3600:
                    os.write(write_fd, b'{"action":"close"}\n')
                return None
            result = {'CreateSession': {'session_handle': '/mock/session'},
                      'SelectDevices': {}, 'Start': {'devices': 2}}[method]
            handle = '/mock/'+method
            self.callback(None, None, handle, None, None, Reply((0, result)), None)
            return Reply((handle,))

    connection = Connection()
    ns = types.SimpleNamespace
    gio = ns(bus_get_sync=lambda *a: connection, BusType=ns(SESSION=1),
             DBusSignalFlags=ns(NONE=0), DBusCallFlags=ns(NONE=0))
    glib = ns(Variant=lambda signature, value: (signature, value),
              MainContext=ns(default=lambda: ns(pending=lambda: False)))
    sys.modules['gi.repository'] = ns(Gio=gio, GLib=glib)

    def ready(read, write, error, timeout=0):
        result = original_select(read, write, error, 0)
        if not result[0]:
            clock.sleep(timeout)
        return result

    def dumps(value, *args, **kwargs):
        if mode == 'logging_delay' and isinstance(value, dict) and value.get('kind') == 'request_begin':
            clock.sleep(.02)
        return original_dumps(value, *args, **kwargs)

    root = Path(tempfile.mkdtemp(prefix='pm045-portal-offline-'))
    try:
        pointer.time, sys.stdin, pointer.select.select = clock, input_stream, ready
        pointer.json.dumps = dumps
        if mode == 'partial':
            os.write(write_fd, b'{"action":')
        else:
            command = {'action': 'pointer_stream', 'output_scale': 1,
                       'initial_position_verified': True}
            os.write(write_fd, (json.dumps(command)+'\n').encode())
        failure = None
        with contextlib.redirect_stdout(io.StringIO()):
            try:
                pointer.serve(root/'run')
            except (TimeoutError, InterruptedError) as exc:
                failure = exc
        assert session_closed == ['/mock/session']
        events = [json.loads(line) for line in (root/'run/events.jsonl').read_text().splitlines()]
        if mode == 'normal':
            assert failure is None and len(calls) == 3600
            assert clock.now == 30_000_000_000
            assert sum(r['kind'] == 'request_returned' for r in events) == 3600
        elif mode == 'slow':
            assert isinstance(failure, TimeoutError) and len(calls) == 1
            assert clock.now == 20_000_000
        elif mode == 'logging_delay':
            assert isinstance(failure, TimeoutError) and not calls
            assert any(r['kind'] == 'deadline_missed_before_call' for r in events)
        elif mode == 'dbus_timeout':
            assert isinstance(failure, TimeoutError) and len(calls) == 1
            assert not any(r['kind'] == 'request_returned' for r in events)
            assert clock.now == 100_000_000
        elif mode == 'cancel':
            assert isinstance(failure, InterruptedError) and len(calls) == 1
        else:
            assert failure is None and not calls and clock.monotonic() == 600
        assert events[-1]['qualification_pass'] is False
    finally:
        pointer.time, sys.stdin, pointer.select.select = original_time, original_stdin, original_select
        pointer.json.dumps = original_dumps
        if old_repository is None:
            del sys.modules['gi.repository']
        else:
            sys.modules['gi.repository'] = old_repository
        input_stream.close()
        os.close(write_fd)
    print(mode+': PASS; receipts '+str(root/'run'))


if __name__ == '__main__':
    check_schedule()
    for case in ('normal', 'slow', 'logging_delay', 'dbus_timeout', 'cancel', 'partial'):
        check_service(case)
    print('Offline controls passed. No native delivery or runtime qualification.')
