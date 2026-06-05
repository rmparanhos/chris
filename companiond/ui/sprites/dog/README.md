# Dog sprite (PNG, one file per state)

Drop the 5 transparent PNGs here, named exactly:

- `idle.png`
- `alert.png`
- `approved.png`
- `denied.png`
- `pr.png`

Recommended: 256×256 (or 512×512), transparent background, pixel art kept crisp
(downscale with nearest-neighbor). See `docs/sprite-brief.md` for the full spec
and the Leonardo prompt.

Until these files exist, selecting the "Dog" character in the app falls back to
the blob placeholder automatically.
