# ProviderGlyph approved masters

> status: approved for WhoNeedsMe V1
> approved by: Kun
> approval date: 2026-07-17

Kun attests that permission has been obtained to use and redistribute the two
selected official marks in the public WhoNeedsMe repository and product. This
repository records that owner attestation; it does not embed private legal or
commercial correspondence.

## Selected assets

| Provider | Human-review choice | Master | First-party source | SHA-256 |
|---|---|---|---|---|
| Claude Code | A | `claude-code.svg` | Anthropic Claude Code for VS Code 2.1.199, `resources/claude-logo.svg` | `a3101f3047a119aa11825ad9369510f0c472428c8c52d420e31bc62db44a8364` |
| OpenAI Codex | A | `codex-dark.png` | OpenAI-signed ChatGPT/Codex App 26.715.21425, `Contents/Resources/icon-codex-dark-color.png` | `69fb4384e161be8a20dcb94a9ac34aea4fbfaeb67514110a71e7b0732eccb0fc` |

The source and public-license investigation is recorded in
`docs/research/provider-cli-icons.md`. The local 16px comparison evidence lives
under `artifacts/design-qa/provider-glyphs/`.

## Consumption contract

- Render each glyph in a `16px` CSS box with `object-fit: contain`; provide a
  `32px` raster for the Codex Retina path or let the runtime downsample the
  approved 1024px master.
- Keep the official geometry and colors unchanged. Do not add state badges,
  recolor the marks, or use either glyph as a Waiting / Needs You signal.
- Pair the glyph with visible provider text where the row has room and always
  expose the full provider name to accessibility APIs.
- These masters are Provider information only. WhoNeedsMe attention remains
  `--color-attention: #F05A5D` and is never inferred from provider color.
