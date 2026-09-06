# Independent diagnostic execution failure

This extra diagnostic is not one of the five mandatory passing replay scenarios.
The same identified binary received the normal six missing-consumer attempts,
then three wheel-up events over history. `overscroll-three-up.png` was captured
and independently inspected: history is blank despite the six existing records.

The reviewer then requested `["wheel", -3]` to return. This was an invalid use
of the capture runner's wheel API, which supports positive wheel-up repeats:

```text
RuntimeError: ('xdotool', 'click', '--repeat', -3, '--delay', '120', '4'):
Invalid repeat value '-3' (must be >= 1)
```

The subprocess exited 1 at `workflow_delivery_pilot_capture.py:236` via
`Capture.run`. The runner's `finally` preserved source comparisons and closed
its owned processes. No `overscroll-return.png` exists and no successful normal
close or recovery is claimed for this partial diagnostic. Requested and
completed input lists, GUI/state logs, native accessibility logs and screenshots
are retained unchanged. This failure is reviewer invocation error, not evidence
of a product crash. No mandatory replay run was replaced or edited.
