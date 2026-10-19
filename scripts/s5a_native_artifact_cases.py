"""Bounded canonical Proposal creation/Undo/Redo through public CLI and native input.

No saved Review artifact is supplied to the GUI. Producer IDs and geometry come
from the public creation/preview replies; current-frame input plans are reviewed
outside the running application. Operational credential bytes are never read.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import traceback

from native_project_source_inventory import canonical_source_snapshot, digest
from prove_native_project_startup_gui import Atspi, StartupQA
from s5a_native_selection_observation import latest_state, qualify, selection_snapshot

ROOT = Path(__file__).resolve().parents[1]
OWNED = ROOT / '.datum/scratch/s5a-c04-managed-20261018'
QUALITY = ROOT / 'crates/test-harness/testdata/quality'
BASELINE = QUALITY / 'route_strategy_curated_baseline_v1/same-outcome-default'
SHARDS = {
    'project.json': 'cf7f4b611baf58289276a0197d594e41eb7939f29aba7e63e0c06998bd2214fb',
    'board/board.json': 'b92efd634e346eb88a070843add8ad635abf77002a4b1abfc70f9e9f02ccf746',
    'schematic/schematic.json': 'e4d03de428a29e826d16324e688d1de2e88e5900a2a6161ea1e6bb63a9214ba9',
    'rules/rules.json': 'db2f5959129d9b7073e8a145773e7c74f76eb4a98e6455288601118e0052794f',
}
BINARIES = {
    'datum-gui': 'f991d8b75e841ad0b31d0ac806cb26c876680fbde97891fdaeb236aab5bb53f8',
    'datum-eda': 'be312f69d6ef3b936f0dc3af6975603200438640a6782a784479f0e513126da9',
    'eda-engine-daemon': 'e58d388d0e7d845d53344e8ac62627b95eff5b1ba692e6dc486621a482c5d91f',
}


def complete_states(text, after=0):
    """Read only complete LF-terminated records from the existing v2 stream."""
    latest_state(text)  # Validate stream version, completeness and ordering.
    rows = []
    for line in text.splitlines(keepends=True):
        if line.startswith('DATUM_ACTION_EVIDENCE ') and line.endswith('\n'):
            row = json.loads(line.removeprefix('DATUM_ACTION_EVIDENCE '))
            if row['event'] == 'state' and row['sequence'] > after:
                rows.append(row)
    return rows


def save_reply(text, after):
    """A published terminal reply after this input's Applying record is required."""
    applying = None
    replies = {'Project saved',
               'Project history changed since the edit began; refresh before editing',
               'Project changed since the edit began; refresh before editing',
               'Save not confirmed: Project integrity needs recovery',
               'Edit completed but the Project changed during refresh; reopen before editing',
               'Project changed during selection preparation; reopen before editing',
               'Edit worker stopped; reopen the Project before editing'}
    for row in complete_states(text, after):
        message = row['value'].get('console_latest')
        if message == 'Applying Project action':
            applying = row['sequence']
        elif applying is not None and message in replies:
            return {'applying_sequence': applying, 'reply_sequence': row['sequence'],
                    'message': message, 'terminal_state': row}
    return False


def prepare_input(run):
    """Copy exactly four governed tracked shards, never an operational lock."""
    manifest = QUALITY / 'native_project_validation_manifest_v1.json'
    assert digest(manifest) == '2f04284e0cc52e4571432a2ee7cc6e6377f7df74ea4d9916be1ccbe5f23af596'
    rows = [r for r in json.loads(manifest.read_text())['fixtures']
            if r['fixture_id'] == 'same-outcome-default']
    assert len(rows) == 1 and rows[0]['expected_valid'] is True
    assert ROOT / rows[0]['project_root'] == BASELINE
    tracked = subprocess.run(['git', 'ls-files', '--', str(BASELINE)], cwd=ROOT,
                             text=True, capture_output=True, check=True).stdout.splitlines()
    assert {str((ROOT / p).relative_to(BASELINE)) for p in tracked} == set(SHARDS)
    project = run / 'project'
    for relative, expected in SHARDS.items():
        source = BASELINE / relative
        assert not source.is_symlink() and digest(source) == expected
        target = project / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    assert canonical_source_snapshot(project)['canonical_files'] == SHARDS
    requests = QUALITY / 'route_strategy_curated_baseline_v1/route-strategy-batch-requests.json'
    request = [r for r in json.loads(requests.read_text())['requests']
               if r['request_id'] == 'same-outcome-default']
    assert len(request) == 1 and ROOT / request[0]['project_root'] == BASELINE
    return project, {'manifest_sha256': digest(manifest), 'request_sha256': digest(requests),
                     'request': request[0], 'source': str(BASELINE), 'shards': SHARDS}


