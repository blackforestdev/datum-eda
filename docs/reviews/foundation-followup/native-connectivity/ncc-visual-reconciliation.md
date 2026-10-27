# NCC — Claude-owned visual reconciliation of units-and-grid-model.html

**From:** frontend-dev-01 (Claude), visual lane — exclusive owner of `docs/gui/prototypes/*.html`.
**To:** PM-Agent-01 (requested this artifact) + dev-agent-01 (NCC claim-holder) + audit-agent-01 (NCC-REVIEW reviewer, relayed via PM).
**Date:** 2026-10-26. **Step served:** NATIVE-CONNECTIVITY-CONTRACT / the Claude-owned visual reconciliation that NCC-REVIEW must check exists (FOUNDATION_FOLLOWUP_PLANS.md:122-130).
**Posture (confirmed by PM clarifier):** PLANNING-STAGE **prototype-depiction** reconciliation on paper — does the prototype correctly DEPICT the reconciled connectivity behavior (dev's C02 §D file/region/outcome/preserved-decisions list) consistently with the governed clauses. NOT a running-app golden-parity gate; there is no native connectivity UI yet and NCC-REVIEW bars a new native proof. **Read-only: the prototype is INSPECT-ONLY. I authored no HTML change and bless nothing.** Any actual edit to the prototype is a separate owner-authorized Claude-prototype action after NCC-C03.

---

## §A — What I inspected DIRECTLY vs READ from documents
**Directly inspected (this session, byte-level):**
- The prototype `docs/gui/prototypes/units-and-grid-model.html` — §2 block (lines 123-133) and §7B block (lines 209-251) in full DOM/CSS, plus the PM-040 supersession banner (107), the UNIT-I02R amendment (108), Rule 4 (151-153) and the §7B four-statement status (269). States are static (a findings document; the `.gopt.on` active-row markers and `.statusmark[data-state]` chips are the only "state" expressed — all read directly).
- `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` lines 82-137 (§4.1-§4.6) — dev's source-pinned copy.
- `docs/SCHEMATIC_CONNECTIVITY_SPEC.md` lines 34-104 — the copy the prototype's citation actually names.

**Read from documents (carried faithfully, NOT independently re-derived):**
- dev's NCC-C01 clause matrix (`docs/reviews/foundation-followup/native-connectivity/NCC-C01-reconciliation.md`, committed f37cc541) and NCC-C02 behavior draft (dev scratchpad `NCC-C02-reconciliation.md`, §D handoff).
- FOUNDATION_FOLLOWUP_PLANS.md:110-140 (NCC-C02 / NCC-REVIEW / NCC-C03 bounds).
- The PM decisions and `authority-review.md` section numbers (PM053/PM055/PM003/PM023/PM026/PM000D; §691-722, §697-701, §712-722, §966, §969-971) **as cited by dev** — I did not re-open authority-review.md or the decision records to re-verify those line numbers (see §F scope boundary). I verified the one citation I could reach directly — the spec — and it is where the findings below bite.

---

## §B — §2 depiction reconciliation: FORMATION layer depicted faithfully; IDENTITY layer not shown as distinct

**What §2 depicts (direct reading of lines 123-133):**
- `.quote` (126-127): *"A pin endpoint that coincides with a wire endpoint (within zero tolerance — integer coordinates, exact match) creates a connection."* cited `docs/SCHEMATIC_CONNECTIVITY_SPEC.md:92 — ratified`.
- `.lead` (129): *"Two objects … are electrically connected if and only if their integer coordinates are identical. Not close. Identical."*
- `.hero` (131): *"What guarantees that identity is the wire tool snapping to the pin — not the lattice … the editor writes the pin's exact coordinate into the endpoint; the integers match because the snap made them match."*
- `.cap` (133): the KiCad-grid-warning rebuttal — the real requirement is *"the wire tool must snap to pin endpoints exactly, always."*

**Reconciliation against the C01/C02 layered model:**
- §2 is a clean depiction of the **FORMATION layer (C01 B2)** and the **pin-attachment gesture (C02 behavior 4, D1):** a connection forms by exact coordinate coincidence, and that coincidence is *guaranteed by object-snap writing the pin's exact coordinate into the wire endpoint.* That snap-not-lattice framing is consistent with PM023 (object-snap overrides grid; committed coords exact) and with dev's C02 behavior-4 recommendation (attach by exact pin coordinate; object-snap overrides grid). The depiction of the formation/attachment leg is sound.
- **The gap dev's C02 §D names, confirmed in the pixels:** §2 does **not** depict the **IDENTITY layer (C01 B1)** as a thing distinct from formation. There is no depiction of a net's stable UUID/NetId surviving quantize, addressed by identity rather than coordinate. Shown alone, §2's "connected iff coordinates identical" (129) and Rule 4's "coincidence *is* connectivity" (152) read as *coordinate-coincidence is the whole connectivity story* — which is precisely the framing C02 §D asks the eventual prototype update to split into two distinct layers (formation vs identity), **not coordinate-coincidence-as-identity.**
- Note in fairness: §2's own word "identity" at line 131 means *the guarantee of the connection* (the snap), not net identity/UUID. So §2 does not assert coincidence-*as-net-identity*; it simply never shows the identity layer. The depiction isn't false — it's **one layer shown without its partner.**

**D3 conflict — PRESERVED, not resolved (per PM/dev instruction):** §2's exact-coordinate-coincidence basis is recorded here as **ONE SIDE** of the open D3 attachment/update conflict (C01 §D, B4): the other side (UVT §3.3, as cited by dev) holds that connectivity *survives quantize because nets are UUID/net-addressed, not coordinate-coincident.* C01 isolates these as two layers and leaves the binding attachment/update contract open for NCC-C03 owner disposition. **I do not resolve it, do not pick a side, and do not declare the prototype wrong.** §2 is faithfully the formation-side framing; whether the prototype's depiction should later be extended to show both layers is a D3/owner decision, and the HTML change is a separate owner-authorized action — not this lane's, not now.

---

## §C — Stale citation finding (precise; sharper than the handoff assumed)

The prototype's §2 `.cite` (line 127) reads `docs/SCHEMATIC_CONNECTIVITY_SPEC.md:92 — ratified`. Three facts, all directly verified:

1. **The quoted sentence is a real, near-verbatim quote** of the **`docs/` copy's "### Pin Positions"** section — `docs/SCHEMATIC_CONNECTIVITY_SPEC.md:99-100`: *"A pin endpoint that coincides with a wire endpoint (within zero tolerance — integer coordinates, exact match) creates a connection."* The prototype did not invent the "zero tolerance / exact match" phrasing; it is the docs/-copy's own wording. The **line number is stale**: `docs/`:92 is inside **"### No-Connect Markers"** (*"It asserts: 'this pin is intentionally unconnected.'"*) — unrelated to formation. Correct anchor in the docs/ copy is **:96-100 (Pin Positions)**.
2. **Against dev's source-pinned `specs/` copy, the citation is stale a second way:** there `:92` is the **§4.2 Labels** boundary, and the formation rule is **§4.1:85-91** (*"share an endpoint, or a junction explicitly joins them, or the source format defines the touch …; crossing wires without a junction are not connected"*). dev's C01 §E / C02 §D re-grounding note ("`:92` now lands on the §4.2 labels boundary; the formation rule is §4.1:85-91") is **accurate** — verified directly.
3. **The two governed copies diverge, and they diverge on exactly the D3 question.** `specs/` §4.1 frames formation as *shared-endpoint / explicit-junction / source-format-touch* and contains **no** "exact coordinate coincidence / zero tolerance" clause. The `docs/` copy's "Pin Positions" **does** state exact-coincidence-creates-a-connection. So the prototype's exact-coincidence basis is sourced to the `docs/` copy; the `specs/` copy that dev treats as authority phrases formation differently and lacks that clause. **The D3 conflict therefore already exists in governed text, between the two spec copies — not only between the prototype and UVT §3.3.**

**Routing:** findings #1 and #2 are the re-grounding the eventual (owner-authorized) prototype update must apply — point the citation at the live Pin-Positions / §4.1 formation text, in whichever copy NCC-C03 ratifies as authoritative. Finding #3 (two divergent governed spec copies) is a **source-traceability flag that is NOT NCC-owned and NOT mine to resolve** — it belongs to the owning spec route / ETM reconciliation and is an owner input at NCC-C03. I surface it; I do not pick the authoritative copy or edit either spec.

---

## §D — §7B depiction reconciliation: FAITHFUL to the preserved presets

Direct reading of lines 209-251. §7B (`.sec` "7B · CLAY ROW 1 — DECIDED BY THE OWNER") depicts three `.win` preset columns:
- **Metric · default** — 1-2-5 decade series (2.5 / 1.0 / **0.5 `on`** / 0.2 / 0.1 / 0.05 mm); `.cap` states *"Board default 0.5 mm; schematic default 2.5 mm."*
- **Mil · preference** — same series in mil (100 / **50 `on`** / 20 / 10 / 5 / 1 mil); `.cap` *"one preference change."*
- **Imperial-derived metric** — 2.54 / 1.27 / 0.635 / 0.254 / 0.127 mm; `.cap` *"never the default and never suggested."*

This matches the owner-dispositioned preset decision exactly (authority-review §697-701 as cited): **Metric default (board 0.5 / schematic 2.5 mm); Mil optional; Imperial-derived never default.** The `.law` at 250 (*"What makes this safe is the pin snap, not the lattice"*) and `.cap` at 251 (*"Governance impact: none"*) are consistent with PM023 object-snap and with §2's formation framing. **Depiction is faithful; §7B needs no change and is a PRESERVED decision — I do not reopen it.**

---

## §E — Preserved decisions confirmed NOT reopened
- **§7B presets** (Metric default board 0.5 / schematic 2.5 mm; Mil optional; Imperial-derived never default) — faithfully depicted (§D); untouched.
- **PM-040 units supersession** — the prototype already records it (banner line 107; status statement 2 at line 269: existing Project resolves display/measurement/bare-numeric from Project Working Units; Global Units are defaults for future Projects). Correct as recorded; not reopened. Note PM-040 supersedes Rule 4's *per-person display ownership* claim only; §7B status statement 1 explicitly affirms the grid-preset decision is unaffected by PM-040 — consistent.
- **PM023 shared snap** — §2 (131) and §7B (250) both rest connection on the wire-tool pin snap (object-snap overrides grid); consistent, not reopened.

---

## §F — Scope boundary (what this reconciliation does and does NOT cover)
- **Covers:** the prototype's §2 and §7B depiction vs the C01/C02 layered model and the preserved decisions; the spec:92 citation (verified directly against both spec copies). Visual/depiction authority only.
- **Does NOT cover / not blessed here:** (a) I did not re-verify dev's `authority-review.md` and PM-decision line citations — carried faithfully, not re-derived. (b) I did not adjudicate which spec copy (`specs/` vs `docs/`) is authoritative — that is the owning spec route + owner at NCC-C03. (c) I did not resolve D1/D2/D3 or the atomic-refusal/compensating-journal records — those are dev's draft + owner ratification. (d) No native proof, no topology claim, no running-app render (none exists; none required at this stage). (e) No HTML edit — the prototype is inspect-only; the actual depiction update is a separate owner-authorized Claude-prototype action.

---

## §G — Bottom line
The Claude-owned visual reconciliation **exists** and is recorded here, as NCC-REVIEW requires it to.
- **§2 depicts the FORMATION layer and the pin-attachment gesture faithfully**, but shows it without the distinct IDENTITY layer — matching the gap dev's C02 §D outcome names (show formation vs identity as distinct layers, not coordinate-coincidence-as-identity). Recorded as one side of the open **D3 conflict and PRESERVED** for owner disposition at NCC-C03; not resolved, prototype not declared wrong.
- **Citation `:92` is stale** two ways and the quoted exact-coincidence rule lives at the `docs/`-copy Pin-Positions section (:96-100) / `specs/` §4.1 (:85-91); additionally the **two governed spec copies diverge on the exact formation question** — surfaced as a non-NCC source-traceability/owner input, not resolved here.
- **§7B presets, PM-040, PM-023 are faithfully depicted and NOT reopened.**
- The eventual prototype update (owner-authorized, NCC-C03+) would: (1) re-ground the §2 citation to the live formation text in the ratified copy; (2) add the identity layer beside formation so the two read as distinct; (3) leave §7B untouched. **That is a handoff description, not an edit and not a blessing** — the HTML stays as-is until an owner-authorized action.

**Artifact path:** `/tmp/claude-1000/-home-bfadmin-Documents-datum-eda/8e619a1b-92df-496e-a133-9ed9b3dd45b0/scratchpad/ncc-connectivity/NCC-VISUAL-RECONCILIATION.md`
