# A1 measured findings and final disposition

**The corrected fixed batch completed. It measured conditional GPU costs, but
unstable marker/completion behavior prevents a reliable division into unavoidable
backend cost and avoidable Datum overhead. A1 ends here; no additional samples or
renderer redesign are recommended from this evidence.**

The useful findings are: the copy-only graph's median is about1ms in both cases;
omitting restoration lowers the GPU interval by about1ms; omitting the atlas
attachment/resolve lowers it by roughly0.5ms. Frozen preparation saves only
about0.025ms. Painter and marker effects do not satisfy the predeclared consistency
rule. Crucially, some GPU reductions do not translate into completion-time
reductions. These observations do not establish recoverable production savings.

## Setup differences resolved before timing

The terminal fallback was material to the submitted work: every selected tile
submits Workspace glyph batches, without per-glyph CPU culling to its scissor.
The initializer now projects the archived closed-dock `shell 1` tab, including
its close glyph and geometry, instead of the empty-list `terminal` fallback.

The earlier claim that the archived status lacked DRC68 was a visual-reading
error. Exact comparison showed the original status strip already matched. A
provisional setup that removed DRC68 and omitted the space in `shell 1` failed
its pre-timing pixel check:299different pixels perimage. That failure, executable
and images are preserved. The final initializer leaves the original status
unchanged and corrects the tab. Two successful diagnostic builds and two untimed
setup processes were used; no measured samples were spent during those repairs.

Before timing, all four H/V before/after images matched every pixel of the
archived terminal/status strip(y742..800), with no mask or tolerance. Actual
single-board layout, camera, selection, focus, pointers and hit-tested hover were
compared with original native receipt sequences409/1048. Board geometry and
crosshair arms were checked against independent archived image evidence and
visually inspected. The measured batch's two reference PNGs are byte-identical
to the corrected setup images. See [chrome assessment](chrome-assessment.md),
[comparison](comparison.json), [H](reference-0.png) and [V](reference-1.png).

The final executable and complete launch configuration were sealed before entry
and preserved outside Cargo. SHA-256:
`aae1a8a798539c3fcbc2ee9204be5f9b05ea5bd6aeb1c955fb7adc308868bb49`.
Its offline negative control passed. The seven controls, original analysis,
workload coordinates, order, correctness requirements and resource limits did
not change. Only terminal view-state initialization differs from the previously
independently reviewed setup source. No normal renderer change was made.

## Execution and evidence validity

One corrected measured process completed in20.872773seconds on the pinned
P630/Vulkan/Mesa25.0.7, X11,1280×800,scale1,Bgra8UnormSrgb,exact8,Fifo configuration.
It emitted exactly646observations:16conformance,14correctness,56warmup and
560measured. No retry, discarded sample or additional tail collection occurred.
The original560wrong-layout timings remain invalid and excluded, not erased.

All630sample observations passed exact acquired-target byte comparison. N/D/M/P
agree with the full reference; A/B/C agree with the same deliberate prefix hybrid
for each case. D/P prepared payload, ordered commands and upload parity checks
passed. Each case selected84whole32pixel tiles. Preparation-upload byte counts
were zero in this fixed resident fixture; that is not a claim about cold/native
upload workloads. The16clock/operation conformance observations completed with
expected bytes, positive period and complete nonreversed queries. Conformance
is not evidence of zero timing perturbation.

The diagnostic submission counter ended at1941/2200. Maximum sampled tracked
GPU ownership was90,099,364bytes, below512MiB. The64MiB diagnostic CPU guard passed
(no numeric peak was emitted); numeric output was2,311,760bytes, below16MiB.
These are diagnostic accounting checks, not complete DRM/lifetime/peak acceptance.

[analysis.json](analysis.json) contains every cell's five block medians, range,
overall median and maximum for GPU and all CPU/acquire/present/readback phases.
The unchanged analysis script and every raw record are in [raw.tar.gz](raw.tar.gz).
No percentiles were subtracted or combined into an additive attribution.

## Measured resident-graph costs

GPU interval medians in milliseconds; these exclude common reset, preparation,
query readback and image readback, but include the final C-to-native-target copy.

| Control | H | V |
|---|---:|---:|
| D: full resident graph, outer queries | 4.670 | 4.534 |
| M: additional production markers/pass queries | 4.593 | 4.642 |
| P: identical graph, frozen preparation | 4.112 | 4.731 |
| A: suffix draws omitted | 2.451 | 2.739 |
| B: restoration also omitted | 1.491 | 1.570 |
| C: atlas pass/resolve also omitted; copies remain | 1.000 | 0.995 |

The copy-only control's five GPU block medians range0.913–1.089ms(H) and
0.910–1.071ms(V), but its individual maxima reach6.017ms and5.818ms. Its
submit-to-completion medians are4.696ms and4.985ms. A roughly1ms GPU median is
therefore neither a guaranteed deadline nor a minimum memory-bandwidth result.
It includes the chosen fragmented tile copies, final full presentation copy and
backend transitions under the diagnostic's initialized state.