def execute(qa, project, result):
    def checkpoint(label):
        snapshot = canonical_source_snapshot(project)
        assert {p: snapshot['canonical_files'].get(p) for p in SHARDS} == SHARDS
        assert {p: digest(BASELINE / p) for p in SHARDS} == SHARDS
        result.setdefault('checkpoints', {})[label] = snapshot
        return snapshot['canonical_files']

    def cli(label, *args, group='project'):
        argv = [str(qa.binary_dir / 'datum-eda'), '--format', 'json', group, *args]
        command = subprocess.run(argv, cwd=qa.root, env=qa.env, text=True,
                                 capture_output=True, timeout=30)
        entry = {'argv': argv, 'exit_code': command.returncode,
                 'stderr': command.stderr, 'stdout': command.stdout}
        result.setdefault('public_project_cli', {})[label] = entry
        assert command.returncode == 0, (label, command.stderr)
        return json.loads(command.stdout)

    def tip():
        transactions = [json.loads(line) for line in
                        (project / '.datum/journal/transactions.jsonl').read_text().splitlines()]
        assert transactions
        return transactions[-1]

    checkpoint('copied-four-shards')
    request = result['governed_input']['request']
    artifact = qa.root / 'public-route-proposal.json'
    cli('export', 'export-route-proposal', str(project), '--net', request['net_uuid'],
        '--from-anchor', request['from_anchor_pad_uuid'], '--to-anchor', request['to_anchor_pad_uuid'],
        '--profile', 'default', '--out', str(artifact))
    batch = json.loads(artifact.read_text())['proposal']['batch']
    batch_path = qa.root / 'unchanged-exported-batch.json'
    batch_path.write_text(json.dumps(batch, indent=2) + '\n')
    assert json.loads(batch_path.read_text()) == batch
    result['exported_batch'] = {'artifact_sha256': digest(artifact),
                              'batch_sha256': digest(batch_path), 'batch': batch}
    checkpoint('after-public-export-before-creation')
    created = cli('create', 'create', str(project), '--batch', str(batch_path),
                  '--rationale', 'C04 canonical producer lifetime proof', '--source', 'cli', group='proposal')
    proposal = created['proposal']
    assert proposal['batch'] == batch and created.get('source_profile_adoption_transaction') is None
    identity = {k: proposal[k] for k in ['project_id', 'proposal_id', 'prepared_against']}
    identity['batch_id'] = batch['batch_id']
    assert identity['project_id'] == created['project_id']
    assert identity['proposal_id'] == created['proposal_id']
    assert identity['project_id'] == json.loads((project / 'project.json').read_text())['uuid']
    metadata = project / '.datum/proposals' / (identity['proposal_id'] + '.json')
    assert json.loads(metadata.read_text()) == proposal
    original_metadata_hash = digest(metadata)
    creation_tip = tip()
    operations = creation_tip['operations']
    assert len(operations) == 1 and operations[0]['kind'] == 'create_proposal_metadata'
    assert operations[0]['proposal_id'] == identity['proposal_id']
    assert operations[0]['proposal'] == proposal
    assert creation_tip['after_model_revision'] == created['model_revision']
    result['identity'] = identity
    result['creation_tip'] = creation_tip
    created_map = checkpoint('after-public-creation')
    preview = cli('preview', 'preview', str(project), '--proposal', identity['proposal_id'], group='proposal')
    assert preview['proposal_id'] == identity['proposal_id'] and preview['project_id'] == identity['project_id']
    assert preview['model_revision'] == created['model_revision']
    assert preview['prepared_against'] == identity['prepared_against']
    assert preview['render_deltas'], 'real public preview geometry required'
    (qa.root / 'public-preview.json').write_text(json.dumps(preview, indent=2) + '\n')
    assert checkpoint('after-public-preview') == created_map
    qa.start_envelope()
    main = qa.launch('proposal-creation-undo', project_root=project)
    state = lambda: latest_state((qa.root / 'proposal-creation-undo.log').read_text())
    qa.wait('current native state', state)
    qa.wait_presented()
    if state()['value']['focused_content'] != 'Some(Board)':
        qa.key(main, 'Tab')
    qa.wait('ordinary Board focus', lambda: state()['value']['focused_content'] == 'Some(Board)')
    qa.key(main, 'Escape')
    qa.key(main, 'f')
    qa.wait_presented()
    assert state()['value']['selection']['kind'] == 'none'
    assert checkpoint('after-gui-startup-and-fit') == created_map
    result['initial_state'] = state()
    result['initial_capture'] = qa.capture('proposal-current-frame', main)
    plan_path = qa.root / 'current-frame-proposal-plan.json'
    plan = qa.wait('source/preview/current-frame finite Proposal point plan', lambda:
                   json.loads(plan_path.read_text()) if plan_path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'proposal-current-frame.png')
    assert plan['preview_sha256'] == digest(qa.root / 'public-preview.json')
    assert plan['identity'] == identity and plan['basis'] and plan['transform']
    assert len(plan['screen_px']) == 2 and all(type(v) is int for v in plan['screen_px'])
    result['point_plan'] = plan
    revision = created['model_revision']
    wanted = {'project': identity['project_id'], 'revision': revision, 'kind': 'proposal',
              'focus': None, 'scope': None, 'members': [], 'pid': qa.process.pid,
              'oracle': 'Real public canonical Proposal create tuple and preview; zero authored members'}

    def public_read(kind):
        record = state()
        assert record['value']['selection']['kind'] == kind
        nodes = qa.nodes()
        root = next((n for n in nodes if n.get_accessible_id() ==
                     '/org/a11y/atspi/accessible/selection'), None)
        fields = {}
        prefix = '/org/a11y/atspi/accessible/selection/fields/n'
        for node in nodes:
            if node.get_accessible_id().startswith(prefix):
                name = bytes.fromhex(node.get_accessible_id()[len(prefix):]).decode()
                fields[name] = qa.attributes(node)
        observed = {'state': record, 'fields': fields,
                    'root': qa.attributes(root) if root else None,
                    'root_standard_selected': root.get_state_set().contains(Atspi.StateType.SELECTED) if root else False}
        if kind == 'proposal':
            observed['qualified'] = qualify(record, selection_snapshot(qa, Atspi.StateType.SELECTED), wanted)
            # Standard SELECTED is declared only on authored member nodes.
            # Proposal has no such members; its typed subject/producer proves it.
            assert observed['root_standard_selected'] is False
            assert fields['producer_identity']['value-state'] == 'common'
            assert json.loads(fields['producer_identity']['canonical-value']) == identity
            assert record['value']['selection']['related_count'] == 0
        else:
            assert record['value']['selection']['member_count'] == 0
            assert observed['root_standard_selected'] is False
            if root:
                assert observed['root']['subject-kind'] == 'none'
                assert selection_snapshot(qa, Atspi.StateType.SELECTED)['count'] == 0
        assert state()['sequence'] == record['sequence'], 'public read raced native state'
        return observed

    def settled(kind):
        try:
            return public_read(kind)
        except (AssertionError, KeyError, ValueError) as error:
            result['pending_public_read'] = str(error)
            return False

    def context(label, kind):
        before_map = checkpoint(label + '-context-before')
        before = public_read(kind)
        argv = [str(qa.binary_dir / 'datum-eda'), '--format', 'json', 'context', 'get',
                '--project-root', str(project)]
        env = dict(qa.env)
        for key in ['DATUM_DISCOVERY', 'DATUM_TERMINAL_CONTEXT']:
            env.pop(key, None)
        query = subprocess.run(argv, cwd=qa.root, env=env, text=True,
                               stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=15)
        entry = {'argv': argv, 'exit_code': query.returncode,
                 'discarded': 'Transient raw envelope stdout/stderr; only selection_context persisted.'}
        result.setdefault('selection_context_get', {})[label] = entry
        assert query.returncode == 0
        selected = json.loads(query.stdout)['selection_context']
        entry['selection_context'] = selected
        if kind == 'proposal':
            assert selected['kind'] == kind
            details = selected['details']
            assert details['project'] == identity['project_id'] and details['revision'] == revision
            assert details['subject']['kind'] == 'proposal'
            assert json.loads(details['subject']['identity']) == identity
            assert details['focus'] is None and details['scope'] is None
            assert details['members'] == [] and details['total'] == 0 and details['stale'] is False
            assert details['inspection']['producer_identity'] == {'state': 'common', 'value': identity}
            assert details['truncated'] is False and not details.get('omitted_inspection_fields', [])
        else:
            assert selected is None or selected['kind'] == 'none'
        after = public_read(kind)
        assert before == after
        assert checkpoint(label + '-context-after') == before_map
        entry.update(before=before, after=after)

    def acquire(label):
        before_map = checkpoint(label + '-input-before')
        before_selection = state()['value']['selection']
        camera = {k: state()['value'][k] for k in plan['camera_basis']}
        assert camera == plan['camera_basis'], 'same finite point requires unchanged camera/layout'
        qa.focus(main)
        diagnostic = Path(qa.env['DATUM_GUI_LOG'])
        offset = len(diagnostic.read_text())
        x, y = plan['screen_px']
        # An unchanged pointer coordinate emits no native motion. Reset one
        # pixel without a button, then return to the one reviewed hit point.
        qa.command('xdotool', 'mousemove', '--window', main, str(x + 1), str(y))
        qa.wait('non-click one-pixel pointer reset delivered', lambda:
                f'cursor moved {x + 1:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
        assert state()['value']['selection'] == before_selection
        assert {k: state()['value'][k] for k in plan['camera_basis']} == camera
        assert checkpoint(label + '-nonclick-reset') == before_map
        offset = len(diagnostic.read_text())
        qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
        qa.wait('actual Proposal point delivery', lambda:
                f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
        qa.command('xdotool', 'click', '1')
        result[label] = qa.wait('whole real canonical Proposal selected', lambda: settled('proposal'))
        assert checkpoint(label + '-input-after') == before_map
        qa.capture(label, main)
        context(label, 'proposal')

    def refresh(label, kind, need_cause=False):
        before_map = checkpoint(label + '-refresh-before')
        before = state()['sequence']
        diagnostic = Path(qa.env['DATUM_GUI_LOG'])
        offset = len(diagnostic.read_text())
        qa.key(main, 'ctrl+s')
        reply = qa.wait('actual post-input terminal Save success/refusal publication', lambda:
                        save_reply((qa.root / 'proposal-creation-undo.log').read_text(), before))
        result.setdefault('ordinary_refresh_replies', {})[label] = reply
        observed = qa.wait('explicit ordinary Save refresh ' + label, lambda:
                           settled(kind) if state()['sequence'] >= reply['reply_sequence'] else False)
        result.setdefault('ordinary_refresh', {})[label] = observed
        result.setdefault('refresh_worker_diagnostics', {})[label] = diagnostic.read_text()[offset:]
        assert checkpoint(label + '-refresh-after') == before_map
        if need_cause:
            records = []
            for row in complete_states((qa.root / 'proposal-creation-undo.log').read_text(), before):
                message = row['value'].get('console_latest')
                notice = row['value']['selection'].get('notice')
                if message or notice:
                    records.append({'sequence': row['sequence'], 'console_latest': message, 'notice': notice})
            result['removal_cause_observations'] = records
            assert any('artifact' in json.dumps(r).lower() and
                       ('invalidat' in json.dumps(r).lower() or 'disappear' in json.dumps(r).lower()
                        or 'no longer' in json.dumps(r).lower()) for r in records), 'truthful actual removal cause required'
        qa.capture(label, main)
        context(label, kind)

    def journal_mutation(action, expected_tip, model_revision):
        assert tip() == expected_tip, 'exact current journal transaction guard'
        journal_count = len((project / '.datum/journal/transactions.jsonl').read_text().splitlines())
        reply = cli(action, action, str(project), '--expected-model-revision', model_revision,
                    '--expected-tip-transaction', expected_tip['transaction_id'])
        assert reply['status'] == 'applied' and reply['project_id'] == identity['project_id']
        assert reply['guard']['checked'] is True
        assert reply['guard']['current_tip_transaction'] == expected_tip['transaction_id']
        assert reply['before_model_revision'] == model_revision
        assert reply['transaction'] == tip()
        assert reply['journal_len'] == journal_count + 1
        operations = reply['transaction']['operations']
        assert len(operations) == 1
        assert operations[0]['kind'] == ('delete_proposal_metadata' if action == 'undo' else 'create_proposal_metadata')
        assert operations[0]['proposal_id'] == identity['proposal_id']
        assert operations[0]['proposal'] == proposal
        checkpoint('after-public-' + action)
        return reply

    acquire('canonical-proposal-selected')
    refresh('selected-harmless-save', 'proposal')
    undone = journal_mutation('undo', creation_tip, revision)
    assert not metadata.exists(), 'public creation Undo must actually remove its producer'
    revision = undone['after_model_revision']
    wanted['revision'] = revision
    refresh('creation-undone-selection-none', 'none', need_cause=True)
    redone = journal_mutation('redo', undone['transaction'], revision)
    assert metadata.is_file() and digest(metadata) == original_metadata_hash
    revision = redone['after_model_revision']
    wanted['revision'] = revision
    refresh('creation-redone-no-resurrection', 'none')
    acquire('same-tuple-explicit-reacquisition')
    before_exit = checkpoint('before-terminal-exit')
    qa.terminal_dock_open = state()['value']['dock'] != 'None'
    qa.terminal_command(main, 'exit')
    qa.wait('owned terminal PTYs closed before GUI end', qa.terminal_ptys_closed)
    assert checkpoint('after-terminal-exit') == before_exit
    result['terminal_cleanup'] = 'Ordinary shell exit and observed PTY closure before final GUI SIGTERM.'
    result['final_selection_canonical_files'] = before_exit


def run_proposal_creation_undo(run_dir, close_failed_transport, close_owned_companions):
    run = run_dir.absolute()
    if run.resolve() != run or run == OWNED or not run.is_relative_to(OWNED) or run.exists():
        raise ValueError('run-dir must be a fresh unaliased child of owned C04 scratch')
    runtime_parent = Path(os.environ['XDG_RUNTIME_DIR'])
    info = runtime_parent.stat()
    if (not runtime_parent.is_absolute() or runtime_parent.is_symlink()
            or info.st_uid != os.getuid() or info.st_mode & 0o077):
        raise ValueError('existing user IPC runtime must be private and owned')
    run.mkdir(parents=True)
    project, governed = prepare_input(run)
    qa = StartupQA(run, runtime_parent=runtime_parent)
    # This runner owns removal explicitly after process-stop qualification.
    # Disable TemporaryDirectory's automatic/atexit removal on failed shutdown.
    qa.runtime._finalizer.detach()
    result = {'status': 'FAIL', 'case': 'canonical-proposal-creation-undo', 'project': str(project),
              'governed_input': governed, 'binary_sha256': qa.binary_hashes,
              'limits': 'Canonical Proposal producer removal by creation Undo only. No literal discard, Accepted Review withdrawal, full N10/C04/C05/C06/HUMAN or ordinary window-Close acceptance.',
              'ipc_runtime': {'path': qa.runtime.name, 'mode': oct(Path(qa.runtime.name).stat().st_mode & 0o777)}}
    for key in list(qa.env):
        if key.startswith(('DATUM_', 'WGPU_', 'VK_')) or key in ['EDA_CLI_BIN', 'EDA_ENGINE_SOCKET']:
            qa.env.pop(key)
    qa.env.update({'DATUM_GUI_VERBOSE_LOG': '1', 'DATUM_ACTION_EVIDENCE': '1',
                   'DATUM_TRACE_CLICKS': '1', 'EDA_CLI_BIN': str(qa.binary_dir / 'datum-eda'),
                   'DATUM_GUI_PREFERENCES_PATH': str(run / 'config/gui-preferences.json')})
    try:
        assert qa.binary_hashes == BINARIES, 'reviewed current executable cohort required'
        result['source_sha256'] = {name: digest(ROOT / 'scripts' / name) for name in [
            'prove_s5a_native_selection.py', 's5a_native_artifact_cases.py',
            'native_project_source_inventory.py', 's5a_native_selection_observation.py',
            'prove_native_project_startup_gui.py']}
        execute(qa, project, result)
        result['status'] = 'CAPTURE_ONLY_CANONICAL_PROPOSAL_CREATION_UNDO_REVIEW_PENDING'
    except Exception as error:
        result['error'] = str(error)
        (run / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        result['gui_pid'] = qa.process.pid if qa.process else None
        result['compositor_pid'] = qa.compositor.pid if qa.compositor else None
        teardown(qa, project, result, close_failed_transport, close_owned_companions)


def teardown(qa, project, result, close_failed_transport, close_owned_companions):
    """Attempt each owned shutdown phase; retain IPC if any owner remains live."""
    errors = {}
    def attempt(label, operation):
        try:
            operation()
            return True
        except Exception as error:
            errors[label] = f'{type(error).__name__}: {error}'
            return False
    app, compositor, clipboard = qa.process, qa.compositor, qa.clipboard_writer
    descendants = {}
    def children(pid):
        try:
            for child in Path(f'/proc/{pid}/task/{pid}/children').read_text().split():
                stat = Path(f'/proc/{child}/stat').read_text().rsplit(')', 1)[1].split()
                descendants[child] = stat[19]  # PID reuse cannot qualify an old owner as live.
                children(child)
        except (FileNotFoundError, ProcessLookupError):
            pass
    try:
        if app:
            attempt('snapshot-owned-descendants', lambda: children(app.pid))
        attempt('failed-transport', lambda: close_failed_transport(qa, result))
        attempt('gui-stop', qa.stop)
        companions_stopped = attempt('owned-companions', lambda: close_owned_companions(qa, result))
        for label, process in [('clipboard', clipboard), ('compositor', compositor)]:
            def stop(process=process):
                if process and process.poll() is None:
                    process.terminate()
                    process.wait(timeout=10)
            attempt(label + '-stop', stop)
        if hasattr(qa, 'compositor_log'):
            attempt('compositor-log-close', qa.compositor_log.close)
        def qualify_stopped():
            assert all(p is None or p.poll() is not None for p in [app, compositor, clipboard])
            for pid, start in descendants.items():
                try:
                    current = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
                    assert current[19] != start, 'owned descendant still present: ' + pid
                except (FileNotFoundError, ProcessLookupError):
                    pass
            assert companions_stopped, 'owned companion shutdown was not qualified'
            assert 'snapshot-owned-descendants' not in errors
        stopped = attempt('owned-process-stop-qualification', qualify_stopped)
        result['ipc_runtime']['owners_stopped'] = stopped
        result['owned_descendant_start_ticks'] = descendants
        if stopped:
            attempt('ipc-remove', qa.runtime.cleanup)
        else:
            result['ipc_runtime']['retained_reason'] = 'Owned process-stop qualification failed; automatic cleanup disabled.'
        if qa.previous_focus:
            attempt('restore-outer-focus', lambda: qa.command('xdotool', 'windowactivate', qa.previous_focus, check=False))
        def canonical_check():
            snapshot = canonical_source_snapshot(project)
            result['after_teardown'] = snapshot
            assert {p: snapshot['canonical_files'].get(p) for p in SHARDS} == SHARDS
            if 'final_selection_canonical_files' in result:
                assert snapshot['canonical_files'] == result['final_selection_canonical_files']
        attempt('canonical-after-teardown', canonical_check)
    finally:
        if errors:
            result['cleanup_errors'] = errors
            result['status'] = 'FAIL'
        result['ipc_runtime']['removed_after_process_stop'] = not Path(qa.runtime.name).exists()
        result['actions'] = qa.actions
        result['native_observations'] = qa.observations
        (qa.root / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    if errors and 'error' not in result:
        raise RuntimeError('Owned cleanup failed; see persisted cleanup_errors')
