#!/usr/bin/env python3
"""Post-process the gpt-image-2 raster masters into the Cat runtime asset family.

This is a documented, deterministic recipe — NOT a machine-local path resolver.
It reuses the produce-assets skill's proven `cutout_solid` for the magenta
knock-out, then applies the two steps the plain `process_asset.py` CLI cannot:

  1. cutout_solid  — knock out the solid magenta background, decontaminate edges
                     (no white halo / fringe). Provenance masters live in
                     artifacts/design-qa/cat-assets/generation/ (committed).
  2. retone_light  — the V1 panel surface is dark night-blue (#14223A, see
                     docs/design/design-reference.md §3), so a black cat would be
                     invisible. Map the black body -> near-white figure and the
                     white interior line-art -> dark detail (luminance inversion),
                     preserving alpha. The attention state keeps a small warm
                     coral accent (--color-attention #F05A5D); calm / working stay
                     neutral monochrome (design-reference.md §10.1).
  3. trim + resize — alpha-bbox trim, then LANCZOS resize to each surface's
                     runtime size at @1x / @2x.

Outputs (version-controlled, stable): assets/cat/{micro,compact,illustration}/
and full-resolution retoned masters in assets/cat/masters/.

Run (from repo root, with the produce-assets skill present):
    python3 scripts/build-cat-assets.py

Requires: Pillow, numpy, and the produce-assets skill at
.claude/skills/produce-assets/scripts/process_asset.py (single stable path).
"""
from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
GEN = ROOT / "artifacts" / "design-qa" / "cat-assets" / "generation"
ASSETS = ROOT / "assets" / "cat"

# Single stable path — the produce-assets skill inside this repo/worktree.
sys.path.insert(0, str(ROOT / ".claude/skills/produce-assets/scripts"))
from process_asset import cutout_solid  # noqa: E402

# --color-attention #F05A5D — the one Needs-You warm-coral signal (design-reference §3).
CORAL = (240.0, 90.0, 93.0)
RAMP_LO, RAMP_HI = 0.10, 0.98  # white line-art -> ~#22 dark; black body -> ~#EE light


def _accent_mask(source: Image.Image) -> np.ndarray:
    """Warm-accent mask, detected on the ORIGINAL master before knock-out.

    On the raw master the accent strokes are clean coral (~253,115,105) while the
    body is black, the line-art white and the background magenta — none of which
    read as warm — so detecting here avoids the decontamination artifacts that a
    knocked-out image would introduce (e.g. a stray warm speck on the belly).
    """
    rgb = np.asarray(source.convert("RGB")).astype(np.int32)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    warm = (r - g > 40) & (r - b > 40) & (r > 120) & (g < 200)  # magenta has r-b=0
    warm_img = Image.fromarray((warm * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(3))
    return np.asarray(warm_img) > 127


def retone_light(img: Image.Image, source: Image.Image | None = None) -> Image.Image:
    """Invert luminance so the black cat becomes a light figure for the dark panel.

    Black body -> near-white; white interior lines -> dark detail. Alpha is kept.
    When `source` (the original master) is given, its warm accent strokes are
    re-tinted to the coral token; calm / working pass source=None and stay neutral.
    """
    rgba = np.asarray(img.convert("RGBA")).astype(np.float64)
    rgb = rgba[..., :3]
    lum = 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]
    ramp = (RAMP_LO + ((255.0 - lum) / 255.0) * (RAMP_HI - RAMP_LO)) * 255.0
    if source is not None:
        warm = _accent_mask(source) & (rgba[..., 3] > 40)  # only on the opaque figure
    else:
        warm = np.zeros(lum.shape, dtype=bool)
    out = rgba.copy()
    for c in range(3):
        out[..., c] = np.where(warm, CORAL[c], ramp)
    return Image.fromarray(np.clip(out, 0, 255).round().astype(np.uint8))


def trim_to_alpha(img: Image.Image, pad_frac: float, square: bool) -> Image.Image:
    """Crop to the alpha bounding box, then re-pad (square for tray glyphs)."""
    rgba = np.asarray(img.convert("RGBA"))
    ys, xs = np.where(rgba[..., 3] > 16)
    if len(xs) == 0:
        return img
    cropped = img.crop((int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1))
    cw, ch = cropped.size
    # NOTE: use alpha_composite, not paste(mask=self) — the latter premultiplies
    # and discards RGB under partial alpha, which would wipe the coral accent.
    if square:
        side = max(cw, ch)
        pad = round(side * pad_frac)
        canvas = Image.new("RGBA", (side + 2 * pad, side + 2 * pad), (0, 0, 0, 0))
        canvas.alpha_composite(cropped, (pad + (side - cw) // 2, pad + (side - ch) // 2))
        return canvas
    px, py = round(cw * pad_frac), round(ch * pad_frac)
    canvas = Image.new("RGBA", (cw + 2 * px, ch + 2 * py), (0, 0, 0, 0))
    canvas.alpha_composite(cropped, (px, py))
    return canvas


def resize_wh(img: Image.Image, width: int, height: int) -> Image.Image:
    return img.convert("RGBA").resize((width, height), Image.LANCZOS)


def save(img: Image.Image, path: Path, fmt: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if fmt == "webp":
        img.save(path, format="WEBP", quality=92, method=6, exact=True)
    else:
        img.save(path, format="PNG")
    print(f"wrote {path.relative_to(ROOT)} ({img.width}x{img.height})")


# tier -> (width@1x, square, format, pad_frac)
TIERS = {
    "micro": (18, True, "png", 0.12),        # Tray glyph, full-colour PNG
    "compact": (32, False, "webp", 0.06),    # Panel Header / Alert character
    "illustration": (200, False, "webp", 0.06),  # Empty State
}
# (source master stem, tier, state) -- Alert reuses the compact attention character.
JOBS = [
    ("cat-micro-calm", "micro", "calm"),
    ("cat-micro-working", "micro", "working"),
    ("cat-micro-attention", "micro", "attention"),
    ("cat-compact-calm", "compact", "calm"),
    ("cat-compact-working", "compact", "working"),
    ("cat-compact-attention", "compact", "attention"),
    ("cat-illustration-sleeping", "illustration", "sleeping"),
]


def main() -> int:
    for stem, tier, state in JOBS:
        width, square, fmt, pad = TIERS[tier]
        src = Image.open(GEN / f"{stem}.png").convert("RGBA")
        retoned = retone_light(cutout_solid(src, tolerance=45),
                               source=src if state == "attention" else None)
        trimmed = trim_to_alpha(retoned, pad_frac=pad, square=square)
        name = f"cat-{tier}-{state}"
        # full-resolution retoned master (transparent PNG) for future re-export
        save(trimmed, ASSETS / "masters" / f"{name}.png", "png")
        # @1x sets the aspect; @2x is exactly double so density pairing is clean.
        h1 = max(1, round(trimmed.height * width / trimmed.width))
        for scale in (1, 2):
            save(resize_wh(trimmed, width * scale, h1 * scale),
                 ASSETS / tier / f"{name}@{scale}x.{fmt}", fmt)
    print("Cat asset family rebuilt.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
