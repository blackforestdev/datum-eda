"""Retain command output and failure observations without overwriting earlier proof."""
from pathlib import Path
import subprocess
import os
import signal

from .inputs import canonical, digest, require


class Capture:
    def __init__(self, root, inventory=()):
        self.root = Path(root).absolute()
        require(self.root == self.root.resolve() and self.root.is_dir(), 'capture root must be a real directory')
        self.inventory = {r['path']: r for r in inventory}

    def put(self, raw):
        if not isinstance(raw, bytes):
            raw = canonical(raw)
        sha = digest(raw)
        directory = self.root / 'command-capture'
        directory.mkdir(exist_ok=True)
        require(not directory.is_symlink(), 'capture directory is redirected')
        path = directory / sha
        try:
            with path.open('xb') as stream:
                stream.write(raw)
        except FileExistsError:
            from .inputs import read_file
            require(read_file(self.root, 'command-capture/' + sha) == raw, 'existing capture differs')
        reference = {'path': 'command-capture/' + sha, 'sha256': sha}
        self.inventory[reference['path']] = reference
        return reference

    def command(self, argv, *, cwd, candidate, inputs, timeout=3600):
        """Timeouts and failed commands remain evidence; callers cannot turn them into pass."""
        try:
            with subprocess.Popen(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  start_new_session=True,
                                  env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}) as process:
                try:
                    stdout, stderr = process.communicate(timeout=timeout)
                    code = process.returncode
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    stdout, stderr = process.communicate()
                    code = 124
                    stderr += b'\nProof command timed out.\n'
        except OSError as error:
            code, stdout, stderr = 127, b'', str(error).encode()
        record = {'schema': 'datum.preferences.command-observation.v2', 'command': argv,
                  'exit_code': code, 'candidate_revision': candidate, 'input_manifest_sha256': inputs,
                  'executed_tests': executed_tests(stdout + b'\n' + stderr),
                  'stdout': self.put(stdout), 'stderr': self.put(stderr)}
        return {k: record[k] for k in ('command', 'exit_code', 'candidate_revision', 'input_manifest_sha256')} | {
            'log': self.put(record)}


def executed_tests(raw):
    """Parse completed test-run summaries, not test discovery or printed function names."""
    import re
    output = raw.decode('utf-8', 'replace')
    rust = [int(n) for n in re.findall(r'test result: ok\. (\d+) passed;', output)]
    python = [int(n) for n in re.findall(r'^Ran (\d+) tests? in ', output, re.MULTILINE)]
    return sum(rust) + sum(python)
