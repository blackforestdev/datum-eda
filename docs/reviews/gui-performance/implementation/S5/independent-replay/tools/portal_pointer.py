"""Bounded PM045 pointer schedule through the existing input-only portal ABI.

Planning is offline. Serving requests portal consent and needs exclusive desktop
use. Sent requests are not native delivery, semantic output or acceptance proof.
The caller owns GUI setup, physical geometry/scale verification, CPU/GPU capture,
native receipts, output oracles and the five-second still tail.
"""
import argparse
from fractions import Fraction
import json
import os
from pathlib import Path
import select
import sys
import time
import uuid


COUNT = 3600
HZ = 120
DURATION_NS = 30_000_000_000


def schedule():
    """One closed 400x240 physical-pixel rectangle; four 7.5-second edges."""
    previous = (Fraction(300), Fraction(150))
    rows = []
    for index in range(COUNT):
        edge, step = divmod(index, 900)
        u = Fraction(step + 1, 900)
        position = ((300 + 400*u, Fraction(150)),
                    (Fraction(700), 150 + 240*u),
                    (700 - 400*u, Fraction(390)),
                    (Fraction(300), 390 - 240*u))[edge]
        rows.append({'index': index, 'offset_ns': round(Fraction(index*10**9, HZ)),
                     'physical_position': list(map(float, position)),
                     'physical_delta': [float(a-b) for a, b in zip(position, previous)]})
        previous = position
    return rows


def declaration():
    return {'qualification_pass': False, 'count': COUNT, 'hz': HZ,
            'duration_ns': DURATION_NS, 'initial_physical_position': [300, 150],
            'method': 'Relative NotifyPointerMotion using existing input-only RemoteDesktop portal; no ScreenCast or device installation.',
            'preconditions': 'Exclusive desktop; verified target focus/pane, initial position and actual output scale; fresh consent; warmup completed.',
            'limits': 'Unexecuted native method. Portal receipt is not delivered motion. Verify each acknowledged native position and final semantic state; reject quantization, clipping, dropped input or contamination. Never relabel as Wayland when target is X11.',
            'rows': schedule()}


class Commands:
    """Bounded byte buffering; a partial line never blocks session revocation."""
    def __init__(self, fd):
        self.fd = fd
        self.buffer = b''
        self.eof = False

    def poll(self, timeout=0):
        if self.eof:
            return [{'action': 'close'}]
        if not select.select([self.fd], [], [], timeout)[0]:
            return []
        chunk = os.read(self.fd, 4096)
        if not chunk:
            self.eof = True
            if self.buffer:
                raise ValueError('EOF within incomplete command')
            return [{'action': 'close'}]
        self.buffer += chunk
        if len(self.buffer) > 4096:
            raise ValueError('command buffer exceeds4096bytes')
        lines = self.buffer.split(b'\n')
        self.buffer = lines.pop()
        return [json.loads(line) for line in lines]


