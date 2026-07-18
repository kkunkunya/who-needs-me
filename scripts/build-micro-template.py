#!/usr/bin/env python3
"""
Tray micro glyph builder for issue #15.

Two experiments in one script:
  STEP 1 (raster enlarge): crop the alpha bbox of the existing retoned raster
          master and scale it to fill an 18px frame, then render on light AND
          dark menu-bar backgrounds. Demonstrates the physics failure: a light
          cat stays light -> invisible on a light menu bar no matter the scale.
  STEP 2 (SVG-template equivalent): draw a minimal head+ears cat glyph as a
          macOS *template* image (monochrome + alpha). macOS auto-inverts a
          template image per menu-bar appearance, so the SAME asset reads on
          both light and dark bars. calm/working are template (black+alpha);
          attention keeps the warm coral #F05A5D non-template (Signal Earns
          Color). We simulate template inversion in the preview.

Runtime PNGs are pure-black (or coral) + alpha at 18px (@1x) and 36px (@2x).
Geometry lives here as primitives; we also emit a matching .svg source so the
template is literally an SVG->PNG derivation (single source of truth).
"""
import os
from PIL import Image, ImageDraw, ImageOps

ROOT = "/Volumes/Storge/personal-project/who-need-me/feat-issue-15-cat-assets"
MICRO = os.path.join(ROOT, "assets/cat/micro")
MASTERS = os.path.join(ROOT, "assets/cat/masters")
QA = os.path.join(ROOT, "artifacts/design-qa/cat-assets")

CORAL = (240, 90, 93)     # #F05A5D
BLACK = (0, 0, 0)

# menu-bar background approximations (translucent bar over neutral desktop)
LIGHT_BAR = (236, 236, 238)
DARK_BAR = (38, 38, 40)

# ---- glyph geometry (0..100 box) ------------------------------------------
# Each state: list of ("fill"/"erase", shape). shape = ("ellipse", cx,cy,rx,ry)
# or ("poly", [(x,y),...]).  Fills union into the silhouette; erases punch holes
# (eyes) so the menu bar shows through -> classic template knockout.

def ears(state):
    if state == "calm":
        # relaxed, angled outward, shorter -> a resting "loaf"
        return [
            ("fill", ("poly", [(22, 46), (18, 16), (47, 38)])),
            ("fill", ("poly", [(78, 46), (82, 16), (53, 38)])),
        ]
    # working / attention: upright, sharp, alert
    return [
        ("fill", ("poly", [(25, 44), (31, 5), (49, 36)])),
        ("fill", ("poly", [(75, 44), (69, 5), (51, 36)])),
    ]

def head(state):
    if state == "calm":
        # sits a touch lower / rounder
        return [("fill", ("ellipse", 50, 64, 34, 29))]
    return [("fill", ("ellipse", 50, 62, 33, 30))]

def eyes(state):
    if state == "calm":
        # closed, serene slits
        return [
            ("erase", ("ellipse", 37, 64, 8, 2.4)),
            ("erase", ("ellipse", 63, 64, 8, 2.4)),
        ]
    # open, alert almond eyes
    return [
        ("erase", ("ellipse", 37, 61, 8, 6.2)),
        ("erase", ("ellipse", 63, 61, 8, 6.2)),
    ]

def glyph(state):
    return head(state) + ears(state) + eyes(state)

# ---- rasterize a glyph mask at target size via supersampling ---------------
def render_mask(state, size):
    S = 32
    W = size * S
    mask = Image.new("L", (W, W), 0)
    d = ImageDraw.Draw(mask)
    def px(v):
        return v / 100.0 * W
    for op, shape in glyph(state):
        val = 255 if op == "fill" else 0
        if shape[0] == "ellipse":
            _, cx, cy, rx, ry = shape
            d.ellipse([px(cx-rx), px(cy-ry), px(cx+rx), px(cy+ry)], fill=val)
        else:
            pts = [(px(x), px(y)) for x, y in shape[1]]
            d.polygon(pts, fill=val)
    return mask.resize((size, size), Image.LANCZOS)

def render_glyph_rgba(state, size, color):
    mask = render_mask(state, size)
    img = Image.new("RGBA", (size, size), color + (0,))
    solid = Image.new("RGBA", (size, size), color + (255,))
    img = Image.composite(solid, img, mask)
    img.putalpha(mask)
    return img

