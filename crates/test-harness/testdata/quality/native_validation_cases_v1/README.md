# Native Validation Cases v1

This directory contains small checked-in native-project fixtures used only for
the structural native validation gate.

Cases:
- `duplicate-pad-uuid-invalid`: board pads intentionally reuse one authored UUID
- `missing-sheet-file-invalid`: schematic root references a missing sheet file
- `unsupported-schema-version-invalid`: board root uses an unsupported schema version

These fixtures are exercised through:

```bash
python3 scripts/check_native_project_fixtures.py
```

The checked-in expectations live in
`crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json`.
Together with `route_strategy_curated_baseline_v1`, that manifest currently
covers every checked-in native project root under
`crates/test-harness/testdata/quality`.
