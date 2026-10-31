"""Finite GP-CM05 V2 coverage and unchanged numeric acceptance limits."""

BUDGETS = {
    'release-query': {'p95_ms': 50, 'max_ms': 100, 'max_rss_mib': 32},
    'durable-mutation': {'p95_ms': 100, 'max_ms': 250, 'max_rss_mib': 32},
    'project-genesis': {'p95_ms': 100, 'max_ms': 250, 'max_rss_mib': 32},
    'project-validation': {'p95_ms': 50, 'max_ms': 100, 'max_rss_mib': 32},
    'storage-growth': {'empty_project_kib': 16, 'per_preference_generation_kib': 32},
    'gui-feedback': {'p95_ms': 50, 'max_ms': 100},
    'native-window-open': {'p95_ms': 500, 'max_ms': 1000},
    'window-lifecycle': {'max_rss_growth_mib': 4, 'orphan_windows': 0, 'orphan_writer_leases': 0},
}

VARIANTS = {
    'clean-install': {
        'missing-store': (['engine', 'gui', 'cli', 'mcp'], ['factory-defaults', 'no-repository-created', 'no-project-write']),
        'reopen': (['engine', 'gui', 'cli', 'mcp'], ['same-values', 'no-repository-created']),
    },
    'upgrade': {
        'alias-migration': (['engine'], ['pure-plan', 'exact-pre-migration-backup', 'one-live-alias', 'atomic-promotion']),
        'migration-required': (['gui', 'cli', 'mcp'], ['mutation-refused', 'bytes-preserved', 'inspection-available']),
    },
    'downgrade-preservation': {
        'unknown-key': (['engine'], ['inactive', 'exact-bytes-survive']),
        'unknown-field': (['engine'], ['inactive', 'exact-bytes-survive']),
        'future-version': (['engine'], ['typed-refusal', 'exact-bytes-survive']),
        'restore-both-directions': (['engine'], ['original-and-restored-digests', 'no-silent-rewrite']),
    },
    'corrupt-store': {
        'truncated-head': (['engine', 'gui', 'cli', 'mcp'], ['bytes-preserved', 'truthful-read-only-state', 'no-silent-repair', 'no-project-write']),
        'missing-head': (['engine', 'gui', 'cli', 'mcp'], ['bytes-preserved', 'truthful-read-only-state', 'no-silent-repair', 'no-project-write']),
        'missing-generation': (['engine', 'gui', 'cli', 'mcp'], ['bytes-preserved', 'truthful-read-only-state', 'no-silent-repair', 'no-project-write']),
        'wrong-digest': (['engine', 'gui', 'cli', 'mcp'], ['bytes-preserved', 'truthful-read-only-state', 'no-silent-repair', 'no-project-write']),
        'unpublished-generation': (['engine', 'gui', 'cli', 'mcp'], ['bytes-preserved', 'truthful-read-only-state', 'no-silent-repair', 'no-project-write']),
    },
    'portable-collision': {
        'alias-collision': (['engine'], ['both-values-retained', 'whole-operation-refused', 'no-side-effects']),
        'capability-injection': (['engine'], ['authority-refused', 'unknown-data-preserved']),
    },
    'backup-restore': {
        'exact-backup': (['engine'], ['all-bytes-preserved', 'no-project-write']),
        'restore': (['engine'], ['preview-pure', 'backup-before-restore', 'exact-generation-authority']),
        'reverse-restore': (['engine'], ['original-bytes-restored']),
        'stale-preview': (['engine'], ['refusal', 'no-write']),
        'incomplete-backup': (['engine'], ['refusal', 'no-write']),
    },
    'project-genesis-crash': {
        'staging-prepared': (['engine'], ['destination-absent-or-complete', 'no-false-acknowledgement', 'owned-cleanup-only', 'restart-recovery']),
        'project-built': (['engine'], ['destination-absent-or-complete', 'no-false-acknowledgement', 'owned-cleanup-only', 'restart-recovery']),
        'project-validated': (['engine'], ['destination-absent-or-complete', 'no-false-acknowledgement', 'owned-cleanup-only', 'restart-recovery']),
        'staging-synced': (['engine'], ['destination-absent-or-complete', 'no-false-acknowledgement', 'owned-cleanup-only', 'restart-recovery']),
        'project-published': (['engine'], ['destination-absent-or-complete', 'no-false-acknowledgement', 'owned-cleanup-only', 'restart-recovery']),
    },
    'project-genesis-replay': {
        'identical-concurrent': (['engine', 'daemon'], ['one-publication', 'one-receipt', 'identical-replay']),
        'response-loss': (['cli', 'mcp'], ['restart-replay', 'no-second-write']),
        'changed-content': (['cli', 'mcp'], ['idempotency-conflict', 'destination-unchanged']),
        'other-request-existing-target': (['cli', 'mcp'], ['project-target-exists', 'destination-unchanged']),
    },
    'existing-project-stability': {
        'global-populated': (['gui', 'cli', 'mcp'], ['exact-eight-seeds', 'resolver-valid-four-file-project', 'immutable-receipt', 'later-global-edits-no-change']),
        'global-missing': (['gui', 'cli', 'mcp'], ['exact-eight-seeds', 'factory-contribution-disclosed', 'no-preference-store']),
        'factory': (['gui', 'cli', 'mcp'], ['exact-eight-seeds', 'no-preference-read-or-write', 'immutable-receipt']),
        'fixed-identities': (['engine', 'cli', 'mcp'], ['byte-identical-projects', 'same-normalized-request']),
        'unavailable-global': (['gui', 'cli', 'mcp'], ['refusal', 'no-implicit-factory-fallback', 'draft-preserved']),
    },
    'managed-offline': {
        'inactive': (['engine'], ['state-disclosed', 'inactive-source-never-wins', 'no-management-ui', 'no-network-required']),
        'expired': (['engine'], ['state-disclosed', 'inactive-source-never-wins', 'no-management-ui', 'no-network-required']),
        'unavailable': (['engine'], ['state-disclosed', 'inactive-source-never-wins', 'no-management-ui', 'no-network-required']),
        'offline': (['engine'], ['state-disclosed', 'inactive-source-never-wins', 'no-management-ui', 'no-network-required']),
    },
    'surface-parity-negative': {
        'active-inventory': (['gui', 'cli', 'mcp'], ['11-active', '45-reserved-absent', 'two-sections', 'same-service']),
        'all-query-schemas': (['engine', 'cli', 'mcp'], ['same-result-fields', 'same-disclosure']),
        'reserved-keys': (['cli', 'mcp'], ['typed-refusal', 'no-activation']),
        'malformed-requests': (['engine', 'cli', 'mcp'], ['one-code-per-input-class', 'no-write']),
        'all-stable-refusals': (['engine', 'cli', 'mcp'], ['exact-code-and-details', 'cli-exit-2', 'mcp-envelope']),
        'writer-race': (['cli', 'daemon', 'mcp'], ['one-winner', 'stale-or-conflict', 'no-hidden-retry']),
        'lease-restart': (['engine', 'daemon'], ['dead-process-releases-lease', 'no-sentinel-authority']),
    },
    'human-agent-authority': {
        'spoofed-actor': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'no-foreground-tty': (['cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'direct-mcp-set-reset': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'stale-proposal': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'acceptance-mismatch': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'acceptance-expired': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'acceptance-consumed': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'daemon-restart': (['cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
        'request-id-conflict': (['engine', 'cli', 'mcp'], ['no-unauthorized-write', 'safe-error-details', 'project-unchanged']),
    },
    'accessibility-interaction': {
        'keyboard-all-controls': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
        'search-and-clear': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
        'explanation-and-return': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
        'set-reset-refusal': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
        'new-project-modes': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
        'native-close-reopen': (['gui'], ['native-at-spi-evidence', 'correct-focus', 'correct-name-role-value-state', 'no-trap', 'truthful-announcement']),
    },
    'responsive-render': {
        'ordinary': (['gui-render', 'gui'], ['all-controls-reachable', 'no-clipping', 'correct-provenance']),
        'narrow': (['gui-render', 'gui'], ['all-controls-reachable', 'explanation-stacks', 'no-clipping']),
    },
    'noncolor-render': {
        'high-contrast-on': (['gui-render', 'gui'], ['word-and-shape-cues', 'all-information-preserved', 'no-motion-dependent-action']),
        'high-contrast-off': (['gui-render', 'gui'], ['word-and-shape-cues', 'all-information-preserved', 'no-motion-dependent-action']),
        'reduced-motion-on': (['gui-render', 'gui'], ['word-and-shape-cues', 'all-information-preserved', 'no-motion-dependent-action']),
        'reduced-motion-off': (['gui-render', 'gui'], ['word-and-shape-cues', 'all-information-preserved', 'no-motion-dependent-action']),
        'color-removed': (['gui-render', 'gui'], ['word-and-shape-cues', 'all-information-preserved', 'no-motion-dependent-action']),
    },
}

# Every (family, variant, surface) REMOVED from a VARIANTS owed-set vs the pre-GP-CM05E
# baseline is recorded here with its ruling + the structural citation that the surface has
# NO INDEPENDENT AUTHORITY for the assertion-class (the triggering state is unreachable from
# that surface, so it is a projection of engine state). matrix.py refuses a reduction without
# a resolvable entry. Modes: engine_surface_by_contract (removed, proven on engine) | native
# (owed; informational producer note) | scope_exit (owner-visible qf, GP-CM05A gate).
SURFACE_DISPOSITIONS = {
    ('managed-offline', v, s): {
        'mode': 'engine_surface_by_contract',
        'ruling': 'PM-Agent-01 2026-10-31 managed-offline per-state reclassification (option b)',
        'citations': [
            'crates/engine/src/preferences/resolver.rs: ProviderGenerationState / offline_valid_until / ManagedPin are resolver-level only, injected solely in resolver_golden_tests',
            'no product / CLI / MCP / env / file / flag path activates a managed provider (dev finding gp_cm05e_managed_offline_mechanism_finding.md)',
            'GLOBAL_PREFERENCES_PRODUCTION_ACCEPTANCE_CONTRACT.md:179-180 (managed/offline fixtures cannot activate a managed provider)',
        ],
        'engine_coverage': ('managed-offline', v, 'engine'),
    }
    for v in ('inactive', 'expired', 'unavailable', 'offline')
    for s in ('gui', 'cli', 'mcp')
}
SURFACE_DISPOSITIONS.update({
    # daemon-restart + no-foreground-tty engine surfaces removed from human-agent-authority
    # (audit cb67dad4): neither is engine-reachable; owner-visible in qualification_followups.
    ('human-agent-authority', 'daemon-restart', 'engine'): {
        'mode': 'scope_exit',
        'reason': 'close_session->Expired is an acceptance-lifecycle refusal, not a real daemon-process restart (new-instance/empty-state/new daemon_id) seam; audit reclassified engine-unreachable at b8e9085b/cb67dad4',
        'owner_gate': 'GP-CM05A',
        'qualification_followup': True,
    },
    ('human-agent-authority', 'no-foreground-tty', 'engine'): {
        'mode': 'scope_exit',
        'reason': 'no TTY/foreground concept exists in the engine product service or PreferenceActorV1 (grep-confirmed, cited in the human_agent_authority module header); its real home is the CLI/MCP surface',
        'owner_gate': 'GP-CM05A',
        'qualification_followup': True,
    },
})

# Pre-reduction owed surface sets for the variants whose owed-set was REDUCED (not deleted) by
# the GP-CM05E surface-disposition co-move. matrix.py requires every surface present in a
# baseline set but absent from the current VARIANTS owed-set to carry a SURFACE_DISPOSITIONS
# entry (no silent weakening). Deleted variants are disclosed in qualification_followups, not here.
BASELINE_SURFACES = {
    ('managed-offline', 'inactive'): ['engine', 'gui', 'cli', 'mcp'],
    ('managed-offline', 'expired'): ['engine', 'gui', 'cli', 'mcp'],
    ('managed-offline', 'unavailable'): ['engine', 'gui', 'cli', 'mcp'],
    ('managed-offline', 'offline'): ['engine', 'gui', 'cli', 'mcp'],
    ('human-agent-authority', 'no-foreground-tty'): ['engine', 'cli', 'mcp'],
    ('human-agent-authority', 'daemon-restart'): ['engine', 'cli', 'mcp'],
}

# The explicit gate catalog validate_gates admits; it must stay reconciled with gates.commands()
# (the matrix regression suite asserts the two never drift). acceptance-support-tests/-clippy are
# the cb67dad4 feature-ON gates keeping full fixture test+clippy coverage outside the shipping
# workspace gates.
GATES = ['matrix', 'boundary', 'menu', 'resolver', 'daemon', 'private-writer', 'mcp', 'dependencies', 'cargo-resources', 'source-health', 'governance', 'parity', 'traceability', 'progress', 'alignment', 'project-state', 'workspace-tests', 'clippy', 'acceptance-support-tests', 'acceptance-support-clippy', 'checker-regressions']

PRODUCT_BOUNDARY = {'active_global_descriptors': 11, 'active_sections': ['appearance', 'units'], 'reserved_descriptors': 45, 'project_seed_keys': 8, 'project_seed_modes': ['global', 'factory'], 'live_project_following': False, 'mcp_direct_set_or_reset': False, 'excluded': ['reserved descriptor activation', 'aggregate Project Units seed activation', 'Manage Preferences', 'broader Project Preferences', 'Publish', 'Revision']}

BACKENDS = ("wayland", "x11")
SCALES = (1.0, 2.0)
WINDOWS = ((1024, 900), (720, 760))

# GP-CM05E coordinate-level COVERAGE DISPOSITIONS -- the SURFACE_DISPOSITIONS analog for the timed
# (measurement) and native corpora. The full-matrix coordinate space (BACKENDS x SCALES x WINDOWS)
# and the matrix protocol's declared scale_factors are UNCHANGED; a disposition removes a coordinate
# class from the pin-STRICT REQUIRED set only, with its ruling + resolvable citations, so
# measurement_inventory()/validate_native require EXACTLY (full matrix - dispositioned) and never a
# silent partial. Both modes are owner-visible qualification_followups ruled at the GP-CM05A gate:
#   env_blocked -- capturable nowhere on the reference display this pin (the scale-2 coordinate
#                  classes); the removed rows are OWED, retained as feasibility disclosure.
#   scope_exit  -- structurally out of the pinned candidate's acceptance scope (ruled at GP-CM05A).
# The four scale-2 GUI coordinate classes: device extents (e.g. 1024x900@2 = 2048x1800) exceed the
# reference 1920x1080 :1 display, the owned Weston clamps, and the realized logical viewport mis-binds
# the matrix coordinate, so production is refused rather than falsely stamped (verified live via
# xdotool getdisplaygeometry; measurement AND capture surfaces alike).
ENV_BLOCKED_SCALE2_COORDINATES = (
    ("wayland", 2.0, (1024, 900)), ("wayland", 2.0, (720, 760)),
    ("x11", 2.0, (1024, 900)), ("x11", 2.0, (720, 760)),
)
MEASUREMENT_DISPOSITIONS = {
    'scale-2-environment-blocked': {
        'mode': 'env_blocked',
        'coordinates': ENV_BLOCKED_SCALE2_COORDINATES,
        'ruling': 'PM-Agent-01 2026-10-31: the four scale-2 GUI coordinate classes are '
                  'environment-blocked on the reference 1920x1080 :1 display, so their 32 measurement '
                  'variants (gui-feedback 6x4 + native-window-open 1x4 + window-lifecycle 1x4) are '
                  'OWED rather than falsely stamped; the strict measurement required set is 90 - 32 = 58.',
        'citations': [
            'specs/active_frontier.json GP-CM05E ruling: the four scale-2 coordinate classes stay '
            'OWED with the device-extent/clamp/mis-bind mechanism and the three-step remedy order',
            'frontend-dev-01 scale2 FINDING (device=logical x2 exceeds screen; verified live via '
            'xdotool getdisplaygeometry, :1 unchanged at 1920x1080)',
            'frontend ENVELOPE-EQUIVALENCE-ASSESSMENT.md (audit-reviewed; owner-bar at GP-CM05A)',
        ],
        'owner_gate': 'GP-CM05A',
        'qualification_followup': True,
    },
}

# The native observation corpus (N01-N06 at the eight coordinates = 48) is a STRUCTURAL scope-exit
# for the pinned candidate: no recipe in the candidate input closure (blob 1ebf0783, 4030 entries)
# can serve a native fixture under the governed Section 9 independent-replay bar. The contract
# (Section 584) makes native replay live re-invoke `python3 <recipe.path> --native-scenario <id>
# --environment <id>` and parse datum.preferences.native-replay.v2; report.py:124-127 enforces that
# exact argv. accessibility_capture.py is a module with no CLI entry (fails invocability); the
# invocable a11y case recipes reject the 2-tuple native coverage (their parser requires the 5-tuple
# case coverage) and do not compute the N0x assertion inventories; the only `--native-scenario`
# recipe (scripts/fixture.py) is synthetic-only, not in the closure, and declared never-readiness
# (test_support: "Synthetic regression fixtures only"). A conformant in-closure native recipe cannot
# be added without committing a file = a new candidate identity (revision+tree+manifest) that would
# invalidate every row captured at this pin. So native coverage cannot reach candidate_ready here and
# is carried to the GP-CM05A owner gate as a disclosed structural gap, never a pass; the conformant
# native recipe + driver are built outside the window (the redefined native unit) and enter the
# closure at the next pin. This is removed from the registry at the pin that admits that recipe.
# PM-Agent-01 2026-10-31 scope-exit classification, disposition (B)+build; b8e9085b scope-exit
# precedent (daemon-restart / no-foreground-tty), larger scale.
NATIVE_DISPOSITIONS = {
    'native-corpus-structural-scope-exit': {
        'mode': 'scope_exit',
        'scope': 'all',
        'ruling': 'The 48 native observations cannot reach candidate_ready at the pinned candidate: '
                  'no in-closure recipe satisfies membership + --native-scenario invocability + '
                  '2-tuple native coverage + native-replay.v2 + N0x assertion computation; adding one '
                  'requires a re-pin (new candidate identity) that invalidates this pin. Scope-exited '
                  'to GP-CM05A; the strict native required set is the empty set.',
        'citations': [
            'specs/GLOBAL_PREFERENCES_PRODUCTION_ACCEPTANCE_CONTRACT.md Section 584 (native replay '
            're-invokes the candidate-bound recipe with --native-scenario/--environment -> '
            'native-replay.v2; true or a nonempty arbitrary log cannot stand in for replay)',
            'report.py:124-127 (validate_review native branch enforces the exact argv and the '
            'native-replay.v2 canonical result)',
            'candidate input closure blob 1ebf0783 (4030 entries): no conformant native recipe; '
            'accessibility_capture.py is module-only, the a11y case recipes reject 2-tuple coverage, '
            'scripts/fixture.py is synthetic-only and out of closure (test_support synthetic-never-readiness)',
            'roadmap finding gp_cm05e_native_recipe_structural_gap.md (roadmap-agent-01, read-only)',
        ],
        'owner_gate': 'GP-CM05A',
        'qualification_followup': True,
    },
}


def subcases(case, variant):
    # The former multi-subcase variants (surface-parity-negative/preference-write-faults and
    # project-genesis-crash/process-killed) were removed as Class-C scope-exits in the GP-CM05E
    # surface-disposition co-move; every remaining variant is a single default case.
    return ("default",)

TIMED_VARIANTS = {
    "release-query": tuple(f"{verb}/{store}" for verb in ("describe", "list", "get", "search", "explain", "preview-project-units-seed") for store in ("defaults", "populated")),
    "durable-mutation": tuple(f"{verb}/{transport}" for verb in ("set", "reset") for transport in ("standalone", "daemon")),
    "project-genesis": ("factory", "global-missing", "global-populated"),
    "project-validation": ("factory", "global-missing", "global-populated"),
    "gui-feedback": ("search-update", "search-clear", "focus", "explanation-open", "explanation-close", "navigation"),
    "native-window-open": ("open",),
}

NATIVE_ASSERTIONS = {
    "N01": ("ordinary-doorway", "one-owned-window", "close-focus-return", "reopen-durable", "no-orphan"),
    "N02": ("keyboard-all-controls", "all-search-vocabularies", "zero-results-clear", "result-navigation", "explanation-return", "one-focus", "no-trap"),
    "N03": ("set-reset-consumer-timing", "invalid-stale-writer-refusals", "draft-retained", "truthful-explanation", "last-valid-value"),
    "N04": ("global-factory-genesis", "eight-preview-receipt-values", "global-refusal-draft-focus", "no-fallback", "existing-project-unchanged"),
    "N05": ("motion-on-off", "contrast-on-off", "word-shape-provenance", "controls-reachable", "no-motion-dependent-action"),
    "N06": ("at-spi-tree-events", "name-role-value-state", "traversal-provenance", "polite-assertive-updates", "focus-return", "screen-reader-spoken-output"),
}
