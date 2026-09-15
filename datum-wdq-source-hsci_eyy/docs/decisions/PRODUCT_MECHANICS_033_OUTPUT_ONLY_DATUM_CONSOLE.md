# Product Mechanics 033: Output-Only Datum Console

Status: ratified doctrine

## Context

Datum inherited three different meanings for “console”: an obsolete application
output lane, an AutoCAD/EAGLE-like editor command input, and a passive sink for
GUI narration kept away from terminal cells. The fully fledged Datum Terminal
now owns command-line input. Research and an owner-reviewed comparative study
show that immediate action feedback remains necessary, but neither a second
command line nor a catch-all log is.

This decision amends the Console language in decisions 019, 024, and 027 and in
the GUI and terminal specifications. Historical research remains provenance,
not current authority.

## Decision

Datum has one user-facing **Datum Console**: an output-only action-feedback
surface. It accepts no authored text, takes no keyboard focus, parses or
dispatches no verb, emits no design operation, and writes no PTY byte or
terminal cell. The retired editor-command-line candidate is not a deferred
upgrade path. Terminal sessions remain Datum's only command-line input surface;
manual GUI authoring remains tools, menus, shortcuts, and marking gestures over
the direct typed-operation write path.

The selected visual model is Candidate A from
`docs/gui/prototypes/command-feedback-study.html`:

- a compact overlay at the lower-left of the focused workspace pane;
- no reserved canvas geometry and no duplicate strip in unfocused panes;
- routine action echo, active-tool prompt, and refused-action modes;
- truncation before wrapping in narrow panes, with the full record available in
  deliberately opened history; and
- Candidate C notification tiers as a complementary surface, not Console
  modes.

The Console exposes a bounded, expandable session history. Consumer feedback is
not design truth and is never journaled. Committed operation history is projected
from the canonical journal rather than copied into a second handwritten audit
log. History has a deterministic bound and an explicit overflow indication.

## Typed feedback model

The implementation replaces the legacy untyped `ConsoleLaneState` / `Vec<String>`
sink with a typed consumer-state feedback record. Each record carries, at
minimum, severity, source, category, timestamp, display text, and optional
target/action identity. Publishers state facts; the shell chooses the governed
destination and presentation. Canonical action wording derives from the verb
registry or journal provenance where either is authoritative; internal IDs do
not leak as user-facing prose.

This model is not a second mutation or audit authority. It cannot author CLI
strings, operations, proposals, or terminal input.

## Consequence-based routing

Messages route by user consequence, not by the subsystem that produced them:

| Message class | Authoritative destination |
|---|---|
| GUI action/tool/view/selection echo | Datum Console and bounded history |
| Active multi-step tool instruction | Datum Console tool-prompt mode |
| Refused GUI action | Persistent-until-next-relevant-action Console refusal plus accessible announcement |
| Terminal lifecycle, tab, PTY, or session state | Terminal-local chrome; severe/persistent facts also publish to Notices |
| Persistent failure or degraded application state | Sticky banner plus Notices log |
| Background progress | Owning surface or status-bar progress; completion/failure may publish a Notice |
| ERC/DRC findings | Dedicated findings surface, canvas markers/cross-probe, and status count |
| Committed design-operation history | Projection of the canonical journal in expanded history |

The notification backbone remains separately tracked by
`dat-notification-system-ztp`. It complements the Console with transient,
persistent, and out-of-view attention tiers; it does not turn the Console into
a general notification log.

## Accessibility and focus

Visible feedback publishes an equivalent AT-SPI status update without stealing
focus. Routine echoes use polite priority; critical/refusal feedback uses only
the priority justified by its consequence. Icon, text, and severity semantics
remain redundant with color. Auto-fading information remains recoverable in
history, pauses while inspected, and respects a user duration preference.

## Roadmap boundary

The Console implementation is the selected next development task after this
contract recovery. It is not a technical prerequisite of the direct GUI
write-path track, and that write path is not a prerequisite of the display
surface. No dependency edge may re-create that former conflation.

The first implementation slice must replace and classify the legacy untyped
sink before rendering it. Later slices render the selected overlay, connect
bounded history and routing, and close accessibility and conformance evidence.
This decision does not itself authorize an Output tab, interactive editor
command line, notification implementation, findings browser, or GUI mutation.

## Owner evidence

<!-- EVIDENCE:CONSOLE-CONTRACT-RECOVERY:CONSOLE-C05-RATIFIED -->
<!-- EVIDENCE:CONSOLE-CONTRACT-RECOVERY:CONSOLE-C04-OWNER-APPROVED -->
On 2026-08-22 the owner approved the complete Console visual disposition:
Candidate A as the output-only surface, Candidate C as complementary notices,
focused-pane lower-left overlay placement, expandable bounded session history
with journal projection for committed operations, the research routing matrix,
independent GUI-write-path sequencing, and “Datum Console” plus a typed internal
feedback model as the reconciled vocabulary.

## Dependency and licensing impact

None. This decision adds no third-party dependency and grants no dependency or
license exception under Product Mechanics 029.
