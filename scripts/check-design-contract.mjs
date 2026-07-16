import { readFile } from "node:fs/promises";

const css = await readFile(new URL("../ui/styles.css", import.meta.url), "utf8");
const requiredTokens = new Map([
  ["bg-panel", "#1C1C1E"],
  ["border-hairline", "rgba(255,255,255,0.08)"],
  ["text-primary", "#ECECEC"],
  ["text-secondary", "#8A8A8E"],
  ["text-mono", "#A8A8AD"],
  ["text-disabled", "#5A5A5E"],
  ["state-working", "#7C7C82"],
  ["state-idle", "#5A5A5E"],
  ["state-ended", "#48484C"],
]);

for (const [name, value] of requiredTokens) {
  const declaration = `--${name}: ${value};`;
  if (!css.includes(declaration)) {
    throw new Error(`DESIGN.md token is missing or changed: ${declaration}`);
  }
}

if (css.toUpperCase().includes("#E39B3E")) {
  throw new Error("The throwaway debug list must not use Needs-You amber");
}

console.log("DESIGN.md contract: debug surface is neutral and token-aligned");
