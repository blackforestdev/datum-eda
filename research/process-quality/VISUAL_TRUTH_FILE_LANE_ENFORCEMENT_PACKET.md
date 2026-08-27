# Visual-Truth File-Lane Enforcement Decision Packet

> **Status:** Exact mechanism proposed; owner review required before hook
> implementation.
>
> **Tracker:** `dat-visual-truth-file-lane-enforcement-jfl`.
>
> **Boundary:** This packet does not edit a prototype, enable the gate, add a
> dependency, or authorize any agent other than Claude to mutate visual truth.

<!-- REQ:VISUAL-TRUTH-FILE-LANE:LANE-C01 -->

## 1. Owner law already in force

The following policy is not a candidate:

1. Every `docs/gui/prototypes/*.html` file is Claude-owned visual truth.
2. Codex never edits those files, including mechanical link, annotation,
   terminology, formatting, disposition, rename, or deletion work.
3. Codex returns required changes to Claude as a reconciliation list.
4. An agent may not clear a gate it caused through an out-of-lane edit by
   refreshing digests, parity manifests, goldens, or other authority.
5. Commit `c2d1caa` remains accepted; this law applies prospectively.

`AGENTS.md` now states these hard rules beside explicit staging discipline.

## 2. Existing hook path

The repository uses `scripts/git-hooks/pre-commit`, enabled per clone with:

```text
git config core.hooksPath scripts/git-hooks
```

Today it executes only `python3 scripts/check_rustfmt.py --staged`. That gate is
correctly staged-only so another agent's unstaged work cannot block a commit.
The visual-truth lane check must preserve the same shared-worktree property.

## 3. Proposed marker contract

The exact proposed marker is:

```text
DATUM_VISUAL_TRUTH_LANE=claude
```

Rules:

- The value is exact and case-sensitive. Unset, empty, or any other value is
  unauthorized.
- The owner or Claude-specific launcher places it only in the Claude visual
  session environment. It is never stored in the repository, Git config, a
  shared shell profile, `.env`, or a wrapper used by every agent.
- Codex and every non-Claude lane are explicitly forbidden to set, copy,
  forward, request, or suggest this marker.
- The marker grants permission only to commit protected prototype HTML paths.
  It does not grant product authority, owner approval, dependency authority,
  permission to modify non-prototype files, or permission to refresh evidence
  without the normal full review.
- This is an operational capability marker, not a cryptographic identity
  proof. All agents run as the same operating-system user; pretending the
  environment string is an unforgeable security boundary would be false. Its
  strength comes from the owner-controlled launcher, the hard agent doctrine,
  staged refusal, commit audit, and the existing owner-only `--no-verify` rule.

## 4. Proposed protected-path semantics

The check protects every HTML path recursively below:

```text
docs/gui/prototypes/
```

The recursive interpretation prevents a future subdirectory from becoming an
unowned loophole. It covers staged additions, copies, modifications, mode
changes, deletions, and both sides of renames. A rename out of the lane still
touches the protected old path; a rename into it touches the protected new path.

The implementation will obtain staged status with a NUL-delimited Git command
equivalent to:

```text
git diff --cached --name-status -z --find-renames --diff-filter=ACMRD
```

It will parse status records rather than whitespace-splitting filenames. A Git
query or parse failure refuses the commit rather than silently passing.

## 5. Proposed check and hook integration

Add a standard-library-only script:

```text
python3 scripts/check_file_lane_ownership.py --staged
```

Behavior:

```text
no protected staged path
    -> pass; marker is irrelevant

protected staged path + exact marker
    -> pass and list the accepted protected paths

protected staged path + missing/wrong marker
    -> refuse; list every protected path, name the Claude lane,
       and direct the agent to unstage/revert its own edit and send a
       reconciliation list to Claude
```

`scripts/git-hooks/pre-commit` will run this check first, then the existing
staged rustfmt gate. It cannot use `exec` for the first command because both
checks must run:

```text
python3 scripts/check_file_lane_ownership.py --staged
exec python3 scripts/check_rustfmt.py --staged
```

The existing owner-only rule for `git commit --no-verify` remains the sole
emergency bypass. The new check adds no environment-variable bypass, allowlist,
path exemption, author-name heuristic, branch exception, or silent warning
mode.

## 6. Gate-integrity boundary

The hook examines staged paths only. It must not inspect or block on another
agent's dirty-but-unstaged prototype work, because shared-worktree ownership
cannot be inferred safely from file dirtiness.

Therefore the broader refresh-authority law remains an explicit agent duty:

- if an agent's own out-of-lane edit trips evidence, parity, golden, digest, or
  another gate, that agent reports the failure and hands the edit to Claude;
- it does not update `reviewed_digest`, parity manifests, generated authority,
  or blessed output to absorb the failure;
- Claude may refresh governed evidence only after making or accepting the
  authorized visual change and reviewing the complete affected route; and
- a digest refresh never retroactively legitimizes an unauthorized edit.

Trying to infer “whose unstaged edit caused this” inside the hook would create
false ownership claims and let one session block another. The hard doctrine,
staged refusal, explicit reconciliation handoff, and commit history are the
honest enforceable boundary.

## 7. Required proof

<!-- REQ:VISUAL-TRUTH-FILE-LANE:LANE-I01 -->

Implementation must add unit and temporary-repository integration tests proving:

1. unrelated staged files pass without the marker;
2. protected add, copy, modify, mode change, and delete refuse without it;
3. rename into and out of the protected lane refuse without it;
4. empty, case-changed, and arbitrary marker values refuse;
5. the exact marker permits the same protected staged cases;
6. spaces and non-ASCII filenames are parsed without ambiguity;
7. Git command/record parse failures fail closed;
8. the pre-commit hook runs the lane check before rustfmt;
9. the check never mutates the index or working tree; and
10. no prototype file is changed by the enforcement implementation commit.

The proof must run without Cargo, network access, or a new dependency.

## 8. Reconciliation-list contract

When Codex discovers a required prototype change, its handoff to Claude must
name:

- every exact prototype path;
- the anchor or visible region;
- the owner disposition or higher authority being reconciled;
- exact wording/behavior required and what must remain unchanged;
- gates/digests expected to become stale; and
- required render dimensions and visual inspection evidence.

Codex may subsequently review Claude's returned commit and refresh governance
only when that review itself is in lane and the prototype edit is already an
authorized Claude commit.

## 9. Recommendation

Approve this design. It is deliberately simple, fail-closed, dependency-free,
honest about the environment marker's limits, safe in the shared worktree, and
strict enough to stop the exact “mechanical Codex cleanup” failure mode.

<!-- OWNER:VISUAL-TRUTH-FILE-LANE:LANE-C02:LANE-C02 -->
<!-- REQ:VISUAL-TRUTH-FILE-LANE:LANE-C02 -->

## 10. Owner response

Reply exactly:

```text
VISUAL-TRUTH-LANE-DESIGN: approve
```

or:

```text
VISUAL-TRUTH-LANE-DESIGN: revise — <specific correction>
```

Approval authorizes only `LANE-I01`: implement and test the specified staged
hook without changing any prototype file.

<!-- EVIDENCE:VISUAL-TRUTH-FILE-LANE:LANE-C01-PACKET -->
