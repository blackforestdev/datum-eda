# Product Mechanics 055: Bus Run contact authority

Status: owner-ratified clarification.
Frontier: S5A-DERIVED-AUTHORITY / DA-A2.
Issue: dat-s5a-bus-run-contact-pnze; implementation: dat-s5a-derived-authority-hhsw.

## Decision and owning boundary

The owner explicitly answered **“Constrain to origin Bus; disclose foreign
contact”** for physically touching spines belonging to distinct declared Bus
identities, including equal names or scalar member sets. This resolves the
concrete UVT physical Run versus PM053 C03 declaration ambiguity.

A Bus Run is the complete physically connected component within its origin's
resolved declared semantic Bus, in that schematic occurrence. A foreign declared
Bus contact is separate source-qualified contact evidence naming both declarations
and authored occurrences. It does not join membership, equate declarations,
create a scalar Net assignment, or turn physical contact into a scalar short.
An explicit BusInterface remains a semantic relationship, never a local physical
bridge. Disconnected same-Bus representations belong to the semantic Bus tier,
not this Run. Equal names/member sets and hierarchy source UUID reuse establish
neither contact nor identity. Missing/ambiguous declaration or required source
basis returns typed unavailable, never renderer-generated authority.

Preserve section → Run → semantic Bus, complete authoritative membership,
visibility-limited projection, owned name/entry presentation, independently
selected scalar Nets, PM026 D4 refusal/no-op and per-revision lifetime. Foreign
contact disclosure is inspectable/non-color information, not selection treatment
for the foreign Bus or additional mutation authority. Existing origin eligibility
and Bus-entry non-origination rules remain.

## Reconciliation and focused proof

PM026, PM053, UVT §2.2.13/§2.2.20 and schematic connectivity §4.5 consume this
clarification. Existing selection study §6 shows semantic Bus ownership and scalar
separation and requires no HTML or visual styling change. Guidance records the same membership boundary; the unchanged Rendering Book
continues to govern the same visual construction; no new subject or identity mapping is added.

Prove touching foreign declarations stay separate with exact contact pairs,
including identical names/scalar sets; connected versus disconnected same-Bus
spines; owned entries/labels versus related scalar wires; repeated-sheet paths
and explicit interfaces; complete membership beyond the context cap; invalid
origin/assignment/revision and immutable source/journal bytes. Canonical topology
edit, undo, replay/reopen must rederive the same lawful occurrence-qualified sets.
Shared adapters must retain and disclose contact independently of selection.
These are candidate obligations, not proof already accepted.

## Execution boundary

Implementation stays within the existing PM054 E3/E4 query/adapter authorization.
No broader Bus connectivity rewrite, dependency, renderer/resize change or
manufacturing allowance is granted. C07/M1 remains deferred and mandatory T04
unpassed; independent exact candidate review/replay remains reserved. This
clarification alone supplies no complete foundation or S5A acceptance.
