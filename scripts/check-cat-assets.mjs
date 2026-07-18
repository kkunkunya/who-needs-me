// Cat runtime asset family validator (raster; no SVG groups).
//
// Verifies the gpt-image-2-derived Cat asset family that Tray / Panel Header /
// Alert / Empty State consume:
//   - every required file exists (state-set completeness per detail tier);
//   - each PNG/WebP has a transparent alpha channel;
//   - the @1x width matches the tier's runtime size;
//   - the @2x variant is exactly double the @1x variant (density pairing).
// It deliberately does NOT check SVG layer groups — V1 assets are raster masters
// under the approved gpt-image-2 + human-gate exception (see DESIGN.md).
//
// Pure Node, no dependencies. Run: node scripts/check-cat-assets.mjs
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const CAT = join(ROOT, "assets", "cat");

// --- minimal PNG / WebP header readers ------------------------------------
function readPng(buf) {
  if (buf.length < 26 || buf.readUInt32BE(0) !== 0x89504e47) {
    throw new Error("not a PNG");
  }
  if (buf.toString("ascii", 12, 16) !== "IHDR") throw new Error("PNG: no IHDR");
  const width = buf.readUInt32BE(16);
  const height = buf.readUInt32BE(20);
  const colorType = buf[25];
  // color type 6 = truecolour+alpha, 4 = greyscale+alpha
  const hasAlpha = colorType === 6 || colorType === 4;
  return { width, height, hasAlpha, format: "png" };
}

function readWebp(buf) {
  if (buf.toString("ascii", 0, 4) !== "RIFF" || buf.toString("ascii", 8, 12) !== "WEBP") {
    throw new Error("not a WebP");
  }
  const fourcc = buf.toString("ascii", 12, 16);
  if (fourcc === "VP8X") {
    const flags = buf[20];
    const hasAlpha = (flags & 0x10) !== 0;
    const width = ((buf[24] | (buf[25] << 8) | (buf[26] << 16)) >>> 0) + 1;
    const height = ((buf[27] | (buf[28] << 8) | (buf[29] << 16)) >>> 0) + 1;
    return { width, height, hasAlpha, format: "webp" };
  }
  if (fourcc === "VP8L") {
    // lossless: 0x2f signature then 14b (width-1), 14b (height-1), 1b alpha_used
    if (buf[20] !== 0x2f) throw new Error("VP8L: bad signature");
    const b = buf.readUInt32LE(21);
    const width = (b & 0x3fff) + 1;
    const height = ((b >> 14) & 0x3fff) + 1;
    const hasAlpha = ((b >> 28) & 0x1) === 1;
    return { width, height, hasAlpha, format: "webp" };
  }
  // simple lossy VP8 has no alpha channel — not allowed for these assets
  return { width: 0, height: 0, hasAlpha: false, format: "webp" };
}

function readImage(path) {
  const buf = readFileSync(path);
  const head = buf.toString("ascii", 0, 4);
  if (head === "RIFF") return readWebp(buf);
  return readPng(buf);
}

// --- contract -------------------------------------------------------------
// tier -> { dir, ext, width@1x, square, states }
const TIERS = {
  micro: { ext: "png", width: 18, square: true, states: ["calm", "working", "attention"] },
  compact: { ext: "webp", width: 32, square: false, states: ["calm", "working", "attention"] },
  illustration: { ext: "webp", width: 200, square: false, states: ["sleeping"] },
};

const errors = [];
const checked = [];

function fail(msg) {
  errors.push(msg);
}

for (const [tier, spec] of Object.entries(TIERS)) {
  for (const state of spec.states) {
    const base = `cat-${tier}-${state}`;
    // full-resolution retoned master must exist (transparent PNG)
    const masterPath = join(CAT, "masters", `${base}.png`);
    if (!existsSync(masterPath)) {
      fail(`missing master: assets/cat/masters/${base}.png`);
    } else {
      const m = readImage(masterPath);
      if (!m.hasAlpha) fail(`master ${base}.png has no alpha channel`);
    }

    // runtime @1x / @2x
    let one = null;
    for (const scale of [1, 2]) {
      const rel = `assets/cat/${tier}/${base}@${scale}x.${spec.ext}`;
      const path = join(ROOT, rel);
      if (!existsSync(path)) {
        fail(`missing runtime asset: ${rel}`);
        continue;
      }
      const img = readImage(path);
      checked.push(rel);
      if (!img.hasAlpha) fail(`${rel} has no transparent alpha channel`);
      if (scale === 1) {
        one = img;
        if (img.width !== spec.width) {
          fail(`${rel} width ${img.width} != expected @1x width ${spec.width}`);
        }
        if (spec.square && img.width !== img.height) {
          fail(`${rel} must be square (${img.width}x${img.height})`);
        }
      } else if (one) {
        // density pairing: @2x is exactly double @1x in both dimensions
        if (img.width !== one.width * 2 || img.height !== one.height * 2) {
          fail(
            `${rel} (${img.width}x${img.height}) is not 2x the @1x (${one.width}x${one.height})`,
          );
        }
      }
    }
  }
}

// Alert reuses the compact attention character — assert that anchor exists.
if (!existsSync(join(CAT, "compact", "cat-compact-attention@1x.webp"))) {
  fail("Alert Card anchor missing: assets/cat/compact/cat-compact-attention@1x.webp");
}

if (errors.length > 0) {
  console.error(`Cat asset validator: ${errors.length} problem(s):`);
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}

console.log(
  `Cat asset family OK: ${checked.length} runtime files across micro/compact/illustration, ` +
    `transparent alpha, @2x density pairing, state sets complete (no SVG groups checked).`,
);
