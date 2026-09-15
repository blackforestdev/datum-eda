# Type Colour / Grey Value (Grauwert) — Research + Datum Formula

> Agent-assisted research (2026-07-08) into the typographic principle of softening
> large/heavy type toward grey for **even tonal "colour"**, and its distillation into
> a formula for the Datum Rendering Book. Research-only. Owner's instinct confirmed:
> the principle is canonical and the book he was picturing is **Emil Ruder,
> *Typographie* (1967)**.

## The principle (canonical)
On a light ground, large/heavy black type is a **large dark mass that optically
overpowers small text**; reduce its tonal value toward the ground (grey it down) so
**perceived weight is even across sizes**. Names/terms:
- **Type colour / typographic colour** — the apparent greyness a block of type
  projects, independent of hue; good setting has an *even colour* (Type color, Wikipedia).
- **Grauwert (grey value)** — the German-Swiss term; the identical black form reads as
  a lighter or darker grey depending on size/spacing/mass.
- **"Even grey" page** — the classical ideal (Tschichold).
- **Optical (perceived) vs actual weight** + **size–tone contrast** (Albers lineage) —
  the perceptual mechanism.

## Sources (prioritised)
- **Emil Ruder, *Typographie: A Manual of Design* (Niggli, 1967)** — the canonical
  Swiss source; dedicated **"Shades of grey" / Grauwerte** treatment; the direct root of
  "lighten the heavy mass to balance the colour." *The book the owner is picturing.*
  <https://www.typotheque.com/books/typography-a-manual-of-design>
- **Robert Bringhurst, *The Elements of Typographic Style*** (Part 2) — names/standardises
  **"typographic colour"** in English. <https://en.wikipedia.org/wiki/The_Elements_of_Typographic_Style>
- **Josef Müller-Brockmann, *Grid Systems*** — type as a "grey surface"/tonal field.
- **Jan Tschichold, *The Form of the Book*** — the "even grey" ideal.
- **Karl Gerstner, *Designing Programmes*** — the systematise-a-visual-judgement posture.

**Honest caveat:** the classic canon establishes the *principle* rigorously but does not
publish the numeric "display at 70–85% black" figure — those are modern
practitioner/design-system codification, faithful to Ruder. The modern engineering
analogue is the variable-font **grade axis** (adjusts darkness without changing width).
Present the formula as *Ruder's principle, quantified for Datum*, never as a quotation.

## The Datum formula (adopted)
Ground: warm vellum `#E7E1D2` (rel. luminance ≈ 0.87), ink `#25211A` (≈ 0.017); full ink
on vellum ≈ **13.7:1** — a deep reserve, so we can grey substantially and stay legible.

Let `k` = fraction of full ink retained (1.0 = ink; 0.0 = ground). Fill = mix from vellum
toward ink by `k`, interpolated in **OKLab** (perceptually even steps).

```
taper(s) = max(0, log2(s / s_body))          # octaves ABOVE body; 0 for body & smaller
grade(w) = { Regular:0, Medium:1, SemiBold:2, Bold:3 }
R(s,w)   = taper(s) * (a + c*grade(w))
k(s,w)   = clamp(1 - R(s,w), k_min, 1.0)
fill     = oklab_mix(vellum, ink, k)
```
Constants (tuned for vellum/ink): `s_body = 16`, `a = 0.08`, `c = 0.02`, `k_min = 0.62`.

Properties by construction: **body & smaller always full ink** (taper 0 → small text
never softens, incl. bold inline emphasis); **weight only matters once type is large**
(it multiplies the size taper); heroes land in the classic 70–85% band; **grey value is
rotation-invariant** → a title block reads equally balanced horizontal or rotated 90°,
no per-orientation tuning.

### Lookup (Datum scale on `#E7E1D2`, OKLab-mixed)
| Role | Size | Weight | k | Fill |
|------|-----:|--------|----:|------|
| Body / caption / value | ≤16 | any | 1.000 | `#25211A` (ink) |
| Wordmark | 18 | Medium | 0.983 | `#28241D` |
| Title | 21 | Medium | 0.961 | `#2B2720` |
| Landscape hero `02` | 58 | SemiBold | 0.777 | `#4B463E` |
| Portrait hero `02` | 100 | SemiBold | 0.683 | `#5C574E` |

### Legibility floor & guards
- `k_min = 0.62` still yields well above 4.5:1 on this ground (start from 13.7:1); the
  taper only touches *large* text (WCAG allows 3:1 there). Heroes at k≈0.68–0.78 are safe.
- **Only subtract contrast from text that has it to spare (large/heavy).** Never taper
  body, caption, mono data, or anything below `s_body`.
- If Datum adds a darker/low-contrast ground, **re-derive `k_min` from the actual
  ink-vs-ground contrast** so the floor tracks the theme, not a magic number.
- **Mono/data exemption (adopted):** IBM Plex **Mono** is "data" — exempt from the taper
  (always `k = 1.0`) so numeric fields stay crisp regardless of size, while surrounding
  display headings soften. (In title blocks mono is already <16 px, so this only bites if
  mono is ever set large.)

### One-line rule (for the Rendering Book)
> *Type colour (Ruder's Grauwert): body and smaller stays full ink; above 16 px, step the
> fill toward the vellum ground by ~8% per size-octave plus ~2% per weight-step, floored at
> 62% ink — so a 54 px SemiBold hero renders at ~79% ink (a warm grey), giving even tonal
> colour across all sizes and both orientations.* Mono data is exempt (always full ink).

Sources: Type color (Wikipedia); Ruder *Typographie* (Typotheque); Bringhurst *Elements*
(Wikipedia; readings.design PDF); Müller-Brockmann (Neugraphic); Tschichold *Form of the
Book* (Eye); practitioner codification (Imperavi UI Typography, Fontfabric, Nathan Curtis
"Typography in Design Systems").
