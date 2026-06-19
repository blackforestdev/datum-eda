Golden fixtures for the Phase 2 text engine live here.

Purpose:
- deterministic outline flatten fixtures
- text geometry snapshot fixtures
- backend parity fixtures when multiple backends exist
- native visual fidelity projects under `native/`

Phase 2D rule:
- at least one vendored outline font fixture must serialize to byte-identical
  canonical JSON on the supported CI architecture

Native visual fidelity fixtures:
- `native/text-intent-repro`
- `native/text-fidelity-repro`
- `native/text-transform-repro`
- `native/text-density-repro`

These projects are documented in:
- `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md`
