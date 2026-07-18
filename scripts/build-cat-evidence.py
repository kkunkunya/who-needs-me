#!/usr/bin/env python3
"""Render the Cat asset family review evidence (no character-sheet overwrite).

Outputs under artifacts/design-qa/cat-assets/:
  - contact-sheet.png            state grid, NO text labels (pose + coral accent
                                 must carry the states; attention != sleeping).
  - motion/panel-aggregate-*.png Panel Header calm->attention start/end frames.
  - motion/alert-entrance-*.png  Alert Card entrance start/end frames.
  - tray-menubar-preview.png      Tray micro @1x/@2x on light + dark menu bars.

Motion spec recorded alongside in motion/README.md and the QA index.html:
  200ms · cubic-bezier(0.2,0.8,0.2,1) · transform+opacity only (whole-cat
  fade/scale) · no loop · reduced-motion = immediate frame swap.

Run: python3 scripts/build-cat-evidence.py
"""
from __future__ import annotations

import base64
import mimetypes
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
CAT = ROOT / "assets" / "cat"
OUT = ROOT / "artifacts" / "design-qa" / "cat-assets"

PANEL = (0x14, 0x22, 0x3A)       # --color-bg-panel night-blue
CANVAS = (0x0D, 0x14, 0x24)      # --color-bg-canvas
HAIRLINE = (180, 210, 235)       # hairline base (used at low alpha)
TEXT_PRIMARY = (0xF1, 0xF5, 0xF7)
TEXT_SECONDARY = (0x91, 0xA6, 0xB8)


def load(rel: str) -> Image.Image:
    return Image.open(CAT / rel).convert("RGBA")


def fit(img: Image.Image, h: int) -> Image.Image:
    w = max(1, round(img.width * h / img.height))
    return img.resize((w, h), Image.LANCZOS)


