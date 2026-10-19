# Product Mechanics 063: Shared File Browser Controls and Host Actions

Status: owner-authorized bounded implementation decision; delivery pending under
NPD-C09. The owner requested browser New Folder/Rename, then explicitly required
reusable menu objects and continued work. The appointed manager coordinates
implementation and independent review. This records no delivery or HUMAN PASS.

## Shared construction

New and Open use one browser controller and entry model, one context-menu
control, one single-line editor, and one typed host-filesystem service. Each
dialog supplies available actions, selection policy and its final Create/Open
command. It must not duplicate input, drawing, validation, filesystem or
accessibility behavior. Future dialogs can use the same components. This does
not require migrating unrelated terminal, viewport or menubar menus first.

Protocol/render components own presentation and interaction state; application
owners connect input, the existing clipboard and filesystem service. The engine
does not depend on GUI controls. Reuse existing dependencies and rendering;
this decision authorizes no new toolkit or third-party package. Both New and
Open must actually consume the shared implementation.

## Human interaction

Retain the existing Location/Browse control and one dialog per action. New and
Open initially show the existing inline navigation and read the resolved starting
directory, normally Home, without requiring Browse or a Location edit. This
initial read is an explicit directory listing, not an assisted parent query;
show loading, empty or unreadable states truthfully. Preserve initial focus on
Project name in New and Location in Open. Browse remains visible and focuses
the same list. Navigation may show ordinary files distinctly. Files are not folder
navigation; eligible files can be Rename targets. Following the owner's
2026-10-18 QA report, Open Project additionally accepts double-click or Enter on
the exact regular `project.json` entry as a shortcut for its existing Open
command, using the containing directory shown in the candidate preview. This
identifies a candidate, not a validated Project: the same asynchronous Open
preparation, source admission and refusal behavior apply. An arbitrary file,
link entry, similarly named entry or file in New Project does not activate Open.
Folder activation continues to navigate; highlighting never opens a Project.
The exact Project destination/candidate remains visible.

Manifest activation shares the Open button's pending host action, native-edit,
existing Open and session admission guards. A refusal remains visible and does
not queue an automatic retry. Only a ready current listing can supply activation;
editing Location, navigation, a rescan or dialog replacement ends any pending
double-click gesture. Stale displayed rows cannot retarget Open. Preserve the
current Project, terminal selection, geometry and processes on refusal. Popup
and naming-editor Enter retain their own actions and take precedence. The
shared row semantics and accessibility description identify this Open shortcut;
no separate file-opening service, file association, dependency or dialog is added.

Right-click targets the entry under the pointer, independently of a previous
selection. Blank list space offers New Folder; Rename is unavailable without
an eligible entry. Menu/Shift+F10 opens the menu for the current selection;
F2 starts Rename and Ctrl+Shift+N starts New Folder. Arrow navigation, Enter,
Escape and outside dismissal use the shared popup, restore browser focus and
never accidentally activate a folder or Project.

New Folder and Rename use the same naming editor within the existing dialog.
The initial rename name is selected. Normal caret movement, selection,
replacement, Home/End, Delete/Backspace and Ctrl+A/C/X/V work; pointer positioning
and selection use the same editor. New Project name, folder override and
Location reuse that editing behavior. Preserve exact Unicode and spaces;
reject invalid basename components, separators, NUL and dot names without
silent sanitization. Enter submits the naming action; Escape cancels only that
draft and returns focus. Useful refusal details retain the draft. Name the
inner action Create Folder so it cannot be confused with Project Create.

Success refreshes the current location and selects the affected entry without
entering it or creating/opening a Project. Location stays the same parent;
ordinary Enter/double-click enters a folder. Preserve Project naming, folder
override, pinned Units and terminal sessions throughout.

## Location suggestions and completion

The owner's 2026-10-18 QA requests adaptive path filtering, find and Tab
completion after partial `~/Doc` input produced a missing-folder error. New and
Open share this assistance in the existing Location/browser controls. No second
dialog, recursive search, persistent index or new dependency is introduced.

Location remains the exact editable candidate. While editing an unslashed final
component, read its parent even if that component exactly names an existing
directory. Keep the matching siblings visible until explicit navigation or
acceptance; filesystem existence alone does not enter a folder. A typed trailing
separator, selected/unique completion, Enter or ordinary navigation requests the
exact directory listing. Roots and supported home/dot navigation anchors retain
their resolver meaning rather than becoming parent-name queries. For a queried component,
show matching navigable children: case-insensitive literal substring matches,
prefix matches first, with exact filesystem spelling preserved. Matching is
local and transient; dot-folders remain available. Reveal the same inline list
for a nonempty partial query while retaining Browse. Distinguish this suggestion
list and its parent from an ordinary directory listing. A query/no-match state
is neutral, not a red missing-folder refusal while the user types. Real access
failures remain described truthfully; explicit failed Enter/Browse/Create/Open
reports the attempted path and useful reason.

