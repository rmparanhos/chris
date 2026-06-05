# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. Each state = the **same character, same pose and
size**, only the **body color + facial expression** change. We need each state
as its own image so it drops straight into the app.

## Visual style: retro 16-bit anime sprite (pixel art)

Think **SNES/JRPG sprite with an anime face**: crisp pixels, a limited palette,
clean black/dark outline, simple cel shading (one light + one shadow tone), big
expressive anime eyes. Cute, readable at small size. **Not** smooth vector, not
3D, not painterly.

## Output format (the important part)

- **5 separate files per character**, transparent background (PNG with alpha).
- **Square canvas, 1:1.** Generate at 1024×1024, then **downscale to 256×256
  with nearest-neighbor** so the pixels stay crisp (don't let it get blurry).
- **Identical scale, position and camera** across all 5 frames — only color and
  expression differ. They are swapped in place, so they must line up.
- **Crisp pixels, limited palette, dark outline.** No baked-in shadow under the
  character and no background (the app adds its own drop-shadow; the window is
  transparent).
- File names exactly:
  `blob-idle.png`, `blob-alert.png`, `blob-approved.png`, `blob-denied.png`, `blob-pr.png`
  (and the same set with the `cat-` prefix for the second character).

## The 5 states (exact colors)

| State      | Body color (hex) | Face / expression                                   |
|------------|------------------|-----------------------------------------------------|
| `idle`     | `#34cdd6` cyan   | calm, neutral mouth, relaxed anime eyes             |
| `alert`    | `#ff9f43` orange | surprised: small round "o" mouth + a bold `!` mark   |
| `approved` | `#2ecc71` green  | happy, big smile, sparkly cheerful eyes             |
| `denied`   | `#ff6b6b` coral  | sad/annoyed, frown (mouth curved down)              |
| `pr`       | `#539bf5` blue   | pleased, friendly smile (a notification arrived)    |

Outline / eye detail color: very dark teal `#08343a`.

## Paste-ready prompt (English works best in Leonardo)

**Blob:**

> 16-bit pixel art sprite, retro JRPG / anime style, cute round blob mascot
> character, big expressive anime eyes, chibi, front view, thick dark outline,
> limited color palette, crisp clean pixels, simple cel shading, centered,
> isolated on a transparent background, no background, no drop shadow, game
> sprite sheet asset.
>
> Same character in 5 expression variants, identical pose / size / position,
> only body color and face change:
> 1) calm neutral straight mouth, body color cyan #34cdd6;
> 2) surprised small round open mouth with a bold exclamation mark, body color orange #ff9f43;
> 3) very happy big smile and sparkly eyes, body color green #2ecc71;
> 4) sad annoyed downturned frown, body color coral #ff6b6b;
> 5) pleased friendly smile, body color blue #539bf5.

**Cat** (same prompt, swap the shape line):

> ...replace "cute round blob mascot" with "cute chibi cat mascot with two
> triangular ears and small whiskers"...

**Negative prompt** (paste in the negative field):

> blurry, smooth gradients, antialiased, 3d render, realistic, photo, painterly,
> watercolor, soft focus, extra limbs, text, watermark, jpeg artifacts,
> background, drop shadow, low contrast

## Leonardo settings (how to drive it)

- **Model:** `Leonardo Anime XL` (best for the anime face). To push more toward
  pure pixel art, switch to a pixel-art finetune / community model, or add the
  **"Pixel Art" Element** with the slider around 0.4–0.6.
- **Preset Style:** `Pixel Art` if available, otherwise `Anime` / `Dynamic`.
- **Orientation / aspect ratio:** **Square (1:1)** — a sprite is square. Set
  dimensions to **1024×1024** (then downscale to 256 as above).
- **Transparency:** turn **ON** (transparent PNG / "Foreground" mode) so you get
  alpha straight away. If your model doesn't support it, generate on a flat
  magenta `#ff00ff` background and remove it in Leonardo's Canvas → Remove
  Background.
- **Alchemy:** OFF (keeps pixels crisp; Alchemy tends to smooth them).
- **PhotoReal / High Contrast:** OFF.
- **Guidance Scale:** 7–9 (higher = follows the prompt more).
- **Images per generation:** 4, so you can pick the best base.

### Keeping the 5 frames consistent (important)

AI won't naturally draw the exact same character five times. Do this:

1. Generate the **idle** frame first. Pick the best one.
2. **Copy its Seed** and reuse that same seed for the other 4, changing only the
   expression/color words. Same seed + same prompt skeleton = much closer match.
3. Even better: use **Image Guidance → Character Reference** (or Image-to-Image
   at strength ~0.35) pointing at your chosen idle frame, so frames 2–5 keep the
   same body/eyes and only the face/color change.
4. Generate at 1024, then **downscale to 256 with nearest-neighbor** (any image
   editor, or a pixelator) to lock in clean pixels.

## Integration

Once you have the 10 PNGs (5 blob + 5 cat), send them to me and I'll wire them
into the app (swap the current SVG for one image per state). The names above are
all I need for it to "just work".