def cell(bg, size, img, hairline=True):
    c = Image.new("RGBA", size, (*bg, 255))
    if hairline:
        # 1px inner hairline
        px = c.load()
        for x in range(size[0]):
            for y in (0, size[1] - 1):
                px[x, y] = (*HAIRLINE, 36)
        for y in range(size[1]):
            for x in (0, size[0] - 1):
                px[x, y] = (*HAIRLINE, 36)
    c.alpha_composite(img, ((size[0] - img.width) // 2, (size[1] - img.height) // 2))
    return c


def contact_sheet():
    """Labelless state grid: micro row, compact row, illustration — pose + accent only."""
    cw, ch = 220, 180
    rows = [
        ["masters/cat-micro-calm.png", "masters/cat-micro-working.png", "masters/cat-micro-attention.png"],
        ["masters/cat-compact-calm.png", "masters/cat-compact-working.png", "masters/cat-compact-attention.png"],
        ["masters/cat-illustration-sleeping.png", None, None],
    ]
    heights = [96, 118, 128]
    sheet = Image.new("RGBA", (cw * 3, ch * len(rows)), (*CANVAS, 255))
    for r, row in enumerate(rows):
        for c, rel in enumerate(row):
            if rel is None:
                continue
            img = fit(load(rel), heights[r])
            sheet.alpha_composite(cell(PANEL, (cw, ch), img), (c * cw, r * ch))
    OUT.mkdir(parents=True, exist_ok=True)
    sheet.convert("RGB").save(OUT / "contact-sheet.png")
    print("wrote contact-sheet.png")


def _panel_header(state: str, size=(520, 132)):
    """A schematic Panel Header surface with the compact cat at the left."""
    c = Image.new("RGBA", size, (*PANEL, 255))
    cat = fit(load(f"compact/cat-compact-{state}@2x.webp"), 72)
    c.alpha_composite(cat, (28, (size[1] - cat.height) // 2))
    return c


def panel_aggregate_frames():
    (OUT / "motion").mkdir(parents=True, exist_ok=True)
    _panel_header("calm").convert("RGB").save(OUT / "motion" / "panel-aggregate-start.png")
    _panel_header("attention").convert("RGB").save(OUT / "motion" / "panel-aggregate-end.png")
    print("wrote motion/panel-aggregate-{start,end}.png")


def _alert_card(scale: float, opacity: float, size=(460, 150)):
    """Alert Card entrance frame — whole-cat transform+opacity only."""
    card = Image.new("RGBA", size, (*PANEL, 255))
    base = load("compact/cat-compact-attention@2x.webp")
    h = max(1, round(84 * scale))
    cat = fit(base, h)
    if opacity < 1.0:
        a = cat.split()[3].point(lambda v: int(v * opacity))
        cat.putalpha(a)
    # cat anchored at a fixed baseline so card text would not shift
    baseline_y = 33 + (84 - cat.height) // 2
    card.alpha_composite(cat, (28, baseline_y))
    return card


def alert_entrance_frames():
    (OUT / "motion").mkdir(parents=True, exist_ok=True)
    _alert_card(0.96, 0.0).convert("RGB").save(OUT / "motion" / "alert-entrance-start.png")
    _alert_card(1.0, 1.0).convert("RGB").save(OUT / "motion" / "alert-entrance-end.png")
    print("wrote motion/alert-entrance-{start,end}.png")


def tray_menubar_preview():
    """Tray micro on light and dark menu bars, @1x and @2x, to expose the
    accepted full-colour raster risk (faint on a light bar)."""
    bar_dark = (0x1E, 0x1E, 0x22)
    bar_light = (0xF2, 0xF2, 0xF4)
    states = ["calm", "working", "attention"]
    pad = 10
    tile = 40
    w = pad + len(states) * 2 * tile + pad
    strip_h = tile + 2 * pad
    canvas = Image.new("RGBA", (w, strip_h * 2), (*CANVAS, 255))
    for i, bar in enumerate((bar_dark, bar_light)):
        strip = Image.new("RGBA", (w, strip_h), (*bar, 255))
        x = pad
        for state in states:
            for scale in (1, 2):
                icon = load(f"micro/cat-micro-{state}@{scale}x.png")
                strip.alpha_composite(icon, (x + (tile - icon.width) // 2, pad + (tile - icon.height) // 2))
                x += tile
        canvas.alpha_composite(strip, (0, i * strip_h))
    canvas.convert("RGB").save(OUT / "tray-menubar-preview.png")
    print("wrote tray-menubar-preview.png")


def motion_readme():
    txt = """# Cat state motion (V1, raster)

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
"""
    (OUT / "motion").mkdir(parents=True, exist_ok=True)
    (OUT / "motion" / "README.md").write_text(txt)
    print("wrote motion/README.md")


def _data_uri(path: Path) -> str:
    mime = mimetypes.guess_type(str(path))[0] or "application/octet-stream"
    return f"data:{mime};base64," + base64.b64encode(path.read_bytes()).decode("ascii")


def _cat_uri(rel: str) -> str:
    return _data_uri(CAT / rel)


def qa_html():
    """Self-contained QA harness (data-URI embedded) so it renders correctly with
    `python3 -m http.server 4173 --directory artifacts/design-qa/cat-assets`."""
    states = ["calm", "working", "attention"]

    def tier_cells(tier, ext):
        out = []
        for s in states:
            u1 = _cat_uri(f"{tier}/cat-{tier}-{s}@1x.{ext}")
            u2 = _cat_uri(f"{tier}/cat-{tier}-{s}@2x.{ext}")
            out.append(
                f'<figure><div class="tile"><img src="{u2}" alt="{tier} {s}" '
                f'class="{tier}"></div><figcaption>{s}<br><span class="dim" '
                f'data-u1="{u1}" data-u2="{u2}"></span></figcaption></figure>'
            )
        return "\n".join(out)

    illus = _cat_uri("illustration/cat-illustration-sleeping@2x.webp")
    calm_c = _cat_uri("compact/cat-compact-calm@2x.webp")
    attn_c = _cat_uri("compact/cat-compact-attention@2x.webp")
    contact = _data_uri(OUT / "contact-sheet.png")
    tray_prev = _data_uri(OUT / "tray-menubar-preview.png")

    html = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Cat runtime asset family — QA (WhoNeedsMe #15)</title>
<style>
  :root {{
    --canvas:#0D1424; --panel:#14223A; --hairline:rgba(180,210,235,0.14);
    --text:#F1F5F7; --muted:#91A6B8; --attention:#F05A5D;
    --dur:200ms; --ease:cubic-bezier(0.2,0.8,0.2,1);
  }}
  * {{ box-sizing:border-box; }}
  body {{ margin:0; background:var(--canvas); color:var(--text);
    font:14px/1.5 -apple-system,BlinkMacSystemFont,"SF Pro Text",sans-serif; padding:32px; }}
  h1 {{ font-size:20px; margin:0 0 4px; }}
  h2 {{ font-size:15px; margin:36px 0 12px; color:var(--text); }}
  p.note {{ color:var(--muted); max-width:70ch; margin:4px 0 0; }}
  code {{ font-family:"SF Mono",ui-monospace,monospace; color:#B1C3D0; }}
  .panel {{ background:var(--panel); border:1px solid var(--hairline);
    border-radius:15px; padding:20px; }}
  .row {{ display:flex; gap:24px; flex-wrap:wrap; align-items:flex-end; }}
  figure {{ margin:0; text-align:center; }}
  figcaption {{ color:var(--muted); margin-top:8px; font-size:12px; }}
  .tile {{ display:flex; align-items:center; justify-content:center;
    min-width:96px; min-height:96px; }}
  img {{ image-rendering:auto; }}
  img.micro {{ width:18px; height:18px; }}
  img.compact {{ height:32px; }}
  img.illustration {{ width:200px; }}
  .zoom img.micro {{ width:72px; height:72px; image-rendering:pixelated; }}
  .zoom img.compact {{ height:128px; image-rendering:pixelated; }}
  button {{ background:#192B49; color:var(--text); border:1px solid var(--hairline);
    border-radius:9px; padding:7px 14px; font-size:13px; cursor:pointer; }}
  /* --- motion demo: whole-cat transform+opacity only --- */
  .stage {{ position:relative; width:96px; height:96px; }}
  .stage img {{ position:absolute; inset:0; margin:auto; height:64px; width:auto;
    transition:opacity var(--dur) var(--ease), transform var(--dur) var(--ease); }}
  .agg .calm {{ opacity:1; }} .agg .attn {{ opacity:0; transform:scale(0.96); }}
  .agg.on .calm {{ opacity:0; transform:scale(0.96); }}
  .agg.on .attn {{ opacity:1; transform:scale(1); }}
  .alertcat {{ height:64px; transition:opacity var(--dur) var(--ease), transform var(--dur) var(--ease); }}
  .alertcat.enter {{ opacity:0; transform:scale(0.96); }}
  .bar {{ display:flex; gap:14px; padding:10px 14px; border-radius:8px; align-items:center; }}
  .bar.dark {{ background:#1E1E22; }} .bar.light {{ background:#F2F2F4; }}
  .bar img {{ width:18px; height:18px; }}
  .alertcard {{ display:flex; align-items:center; gap:16px; width:420px;
    background:var(--panel); border:1px solid var(--hairline); border-radius:15px;
    box-shadow:0 2px 6px rgba(0,0,0,.28),0 18px 44px rgba(0,0,0,.34); padding:16px 20px; }}
  .alertcard .fact {{ font-weight:600; }} .alertcard .sub {{ color:var(--muted); font-size:12px; }}
  .alertcard .cta {{ margin-left:auto; }}
  @media (prefers-reduced-motion: reduce) {{
    .stage img, .alertcat {{ transition:none !important; }}
  }}
</style></head>
<body>
  <h1>Cat runtime asset family — QA</h1>
  <p class="note">WhoNeedsMe V1 · night-blue dark-only surface (<code>--color-bg-panel #14223A</code>).
  Raster masters from gpt-image-2, retoned to a light figure for the dark panel; the one warm
  <code>--color-attention #F05A5D</code> accent appears on <b>attention</b> only. No SVG groups.</p>

  <h2>1 · Asset family on the panel surface</h2>
  <div class="panel">
    <div class="row" style="margin-bottom:22px"><b class="dim2">micro / Tray (18px)</b></div>
    <div class="row">{tier_cells("micro","png")}</div>
    <div class="row zoom" style="margin-top:8px">{tier_cells("micro","png")}</div>
    <div class="row" style="margin:26px 0 0"><b>compact / Panel Header · Alert (32px)</b></div>
    <div class="row">{tier_cells("compact","webp")}</div>
    <div class="row" style="margin:26px 0 12px"><b>illustration / Empty State (200px)</b></div>
    <div class="row"><figure><div class="tile" style="min-height:160px">
      <img class="illustration" src="{illus}" alt="sleeping"></div>
      <figcaption>sleeping (calm)</figcaption></figure></div>
  </div>

  <h2>2 · Contact sheet — no text labels (pose + coral accent must carry state)</h2>
  <div class="panel"><img src="{contact}" alt="contact sheet" style="max-width:100%;border-radius:8px"></div>

  <h2>3 · Motion — 200ms · cubic-bezier(0.2,0.8,0.2,1) · transform+opacity · no loop · reduced-motion swaps</h2>
  <div class="panel">
    <div class="row">
      <figure>
        <div class="stage agg" id="agg">
          <img class="calm" src="{calm_c}" alt="calm"><img class="attn" src="{attn_c}" alt="attention">
        </div>
        <figcaption>Panel Header aggregate change<br><button onclick="document.getElementById('agg').classList.toggle('on')">calm ⇄ attention</button></figcaption>
      </figure>
      <figure>
        <div class="alertcard">
          <img class="alertcat" id="alertcat" src="{attn_c}" alt="attention">
          <div><div class="fact">1 session needs you</div><div class="sub">waiting for your approval</div></div>
          <button class="cta">Open panel</button>
        </div>
        <figcaption>Alert Card entrance<br><button onclick="replayAlert()">replay entrance</button></figcaption>
      </figure>
    </div>
    <p class="note">Granularity is whole-cat fade/scale — the per-part SVG group animation was dropped with the
    raster pivot (2026-07-17). Start/end frames also exported to <code>motion/</code>.</p>
  </div>

  <h2>4 · Tray on menu bars (accepted full-colour raster risk: faint on a light bar)</h2>
  <div class="panel">
    <div class="row">
      <div class="bar dark"><img src="{_cat_uri('micro/cat-micro-calm@2x.png')}"><img src="{_cat_uri('micro/cat-micro-working@2x.png')}"><img src="{_cat_uri('micro/cat-micro-attention@2x.png')}"></div>
      <div class="bar light"><img src="{_cat_uri('micro/cat-micro-calm@2x.png')}"><img src="{_cat_uri('micro/cat-micro-working@2x.png')}"><img src="{_cat_uri('micro/cat-micro-attention@2x.png')}"></div>
    </div>
    <p class="note">Static preview export: <code>tray-menubar-preview.png</code>. Real macOS menu-bar
    hardware evidence (normal/retina, light/dark) is docked to Kun to capture on-device.</p>
    <img src="{tray_prev}" alt="tray preview" style="margin-top:8px;border-radius:8px">
  </div>

  <script>
    function replayAlert() {{
      const c = document.getElementById('alertcat');
      c.classList.add('enter'); void c.offsetWidth; c.classList.remove('enter');
    }}
    // fill density captions
    document.querySelectorAll('.dim').forEach(s => {{ /* placeholder for @1x/@2x note */ }});
  </script>
</body></html>
"""
    (OUT / "index.html").write_text(html)
    print("wrote index.html")


def main() -> int:
    contact_sheet()
    panel_aggregate_frames()
    alert_entrance_frames()
    tray_menubar_preview()
    motion_readme()
    qa_html()
    print("Cat evidence rebuilt.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
