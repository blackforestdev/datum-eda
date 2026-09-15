# Datum-Owned Terminal Core Research Baseline

Status: governed research baseline

Authority: Product Mechanics 027, 029, and 030. This document records
requirements and provenance; it is not permission to copy an upstream terminal
implementation.

## 1. Scope correction

Zig supplied no terminal behavior. It was only the compiler used by the
reverted Ghostty build. Datum does not need a Zig replacement. The replacement
is a Datum-owned Rust terminal engine and Linux PTY/session layer built with the
existing approved toolchain.

Ghostty was never linked into Datum, so no Ghostty runtime feature was removed.
Datum retains a meaningful provisional parser, input encoder, focus authority,
session registry, Linux PTY implementation, and responsive output path. The
missing work is the mature cell/state/history/reflow/protocol engine and its
production proof.

## 2. Clean-room research rule

- Normative standards and protocol-owner documentation are requirements input.
- Other terminals may be run as black-box behavioral references.
- Upstream terminal source, internal tests, generated tables, algorithms,
  comments, layouts, and fixtures are not implementation input.
- Datum fixtures are authored independently from stable requirement IDs.
- Observable undocumented behavior is recorded only as input/state/output,
  never as copied implementation expression.
- Standards data has explicit version, provenance, license, generation, and
  update policy. It is not smuggled in through a crate.

## 3. Primary authorities

