import { cp, mkdir, rm } from "node:fs/promises";

await rm(new URL("../dist", import.meta.url), { recursive: true, force: true });
await mkdir(new URL("../dist", import.meta.url), { recursive: true });
await Promise.all(
  ["index.html", "styles.css"].map((asset) =>
    cp(
      new URL(`../ui/${asset}`, import.meta.url),
      new URL(`../dist/${asset}`, import.meta.url),
    ),
  ),
);
