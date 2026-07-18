import { readFile } from "node:fs/promises";

const css = await readFile(new URL("../ui/styles.css", import.meta.url), "utf8");
const designReference = await readFile(
  new URL("../docs/design/design-reference.md", import.meta.url),
  "utf8",
);
const requiredTokens = [
  "color-bg-panel",
  "color-border-hairline",
  "color-text-primary",
  "color-text-secondary",
  "color-text-mono",
  "color-text-disabled",
  "color-state-working",
  "color-state-idle",
  "color-state-ended",
];

for (const name of requiredTokens) {
  const sourcePattern = new RegExp("`--" + name + ": ([^`]+)`");
  const sourceMatch = designReference.match(sourcePattern);
  if (!sourceMatch) {
    throw new Error(`design-reference.md token is missing: --${name}`);
  }

  const value = sourceMatch[1];
  const declaration = `--${name}: ${value};`;
  if (!css.includes(declaration)) {
    throw new Error(`Token consumer is missing or changed: ${declaration}`);
  }
}

if (css.toUpperCase().includes("#F05A5D")) {
  throw new Error("The throwaway debug list must not use the Needs-You coral signal");
}

console.log("Design contract: debug surface is neutral and token-source aligned");