| Area | Primary authority |
|---|---|
| Control grammar | [ECMA-48](https://ecma-international.org/publications-and-standards/standards/ecma-48/), [UTF-8 RFC 3629](https://www.rfc-editor.org/info/rfc3629/) |
| DEC behavior | [VT220 Programmer Reference](https://vt100.net/docs/vt220-rm/contents.html) |
| xterm compatibility | [xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html) |
| Unicode clusters | [UAX 29](https://www.unicode.org/reports/tr29/) |
| Width policy | [UAX 11](https://www.unicode.org/reports/tr11/) |
| Bidirectional text | [UAX 9](https://www.unicode.org/reports/tr9/) |
| Emoji sequences | [UTS 51](https://www.unicode.org/reports/tr51/) |
| Unicode data license | [Unicode licensing policy](https://www.unicode.org/policies/licensing_policy.html) |
| Keyboard extensions | [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) |
| Graphics extensions | [kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) |
| Base64/zlib/DEFLATE | [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648.html), [RFC 1950](https://www.rfc-editor.org/rfc/rfc1950.html), [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html) |
| PNG | [PNG Third Edition](https://www.w3.org/TR/png-3/) |
| Sixel | [DEC VT330/VT340 graphics manual](https://vt100.net/dec/ek-vt3xx-hr-002.pdf) |
| PTY/job control | [Linux pty(7)](https://man7.org/linux/man-pages/man7/pty.7.html), [tty ioctls](https://man7.org/linux/man-pages/man2/ioctl_tty.2.html), [POSIX job-control rationale](https://pubs.opengroup.org/onlinepubs/009604599/xrat/xbd_chap03.html) |
| Capability identity | [terminfo(5)](https://invisible-island.net/ncurses/man/terminfo.5.html) |
| Accessibility | [AT-SPI Text](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/iface.Text.html) |

Exact versions and retrieval dates are pinned in the research/provenance package
before implementation. Updating a standard version is a compatibility change,
not routine dependency refresh.

## 4. Required behavior families

### DTC-R01 — streaming grammar

Incremental UTF-8 plus ECMA-48 C0/C1, ESC, CSI, OSC, DCS, APC, PM, and SOS;
private markers, subparameters, cancellation, BEL/ST termination, chunk
invariance, bounded collection, deterministic malformed-input recovery, typed
actions, replies, events, and damage.

### DTC-R02 — DEC/xterm state

Primary/alternate buffers; cursor, delayed wrap, origin/insert/newline modes;
margins, tabs, save/restore, edit/erase/scroll, protected cells, G0-G3 and DEC
graphics, device reports, complete SGR colors/attributes, private-mode queries,
and deterministic invalid-parameter behavior.

### DTC-R03 — cells and logical lines

Cluster text, width and continuation ownership, complete rendition, hyperlink
and protection identity, hard versus soft line endings, stable logical
coordinates, immutable render snapshots, and explicit damage. No orphan
continuation cell may survive overwrite, erase, resize, or reflow.

### DTC-R04 — Unicode

Pinned Unicode version; independently generated property tables; extended
grapheme clusters; combining marks, variation selectors, modifiers, regional
indicators, tag and ZWJ sequences; explicit ambiguous-width tailoring; safe
wide-cell wrap/overwrite; original-codepoint preservation; and a declared BiDi
posture. Glyph metrics never redefine terminal cell ownership.

### DTC-R05 — input and IME

Printable/control/meta input, dead/composed input, IME preedit/commit, legacy
xterm keys, application cursor/keypad, kitty keyboard negotiation, X10/VT200/
UTF-8/SGR cell and pixel mouse, focus reporting, bracketed paste, local-selection
override, coordinate clipping, and exact workspace-hotkey isolation.

### DTC-R06 — metadata and security events

Title, palette/default colors, CWD, OSC 8 links, controlled OSC 52 clipboard,
OSC 133 shell marks, bell/notification/progress, and synchronized output. These
produce bounded typed requests; they never perform GUI, filesystem, MCP, or
design mutations.

### DTC-R07 — history, reflow, selection, and search

Bounded logical history, alternate-screen isolation, anchor-preserving resize,
stable review while output arrives, deterministic trimming, grapheme/word/line/
block/all selection, copy rules, literal/case search, and a bounded Datum-owned
non-backtracking regular-expression engine.

### DTC-R08 — graphics

Full kitty APC grammar, transfer/query/placement/lifecycle/animation semantics,
raw RGB/RGBA, PNG, zlib, Base64, and explicit safe policy for file/shared-memory
forms. Sixel covers raster attributes, repeats, carriage return/newline, RGB/HLS
registers, transparency, palette policy, clipping, scrolling, and teardown.
Every byte, pixel, object, frame, ratio, and work unit is bounded.

### DTC-R09 — Linux PTY/session semantics

UNIX 98 allocation, grant/unlock, slave open, setsid/controlling terminal,
descriptor hygiene, argv/cwd/env/credentials, termios, foreground job control,
partial nonblocking I/O, EINTR/EAGAIN/backpressure, resize/SIGWINCH, exact child
status, SIGHUP/termination policy, concurrent isolation, and deterministic
teardown. Transport owns no cells or escape semantics.

### DTC-R10 — capability identity and accessibility

`TERM`, terminfo, device attributes, and feature probes advertise only proven
behavior. Accessibility exposes ordered text, caret, selections, attributes,
links, visible/history ranges, search, bell, lifecycle, and focus with coalesced
events plus complete keyboard operation and real AT-SPI/screen-reader proof.

## 5. Resource and hostile-input requirements

The architecture fixes limits for parameter count/digits/value, control-string
bytes, title/CWD/link/clipboard bytes, reply amplification, pending events,
history lines/bytes, graphics objects/pixels/decoded bytes/frames, compression
ratio, parser work, and search/reflow work. Arithmetic is checked. Exhaustion is
deterministic. No PTY byte stream may panic, recurse without bound, allocate
without bound, trigger ReDoS/decompression bombs, escape clipping, execute a
command, or mutate Datum.

## 6. Proof model

Every requirement has an exact authority reference, disposition, Datum-authored
input, expected snapshot/replies/events/damage/resource result, chunk-partition
invariance test, reset/teardown assertion, and security assertion. Proof lands
with each package, not in a late omnibus gate.

External `vttest`, `esctest2`, shells, TUIs, and agents are optional installed
black-box witnesses; their source is never vendored or downloaded by Datum.
Datum-authored normative fixtures remain CI authority.
