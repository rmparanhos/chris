# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. Each state = the **same character, same pose and
size**; what changes is the **bandana color + facial expression + ears/tail**.
We need each state as its own image so it drops straight into the app.

## Character: chibi Shiba Inu puppy 🐕

A cute **Shiba Inu** (the "doge" dog) — cream/orange fur, curly tail, pointy
ears, big expressive anime eyes. It wears a small **bandana** whose **color
encodes the state**. Easy to swap the breed later if you want (corgi, golden
retriever puppy, pug) — same prompt, just change "Shiba Inu".

## Visual style: retro 16-bit anime sprite (pixel art)

SNES/JRPG sprite with an anime face: crisp pixels, limited palette, clean dark
outline, simple cel shading (one light + one shadow tone), big expressive eyes.
Cute and readable at small size. **Not** smooth vector, not 3D, not painterly.

## Why a bandana instead of recoloring the dog

A dog that turns green/blue looks wrong. So the **body fur stays the same** in
every frame and we change the **bandana color** (plus the face, ears and tail)
to signal the state. The colors below go on the **bandana**, not the fur.

## The 5 states (bandana color + body language)

| State      | Bandana color (hex) | Face + ears + tail                                         |
|------------|---------------------|------------------------------------------------------------|
| `idle`     | `#34cdd6` cyan      | calm, neutral mouth, ears up relaxed, tail neutral         |
| `alert`    | `#ff9f43` orange    | surprised: small "o" mouth, ears perked, a bold `!` mark   |
| `approved` | `#2ecc71` green     | happy, big open smile / tongue out, ears up, tail wagging  |
| `denied`   | `#ff6b6b` coral     | sad/guilty, frown, ears folded back, looking down          |
| `pr`       | `#539bf5` blue      | curious head-tilt, friendly smile (a notification arrived) |

Outline / eye detail color: very dark teal `#08343a`. Fur: warm cream + orange
(tan `#e0a45e` with a cream belly `#f3e0c0`).

## Output format (the important part)

- **5 separate files**, transparent background (PNG with alpha).
- **Square canvas, 1:1.** Generate at 1024×1024, then **downscale to 256×256
  with nearest-neighbor** so the pixels stay crisp (don't let it get blurry).
- **Identical pose, scale and camera** across all 5 frames — only the bandana
  color, face, ears and tail change. They are swapped in place, so they must
  line up.
- **Crisp pixels, limited palette, dark outline.** No baked-in shadow under the
  dog and no background (the app adds its own drop-shadow; the window is
  transparent).
- File names exactly:
  `dog-idle.png`, `dog-alert.png`, `dog-approved.png`, `dog-denied.png`, `dog-pr.png`

## Paste-ready prompt (English works best in Leonardo)

> 16-bit pixel art sprite, retro JRPG / anime style, cute chibi Shiba Inu puppy
> mascot, cream and orange fur, curly tail, pointy ears, big expressive anime
> eyes, wearing a small bandana, front view, thick dark outline, limited color
> palette, crisp clean pixels, simple cel shading, centered, isolated on a
> transparent background, no background, no drop shadow, game sprite asset.
>
> Same puppy in 5 variants, identical pose / size / position, fur color
> unchanged, only the bandana color and the expression change:
> 1) calm, neutral mouth, ears relaxed, CYAN #34cdd6 bandana;
> 2) surprised, small round open mouth, ears perked, a bold exclamation mark, ORANGE #ff9f43 bandana;
> 3) very happy, big open smile with tongue out, tail wagging, GREEN #2ecc71 bandana;
> 4) sad and guilty, frown, ears folded back, looking down, CORAL #ff6b6b bandana;
> 5) curious head tilt, friendly smile, BLUE #539bf5 bandana.

**Negative prompt** (paste in the negative field):

> blurry, smooth gradients, antialiased, 3d render, realistic, photo, painterly,
> watercolor, soft focus, extra limbs, deformed, text, watermark, jpeg artifacts,
> background, drop shadow, low contrast, recolored fur

## Leonardo settings (how to drive it)

- **Model:** `Leonardo Anime XL` (best for the anime face). To push more toward
  pure pixel art, add the **"Pixel Art" Element** with the slider around
  0.4–0.6, or use a pixel-art finetune / community model.
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

AI won't naturally draw the exact same dog five times. Do this:

1. Generate the **idle** frame first. Pick the best one.
2. **Copy its Seed** and reuse that same seed for the other 4, changing only the
   bandana color + expression words. Same seed + same prompt skeleton = much
   closer match.
3. Even better: use **Image Guidance → Character Reference** (or Image-to-Image
   at strength ~0.35) pointing at your chosen idle frame, so frames 2–5 keep the
   same dog and only the bandana/face change.
4. Generate at 1024, then **downscale to 256 with nearest-neighbor** (any image
   editor, or a pixelator) to lock in clean pixels.

## Integration

Once you have the 5 PNGs, send them to me and I'll wire them into the app (swap
the current SVG for one image per state). The names above are all I need for it
to "just work".
