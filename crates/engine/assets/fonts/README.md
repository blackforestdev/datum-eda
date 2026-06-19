Phase 2 text-engine font assets live here.

Rules:
- required product fonts are vendored here at build time
- no runtime OS font discovery is allowed for required behavior
- every vendored font must be recorded in `FONT_PROVENANCE.md`
- deterministic outline-fixture tests read fonts from this directory

Planned default bundle:
- `newstroke` (stroke dataset, CC0)
- `inter`
- `ibm_plex_sans_condensed`
- `inter_display`
- `jetbrains_mono`