def serve(out):
    # Load only for native execution; --plan needs neither desktop nor GI.
    from gi.repository import Gio, GLib
    out.mkdir(parents=True, exist_ok=False)
    connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    context = GLib.MainContext.default()
    responses = {}
    session = None
    pending = None
    records = (out/'events.jsonl').open('x')
    endpoint = '/org/freedesktop/portal/desktop'
    destination = 'org.freedesktop.portal.Desktop'
    interface = 'org.freedesktop.portal.RemoteDesktop'
    error = None

    def record(kind, **fields):
        row = {'kind': kind, 'monotonic_ns': time.monotonic_ns(),
               'realtime_ns': time.time_ns(), **fields}
        records.write(json.dumps(row)+'\n')
        records.flush()

    def call(method, signature, args, target=endpoint, iface=interface, timeout_ms=5000):
        return connection.call_sync(destination, target, iface, method,
                                    GLib.Variant(signature, args) if signature else None,
                                    None, Gio.DBusCallFlags.NONE, timeout_ms, None)

    def response(conn, sender, path, iface, name, args, data):
        responses[path] = args.unpack()

    subscription = connection.signal_subscribe(
        destination, 'org.freedesktop.portal.Request', 'Response', None, None,
        Gio.DBusSignalFlags.NONE, response, None)

    def request(method, signature, args):
        nonlocal pending
        pending = call(method, signature, args).unpack()[0]
        record('request', method=method, handle=pending)
        end = time.monotonic()+300
        while pending not in responses:
            while context.pending():
                context.iteration(False)
            if time.monotonic() > end:
                raise TimeoutError(method+' consent response timeout')
            time.sleep(.05)
        code, result = responses.pop(pending)
        pending = None
        record('response', method=method, code=code, result=result)
        if code:
            raise RuntimeError(method+' refused: '+str(code))
        return result

    try:
        declared = declaration()
        prepared_rows = declared['rows']
        commands = Commands(sys.stdin.fileno())
        (out/'declaration.json').write_text(json.dumps(declared, indent=2)+'\n')
        token = 'pm045_'+uuid.uuid4().hex
        session = request('CreateSession', '(a{sv})', ({
            'handle_token': GLib.Variant('s', token+'_create'),
            'session_handle_token': GLib.Variant('s', token+'_session')},))['session_handle']
        request('SelectDevices', '(oa{sv})', (session, {
            'types': GLib.Variant('u', 2),  # pointer only
            'handle_token': GLib.Variant('s', token+'_select')}))
        result = request('Start', '(osa{sv})', (session, '', {
            'handle_token': GLib.Variant('s', token+'_start')}))
        if not result.get('devices', 0) & 2:
            raise RuntimeError('portal did not grant pointer input')
        record('ready')
        print(json.dumps({'ready': True, 'output': str(out)}), flush=True)
        expires = time.monotonic()+600
        used = False
        while time.monotonic() < expires:
            while context.pending():
                context.iteration(False)
            incoming = commands.poll(.1)
            if not incoming:
                continue
            if len(incoming) != 1:
                raise ValueError('one command at a time required')
            command = incoming[0]
            if command == {'action': 'close'}:
                break
            # One predeclared stream per process. Never retry failed input here.
            if used or set(command) != {'action', 'output_scale', 'initial_position_verified'}:
                raise ValueError('expected one pointer stream command or close')
            if command['action'] != 'pointer_stream' or command['initial_position_verified'] is not True:
                raise ValueError('initial pointer position must be externally verified')
            scale = command['output_scale']
            if isinstance(scale, bool) or scale not in (1, 1.5, 2):
                raise ValueError('unsupported output scale')
            used = True
            start = time.monotonic_ns()
            record('stream_start', start_ns=start, command=command)
            for row in prepared_rows:
                while context.pending():
                    context.iteration(False)
                incoming = commands.poll()
                if incoming:
                    if incoming == [{'action': 'close'}]:
                        raise InterruptedError('stream canceled by controller')
                    raise ValueError('only close is allowed during a stream')
                due = start+row['offset_ns']
                delay = (due-time.monotonic_ns())/10**9
                if delay > 0:
                    time.sleep(delay)
                sent = time.monotonic_ns()
                if sent-due >= round(10**9/HZ):
                    record('deadline_missed', index=row['index'], due_ns=due, sent_ns=sent)
                    raise TimeoutError('input missed its scheduled interval; no catch-up or retry')
                dx, dy = (v/scale for v in row['physical_delta'])
                record('request_begin', index=row['index'], due_ns=due, ready_ns=sent, desktop_logical_delta=[dx, dy])
                sent = time.monotonic_ns()
                if sent-due >= round(10**9/HZ):
                    record('deadline_missed_before_call', index=row['index'], due_ns=due, observed_ns=sent)
                    raise TimeoutError('pre-dispatch logging exceeded input interval; no motion sent')
                call('NotifyPointerMotion', '(oa{sv}dd)', (session, {}, dx, dy), timeout_ms=100)
                record('request_returned', **row, due_ns=due, sent_ns=sent,
                       returned_ns=time.monotonic_ns(), desktop_logical_delta=[dx, dy])
                if time.monotonic_ns()-due >= round(10**9/HZ):
                    raise TimeoutError('call or receipt logging exceeded input interval; partial trial retained')
            delay = (start+DURATION_NS-time.monotonic_ns())/10**9
            if delay > 0:
                time.sleep(delay)
            record('stream_end', start_ns=start, count=COUNT)
            print(json.dumps({'stream_finished': True, 'native_delivery_qualified': False}), flush=True)
    except BaseException as exc:
        error = exc
        try:
            record('error', message=repr(exc))
        except BaseException:
            pass
    finally:
        # Evidence persistence must never skip revocation of input authority.
        for target, iface in ((pending, 'org.freedesktop.portal.Request'),
                              (session, 'org.freedesktop.portal.Session')):
            if target:
                try:
                    call('Close', None, None, target=target, iface=iface)
                except BaseException as exc:
                    error = error or exc
                    try:
                        record('close_error', target=target, message=repr(exc))
                    except BaseException:
                        pass
        connection.signal_unsubscribe(subscription)
        try:
            record('finished', success=error is None, qualification_pass=False)
        finally:
            records.close()
    if error:
        raise error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--plan', type=Path, help='write an offline schedule; no desktop input')
    mode.add_argument('--serve', type=Path, help='fresh output directory; requests native portal consent')
    args = parser.parse_args()
    if args.plan:
        with args.plan.open('x') as output:
            json.dump(declaration(), output, indent=2)
            output.write('\n')
    else:
        serve(args.serve)


if __name__ == '__main__':
    main()