# ---- emit an SVG source (same geometry) ------------------------------------
def emit_svg(state, color):
    hexc = "#%02X%02X%02X" % color
    parts = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" '
             'width="100" height="100">']
    parts.append('<defs><mask id="m">')
    parts.append('<rect width="100" height="100" fill="black"/>')
    for op, shape in glyph(state):
        c = "white" if op == "fill" else "black"
        if shape[0] == "ellipse":
            _, cx, cy, rx, ry = shape
            parts.append(f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{c}"/>')
        else:
            pts = " ".join(f"{x},{y}" for x, y in shape[1])
            parts.append(f'<polygon points="{pts}" fill="{c}"/>')
    parts.append('</mask></defs>')
    parts.append(f'<rect width="100" height="100" fill="{hexc}" mask="url(#m)"/>')
    parts.append('</svg>')
    return "\n".join(parts)

# ---- STEP 1: raster crop+enlarge from existing master ----------------------
def step1_enlarge(state, size):
    src = Image.open(os.path.join(MASTERS, f"cat-micro-{state}.png")).convert("RGBA")
    bbox = src.getbbox()
    crop = src.crop(bbox)
    # fit into (size,size) preserving aspect, minimal padding
    crop.thumbnail((size, size), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.paste(crop, ((size - crop.width)//2, (size - crop.height)//2), crop)
    return canvas

# ---- preview cell ----------------------------------------------------------
def on_bar(glyph_img, bar_color, cell, upscale):
    """Composite an 18px (or 36px) glyph centered on a menu-bar swatch, then
    nearest-upscale the whole cell so pixels are honest at reading size."""
    g = glyph_img.width
    swatch = Image.new("RGBA", (cell, cell), bar_color + (255,))
    off = (cell - g)//2
    swatch.alpha_composite(glyph_img, (off, off))
    return swatch.resize((cell*upscale, cell*upscale), Image.NEAREST)

def build():
    os.makedirs(MICRO, exist_ok=True)
    states = ["calm", "working", "attention"]
    colors = {"calm": BLACK, "working": BLACK, "attention": CORAL}

    # --- write runtime PNGs + masters + svg (STEP 2) ---
    for st in states:
        col = colors[st]
        for scale, size in [(1, 18), (2, 36)]:
            img = render_glyph_rgba(st, size, col)
            img.save(os.path.join(MICRO, f"cat-micro-{st}@{scale}x.png"))
        # high-res master (template source render)
        master = render_glyph_rgba(st, 512, col)
        master.save(os.path.join(MASTERS, f"cat-micro-{st}.png"))
        with open(os.path.join(MASTERS, f"cat-micro-{st}.svg"), "w") as f:
            f.write(emit_svg(st, col))

    # --- build comparison preview (v2) ---
    cell = 30          # swatch px around the glyph
    up = 6             # nearest upscale factor
    pad = 14
    label_h = 26
    # columns: state x {18 light, 18 dark, 36 light, 36 dark, step1 18 light, step1 18 dark}
    cols = ["18px template\nlight bar", "18px template\ndark bar",
            "36px template\nlight bar", "36px template\ndark bar",
            "STEP1 raster18\nlight bar", "STEP1 raster18\ndark bar"]
    def cellpx(sz):
        return sz*up
    # variable-width columns (36px cells wider). compute widths.
    def col_glyph(state, kind):
        col = colors[state]
        if kind == "t18l":
            return on_bar(inv(render_glyph_rgba(state, 18, col), state, LIGHT_BAR, "light"), LIGHT_BAR, cell, up)
        if kind == "t18d":
            return on_bar(inv(render_glyph_rgba(state, 18, col), state, DARK_BAR, "dark"), DARK_BAR, cell, up)
        if kind == "t36l":
            return on_bar(inv(render_glyph_rgba(state, 36, col), state, LIGHT_BAR, "light"), LIGHT_BAR, 48, up)
        if kind == "t36d":
            return on_bar(inv(render_glyph_rgba(state, 36, col), state, DARK_BAR, "dark"), DARK_BAR, 48, up)
        if kind == "s18l":
            return on_bar(step1_enlarge(state, 18), LIGHT_BAR, cell, up)
        if kind == "s18d":
            return on_bar(step1_enlarge(state, 18), DARK_BAR, cell, up)

    kinds = ["t18l", "t18d", "t36l", "t36d", "s18l", "s18d"]
    # render all cells to know sizes
    grid = {}
    for st in states:
        for k in kinds:
            grid[(st, k)] = col_glyph(st, k)
    col_w = {k: max(grid[(st, k)].width for st in states) for k in kinds}
    row_h = {st: max(grid[(st, k)].height for k in kinds) for st in states}

    from PIL import ImageFont
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
        fontb = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 14)
    except Exception:
        font = ImageFont.load_default(); fontb = font

    left = 90
    header_h = 40
    total_w = left + sum(col_w[k] + pad for k in kinds) + pad
    total_h = header_h + sum(row_h[st] + pad for st in states) + pad + 40
    canvas = Image.new("RGB", (total_w, total_h), (250, 250, 251))
    dd = ImageDraw.Draw(canvas)

    x = left
    for k in kinds:
        dd.text((x, 6), cols[kinds.index(k)], font=font, fill=(60, 60, 66))
        x += col_w[k] + pad
    y = header_h
    for st in states:
        dd.text((8, y + row_h[st]//2 - 8), st, font=fontb, fill=(20, 20, 24))
        x = left
        for k in kinds:
            img = grid[(st, k)]
            canvas.paste(img, (x, y))
            x += col_w[k] + pad
        y += row_h[st] + pad
    dd.text((8, y + 4),
            "Template (calm/working black+alpha, attention coral) auto-inverts per macOS appearance -> legible on BOTH bars.  "
            "STEP1 enlarged raster is a FIXED tone: survives one bar, lost on the other (here lost on dark). A raster cannot invert -> template chosen.",
            font=font, fill=(90, 90, 96))
    out = os.path.join(QA, "tray-menubar-preview-v2.png")
    canvas.save(out)
    print("wrote", out, canvas.size)

def inv(glyph_img, state, bar_color, appearance):
    """Simulate macOS template rendering: template glyphs (calm/working, black)
    render as near-black on a light bar and white on a dark bar. The coral
    attention glyph is NOT a template -> same coral on both bars."""
    if state == "attention":
        return glyph_img
    # template: recolor by appearance
    alpha = glyph_img.split()[3]
    color = (28, 28, 30) if appearance == "light" else (245, 245, 247)
    out = Image.new("RGBA", glyph_img.size, color + (0,))
    solid = Image.new("RGBA", glyph_img.size, color + (255,))
    out = Image.composite(solid, out, alpha)
    out.putalpha(alpha)
    return out

if __name__ == "__main__":
    build()
