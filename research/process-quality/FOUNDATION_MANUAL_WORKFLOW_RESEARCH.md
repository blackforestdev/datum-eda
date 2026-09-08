# Bounded manual-workflow contract basis

Status: WDQ-F02 planning evidence; proposed choices are not ratified.
Owning route: `foundation-manual-workflow`.
Baseline: `84d32fd3`; issue `dat-manual-foundation-contracts-fsw`.

## Reviewed internal authority

WDQ-F01 completed the process, settings, selection and editor-shell routes and
the applicable mutation, storage, identity and library review. The durable
clause classification is `docs/reviews/workflow-foundation/authority-review.md`.
PM001/002/018/019/023/026 and accepted Units authority supply the existing
mutation, genesis, selection and exact-entry rules. PM034–040 govern the
separation of technical history, optional human Revision, Project settings and
machine defaults. PM008/008A provide the library binding requirements and
explicit unresolved library behavior. PM012's status was clarified in
`4e519e0c`, without choosing fixture budgets.

The companion contract connects these obligations; it does not replace their
owning sources or extend pilot acceptance. Existing consumer plans now expose
the planning, execution and acceptance boundaries. A native doorway is still
tracked by `dat-native-project-startup-vrf`, not proven implemented here.

## Primary research: Project writer ownership and persistence

Consulted the Linux man-pages project on 2026-09-08 UTC. These sources describe
operating-system primitives, not a ready-made Datum locking design.

- `flock` permits nonblocking exclusive acquisition and associates the lock
  with the open file description. Duplicated descriptors share it; release
  occurs on explicit unlock or closure of all descriptors. Local advisory
  locks do not stop writers that ignore them, and network filesystem semantics
  vary. [flock(2)](https://man7.org/linux/man-pages/man2/flock.2.html)
- Flushing a file does not by itself guarantee persistence of its directory
  entry; directory synchronization is separate. Synchronization can fail,
  including from I/O errors or exhausted storage.
  [fsync(2)](https://man7.org/linux/man-pages/man2/fsync.2.html)
- Ordinary rename can replace an existing destination. No-replace rename
  requests reject an existing destination, but filesystem support matters.
  [rename(2)](https://man7.org/linux/man-pages/man2/rename.2.html)

Inference for Datum: all legitimate writers and recovery must participate in
one protocol, and write ownership cannot be established solely by a PID file
or a read-then-write journal-tip check. Creation must not overwrite a Project
that appeared after preflight. A successful rename is not complete crash proof.
No syscall wrapper, third-party dependency or concrete lockfile schema is
introduced by this research.

## Recommended bounded decision, still pending

For the first local-filesystem workflow, prefer an engine-owned exclusive
Project guard used by every commit/recovery path, with GUI writes through the
daemon and headless writers acquiring the same guard. This keeps manual and
headless surfaces on one protocol without requiring a permanently running
daemon merely to open a CLI workflow. Alternative: daemon-only ownership with
explicit client connection and daemon lifecycle rules. Neither is ratified by
this report; `dat-project-write-ownership-lock-0ne` requires a numbered decision.

Before that decision closes, specify lock-target stability, descriptor
inheritance, canonical project identity/path aliases, unsupported filesystems,
second-writer refusal, stale owner display and interrupted recovery. Do not
delete a live lock based on age or PID text. The first proof envelope should
name its local filesystem explicitly; remote/shared filesystem support must
not be inferred from a local passing test. This is a proposed proof boundary,
not a permanent product exclusion.

## Other unresolved choices and evidence limits

The contract proposes a no-argument native doorway and an exact edit fixture;
neither is an existing runtime observation. Wire preview/finish/Escape and
native T/crossing formation still need explicit disposition. The recommended
bounded gesture is one uncommitted chain followed by one atomic commit, but
it must not silently supersede the schematic contract's ambiguous Escape text.
Numeric values in the contract are test inputs/oracles, not an authorized
fixture build or claims about an existing library package.

Performance thresholds, visual doorway design, writer mechanism and applicable
prerequisite changes remain owner boundaries. The WDQ-F03 packet must identify
them individually; broad adoption language must not convert them into approval.