With Location focused, the caret at its end and no selected text, unmodified
Tab first accepts an explicitly selected current suggestion, including a
substring-only match. Otherwise, use the nonempty case-insensitive prefix-match
group for completion; substring-only matches remain visible and selectable but
do not block that group. If there are no prefix matches, retain the existing
substring-match fallback. A unique candidate in the completion group accepts its
exact name and appends the folder separator. With multiple candidates, extend
only their exact shared prefix at a UTF-8 boundary, when every name in that group
extends the typed prefix. Case folding does not synthesize a multiple-name prefix.
For example, `dat` with `datum-eda`, `datum-eda-backups`,
`datum-eda-workspaces`, `datum_projects` and `plugdata` completes to `datum`: the
first four form the prefix group, and the underscore prevents adding a hyphen. An extension that exactly names a listed folder remains a query without
appending a separator or entering that folder; sibling matches stay visible.
Case folding is for
matching/ranking only, never synthesized spelling. Otherwise Tab follows ordinary
focus traversal. Shift+Tab always traverses. Down/Up select current suggestions
while retaining Location focus; arrows do not change the path. Enter on the
selected suggestion or list double-click navigates to that
folder, never creates or opens a Project. Escape clears an active suggestion
selection first without changing Location, then retains the dialog's existing
Escape behavior. Popup/naming controls keep their own keyboard precedence;
middle-caret and selected-text editing keep normal editor behavior. Suggestions
never silently replace typed text. Completion preserves exact Unicode, spaces,
relative-path and supported home-shorthand meaning.

For an already explicitly loaded exact directory whose Location lacks a trailing
separator, eligible Tab appends only that separator to the same directory. It
never chooses a child. An already suffixed path uses ordinary Tab traversal.
This permits successive component completion after either typing or browsing.

Highlighting a suggestion does not change the submitted candidate. Create/Open
still validates exactly the typed Location through existing routes without
requiring an extra Enter; never fall back to the query parent or first match.
Accepting a suggestion updates Location explicitly. Naming/host actions and the
project.json shortcut require an exact ready directory listing, not suggestions.
Preserve the unfiltered occupied-name set for collision checks. Bind discovery
and completion to the full current query and generation; editing, navigation,
rescan and dialog replacement invalidate old selections and click pairs. Reads
remain off the UI thread through the existing cancellable worker. Names, folder
override, pinned Units, pending host receipts and current sessions keep their
existing ownership. Announce suggestions/no matches and key actions through the
shared accessibility projection without announcing a filesystem error per key.

## One host-filesystem service

Typed actions create one child directory or rename one regular file/directory
basename within its existing parent. These are host-filesystem actions, separate
from engine genesis, DesignModel mutations, Project/Library display-name edits,
LibraryBinding and Project relocation. No delete, overwrite, cross-parent move
or link-entry mutation is included. Existing link navigation remains governed
by the browser's navigation policy.

Pin the parent directory handle and exact source identity, recheck at mutation,
and use atomic no-replace publication. Refuse changed entries and unavailable
safety semantics. Do not use an overwriting rename after a check-then-act test.
Filesystem work runs off the input thread. Submitted mutations have retained
completion receipts independent of latest-only, cancellable scan results.
Prevent duplicate submission and never retarget an action after navigation.
Refresh only the matching location; retain a truthful completion if its dialog
closes. Cancellation cannot claim to roll back a completed filesystem action.

Project Create/Open admission refuses while a submitted host action or its
retained completion is pending, including after closing/reopening a dialog.
Their buttons and accessibility readiness reflect that temporary condition.
Conversely, host-action admission retains protection until actual Open
preparation retires, even when its dialog was cancelled. Dropping a receiver or
clearing a UI busy flag does not establish worker retirement. Navigation, draft
editing and outer Cancel remain available. Ordinary validation resumes after
completion; no Project action runs automatically. This bounded coordination
prevents an internal rename from relocating a Project during genesis/Open.

Completed explicit host actions persist after New/Open Cancel. Passive browsing
and cancelled naming drafts write nothing. Create/Open remain the only Project
publication actions, through their existing governed engine/worker routes.

## Project and session preservation

Both actions refuse writes inside managed Project/library backing storage and
creation of Datum's reserved backing namespaces (including `.datum` and its
revision/pool/schema storage). New Folder may still create an ordinary child in
an eligible host cwd; the ancestor/relocation restrictions below apply to Rename.

Generic Rename refuses recognized Datum Project roots/containers, including a
selected directory containing a nested Project; canonical manifests, shards and
engine storage namespaces; active library backing paths; and entries equal to
or containing known current, retained or pending Project/library/session
anchors. Session anchors include current cwd as well as origin/launch paths.
Do not rebase live origins or use this service to rename a Project or Library.

Classification is read-only: do not resolve a Project or acquire a writer lease
to probe eligibility, since those operations may create storage or recover state.
A missing/free writer lock does not prove no application has the Project open.
Inspect only the candidate subtree, asynchronously and without following links;
unreadable, changed, ambiguous or incomplete inspection refuses the action.
Use path-component and descriptor identities, not string-prefix matching.
Ordinary host files outside protected namespaces remain eligible. These checks
do not claim knowledge of all external readers or control over uncooperating
external namespace changes.

## Delivery and retained authority

The actual Claude-owned start-page target must cover popup, naming, selection,
busy, refusal and minimum-size states before native translation. Independent
review must verify shared ownership and ordinary native Wayland New/Open use,
text editing, collision/stale/protected-entry refusal, completed-action
persistence and Project/terminal preservation. Use owned scratch data only.

[PM014](PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM.md),
[PM023](PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING.md),
[PM029](PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md) and
[PM059](PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md)
retain their authority. PM037/040 Preferences and genesis semantics do not
change. The current NPD-C09 assignment owns execution; the accepted b015 QA
snapshot remains frozen. Protected visual-owner landing, full qualification
and the S5A hold are not waived.
