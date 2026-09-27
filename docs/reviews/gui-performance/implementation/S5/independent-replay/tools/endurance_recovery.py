"""Existing native recovery oracle shared with the combined endurance runner.
No standalone execution or acceptance claim.
"""
import re
import subprocess
import time

def move_and_ack(run, wid, x, y):
    for target_x in (x + 2, x):
        offset = run.log.stat().st_size
        run.xd('mousemove', '--window', wid, target_x, y)
        expected = f'window event WindowId({wid}) cursor moved {target_x:.2f},{y:.2f}'

        def acknowledged():
            with run.log.open() as f:
                f.seek(offset)
                return expected in f.read()
        run.until(acknowledged)

def recovery(run, host, sequence):
    assert int(run.xd('getwindowfocus')) == run.main
    row = {'host': host, 'sequence': sequence}
    run.report['recoveries'].append(row)
    run.save()
    wid = run.main
    if host != 'MAIN':
        move_and_ack(run, run.main, 148 if host != 'NEW' else 110, 16)
        run.xd('click', 1)
        if host != 'NEW':
            run.xd('key', '--delay', 20, 'Up', 'Right', *(['Down'] if host == 'PROJECT' else []))
        time.sleep(0.15)
        run.xd('key', 'Return')
        title = {'GLOBAL': 'Global Preferences', 'PROJECT': 'Project Preferences', 'NEW': 'New Project'}[host]
        wid = int(run.until(lambda: next((w for w in run.xd('search', '--all', '--onlyvisible', '--pid', run.p.pid, '--name', '^' + title).splitlines() if int(w) != run.main), None)))
        run.until(lambda: int(run.xd('getwindowfocus')) == wid)
        row['initial_pixels'] = run.pixels(wid, host, 1000 + sequence)
    row['window'] = wid
    move_and_ack(run, wid, 100, 100)
    run.wait_to(time.monotonic() + 5, 'pre-recovery-settle')
    before = run.OUT / f'{sequence:02d}-before.png'
    subprocess.run(['import', '-window', str(wid), str(before)], check=True, timeout=10)
    row['before_sha256'] = run.sha(before)
    row['begin_sample'] = run.sample('recovery-begin')
    pending = run.OUT / 'loss-request.pending'
    pending.write_text(str(sequence) + '\n')
    pending.replace(run.OUT / 'loss-request.txt')
    row['request_published_ns'] = time.monotonic_ns()
    move_and_ack(run, wid, 101, 100)
    move_and_ack(run, wid, 100, 100)

    def committed():
        text = run.log.read_text()
        assert 'native device replacement failed:' not in text, 'device replacement failed'
        return text.count('native device replacement committed') == sequence and text.count('native device state preserved workspace=true terminal_registry=true') == sequence and (f'native device loss request accepted sequence={sequence}\n' in text)
    run.until(committed)
    row['commit_observed_sample'] = run.sample('recovery-commit-observed')
    attempts = []
    row['recovery_pixels'] = attempts
    for attempt in range(15):
        after = run.OUT / f'{sequence:02d}-after-{attempt}.png'
        subprocess.run(['import', '-window', str(wid), str(after)], check=True, timeout=10)
        compared = subprocess.run(['compare', '-metric', 'AE', str(before), str(after), 'null:'], capture_output=True, text=True, timeout=10)
        attempts.append({'path': after.name, 'ae': compared.stderr, 'returncode': compared.returncode, 'ns': time.monotonic_ns()})
        if compared.returncode not in (0, 1):
            raise RuntimeError(('recovery comparison infrastructure error', compared.returncode, compared.stderr))
        if compared.returncode == 0:
            break
        time.sleep(0.025)
    assert attempts[-1]['returncode'] == 0, ('recovery pixel mismatch', sequence)
    assert run.sha(before) == row['before_sha256']
    assert int(run.xd('getwindowfocus')) == wid, 'focus changed across device recovery'
    text = run.log.read_text()
    assert text.count('native device loss injected') == sequence
    epochs = re.findall('native device epoch replaced epoch=(\\d+)', text)
    assert len(epochs) == sequence and len(set(epochs)) == sequence
    row['device_epoch'] = int(epochs[-1])
    row['end_sample'] = run.sample('recovery-ready')
    if host != 'MAIN':
        run.xd('key', 'Escape')
        run.until(lambda: int(run.xd('getwindowfocus')) == run.main)
        run.until(lambda: subprocess.run(['xwininfo', '-id', str(wid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=3).returncode != 0)
    row['completed'] = True
    run.save()
    print(f'{sequence}/20 device recoveries complete ({host})', flush=True)


def schedule():
    """270 window cycles and20 recoveries, evenly interleaved over one hour."""
    after = {sequence * 270 // 20: sequence for sequence in range(1, 21)}
    events = []
    for cycle in range(270):
        events.append(("window", ("GLOBAL", "PROJECT", "NEW")[cycle % 3], cycle))
        if cycle + 1 in after:
            sequence = after[cycle + 1]
            events.append(("recovery", ("MAIN", "GLOBAL", "PROJECT", "NEW")[(sequence - 1) % 4], sequence))
    return tuple((index * 3600 / len(events), *event) for index, event in enumerate(events))
