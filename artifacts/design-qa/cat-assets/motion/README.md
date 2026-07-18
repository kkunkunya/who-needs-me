# Cat state motion (V1, raster)

Applies to Panel Header aggregate-state change and Alert Card entrance.
Tray never animates — it swaps between the three static state frames atomically.

- duration: 200ms
- easing: cubic-bezier(0.2, 0.8, 0.2, 1)
- animated properties: transform, opacity ONLY (no width/height/layout)
- granularity: whole-cat fade/scale (NOT per-part eyes/ears/tail groups — the
  layered-SVG group animation was dropped with the raster pivot, 2026-07-17)
- iteration: plays once, does not loop
- reduced-motion (`prefers-reduced-motion: reduce`): no transition — the target
  state frame is swapped in immediately.

Frames in this folder:
- panel-aggregate-start.png / -end.png : Header cat calm -> attention.
- alert-entrance-start.png / -end.png  : Alert cat opacity 0 + scale 0.96 -> opacity 1 + scale 1.

Live demonstration (with the exact easing/duration and a reduced-motion
fallback) is in ../index.html.