B's medians are below4ms in both cases; this batch does **not** support the claim
that the tested image-operation graph necessarily consumes4ms without painters.
Nor does it demonstrate a complete usable renderer meeting the production budget.

## Predeclared contrasts

Values below are medians of the five **within-block differences of cell medians**,
not differences of the overall medians in the table above. A material signal
requires all five differences to have the same sign, magnitude above0.25ms and
above5% of its declared control. Failure of that rule is not proof of equivalence.

| Contrast, ms | H | V | Result |
|---|---:|---:|---|
| D−N completion: query scheme | −0.142 | −0.111 | Mixed signs; unresolved |
| M−D GPU: production markers | +0.556 | +0.233 | Mixed signs; unresolved |
| M−D completion | +0.042 | +0.428 | Mixed signs; unresolved |
| D−P preparation | +0.02505 | +0.02496 | Consistently positive, below materiality |
| D−A GPU: painter/content | +2.009 | +1.109 | Sign reversals; unresolved |
| A−B GPU: restoration | +1.005 | +1.080 | GPU rule met in both cases |
| B−C GPU: atlas attachment/resolve | +0.463 | +0.550 | GPU rule met in both cases |

**Markers.** H's M−D GPU block differences range−1.723..+3.439ms; V's range
−0.397..+3.350ms. Completion contrasts also change sign. This does not establish
negligible markers, a stable removable marker tax, or the cause of historical
native tails. Do not subtract an estimated marker cost to obtain a passing frame.

**Preparation.** D's CPU preparation medians are0.06518/0.06402ms; P's are
0.03985/0.03837ms. The approximately25microsecond block contrast is real at this
fixed derivation's descriptive scope, but below A1's material threshold. P has
identical checked GPU commands, yet its GPU/completion values also vary; its
lower H GPU median cannot be attributed to a different GPU algorithm. The evidence
does not support preparation optimization as the remedy for the GPU budget gap.

**Painter/restoration.** D−A reverses sign in one H and two V blocks, so the
positive typical difference is not a reproducible painter-cost estimate under
A1's rule. Useful painter work and potentially avoidable dispatch remain mixed.
A−B has a positive GPU difference in all ten case/block comparisons, supporting
a conditional observation that this restoration sequence costs GPU time. It does
not show that restoration is unnecessary or that its roughly1ms is recoverable
by a cheaper correct implementation.

**Image operations and wall consistency.** B−C is positive in every GPU block,
but completion tells a different story: median within-block B−C completion is
−0.740ms(H) and−0.561ms(V), with mixed signs. Removing resolve did not consistently
make completion faster. A−B completion is positive in all V blocks but reverses
once in H. The [completion consistency check](completion-consistency.json) uses
only the already-collected wall phases, without changing selection thresholds.
The predeclared rule requires these contradictions to limit GPU-only attribution:
resolve's conditional GPU signal is **not** an established end-to-end saving;
restoration's end-to-end magnitude is also unresolved. No cause for the
GPU/wall disagreement is established by this packet.

## Interpretation and completion decision

**What belongs to required backend operations?** The tested copies, resolve and
restoration have observable costs above. They are this implementation's operations
under common reset, serialized completion and diagnostic native COPY_SRC—not
universal costs required by exact8 rendering. Diagnostic readback lies outside
the GPU bracket but changes cadence/cache state; COPY_SRC may affect allocation
or transitions. Those perturbations were not independently measured.

**What is avoidable Datum overhead?** The fixed CPU preparation bypass demonstrates
only about0.025ms. Painter and marker overhead cannot be isolated reliably here;
restoration/resolve GPU contrasts do not establish a correct cheaper alternative
or stable end-to-end savings. No defensible percentage partition of the historical
native p95 follows from these measurements.

**Does this justify further optimization?** Not another renderer redesign,
preparation campaign or search for4ms. There is conditional GPU cost in the
restoration/resolve sequence, but the measured completion contradictions and
unresolved painter/marker effects do not establish an economically worthwhile
production optimization. A1 ends with measured limited findings and overall
end-to-end attribution inconclusive. No additional experiment or implementation
is started or offered as an automatic successor.

Prior renderer fixes retain their existing evidence. The valid R4 native p95
6.700167ms/p99 10.205084ms still fail the4/8ms requirements; this diagnostic neither
replaces those measurements nor decomposes their tails. The earlier4.330583ms
candidate remains a separate-method observation, not a matched comparison or
proof that another0.33ms is worth pursuing. Host4ms feasibility across other
strategies remains unknown, not disproved.

Complete DRM lifetime/duty, broader recovery/endurance/consumer coverage and
independent runtime replay remain separate qualification gaps. Production source
and R4 binary are unchanged after restoring temporary diagnostic edits. Main GPU
issue and S4/S5 remain open. CPU event-loop optimization remains deferred.
