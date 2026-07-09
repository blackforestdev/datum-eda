# Board Visual Goldens

Layer A render-scene golden fixtures live here.

These are renderer-owned artifacts. Source Datum-native projects may live in
engine testdata, but the rendered PNGs, manifests, and failure artifacts belong
to `gui-render`.

Failure artifacts are written beside the fixture:

- `<fixture>.actual.png`
- `<fixture>.diff.png`
- `<fixture>.report.txt`

The harness design is documented in:

- `docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md`

The Phase 1 D7 board acceptance fixture is:

- `datum-test.fixture.toml`
- `datum-test.known-gaps.md`

It captures the canonical imported KiCad board across the required scale matrix
`1.0, 1.25, 1.5, 2.0`. Owner visual review is recorded separately from the
machine golden comparison.

The first board-text fidelity suite is:

- `text-intent-repro.fixture.toml`
- `text-fidelity-repro.fixture.toml`
- `text-transform-repro.fixture.toml`
- `text-density-repro.fixture.toml`

Golden PNGs are source artifacts. Generated `.actual.png`, `.diff.png`, and
`.report.txt` files are retained only for failing comparisons.
