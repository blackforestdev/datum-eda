"""Render exact owner instructions; preparation never executes these commands."""

from shlex import quote


def promotion_markdown(root, result):
    candidate, base = result["candidate"], result["base"]
    runner, hooks = quote(result["runner"]), quote(result["hooks"])
    environment = quote(result["environment"])
    check = (f"python3 {runner} --root {quote(str(root))} --enforce "
             f"--authority-ref {candidate} --base-ref {base} "
             f"--environment-path {environment}")
    return f"""# Owner-only activation handoff

Preparation succeeded; promotion has NOT been performed. This local candidate
reference retains an object, not authority. Your existing ACCEPT and two DEFER
responses remain valid. This transaction does not authorize Preferences GP-CM05.

Candidate: `{candidate}`
Comparison base: `{base}`
Packet: `{result['packet_sha256']}`
Review: `{result['review_sha256']}`

Review first: inspect the six-file proposal and the external hook. The only
predecessor change removes this pilot from its remaining-unblocks lists. Other
work must stay paused during publication. Do not use an older preparation if
HEAD moved; regenerate against a fresh coordinated base instead of forcing it.

```bash
git -C {quote(str(root))} diff {base} {candidate} --stat
git -C {quote(str(root))} diff {base} {candidate}
```

Only the project owner may select this candidate as trusted authority and run
the following block after review. The first validator invocation is read-only.
Publication/configuration happen only if it passes. Any failure stops the block;
do not bypass a refusal or infer that a partially completed block activated the
gate. Keep this external directory available: the hook depends on its pinned
runner. Do not modify that runner or move/delete it after installation.

```bash
(
set -euo pipefail
cd {quote(str(root))}
test "$(git rev-parse HEAD)" = {base}
test -z "$(git status --porcelain)"
{check} --candidate-ref {candidate}
test "$(git rev-parse HEAD)" = {base}
test -z "$(git status --porcelain)"
git merge --ff-only {candidate}
git config --local datum.workflowDeliveryAuthorityRef {candidate}
git config --local datum.workflowDeliveryBaseRef {base}
git config --local datum.workflowDeliveryEnvironmentPath {environment}
git config --local datum.workflowDeliveryRunnerPath {runner}
git config --local core.hooksPath {hooks}
{check} --staged
{hooks}/pre-commit
python3 scripts/project_status.py check
)
```

Return the candidate/base IDs and the complete verification output, including
any failure. G06 is not reported as operationally complete until publication,
configuration and blocking verification all succeed. No further enrollment,
product acceptance, or successor selection is implied. Earlier incomplete status
notes in the handoff are historical only after this transaction succeeds.
"""
