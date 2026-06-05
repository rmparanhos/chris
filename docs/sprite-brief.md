# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. CHRIS *guards the gate* of your actions (it
approves/denies what the coding agent wants to do), so the character is a
**guard dog**. Each state = the **same dog, same pose and size**; what changes
is the **collar color + facial expression + ears/tail**.

## Character: German Shepherd guard dog 🦮

A noble **German Shepherd** — black-and-tan fur, erect pointed ears, alert and
loyal. It wears a **collar whose color encodes the state**. Semi-realistic
proportions (not super chibi) so it reads as a real guard dog, but still a
clean pixel-art sprite. Easy to swap the breed later — same prompt, change the
breed line (e.g. **bloodhound** with long floppy ears for an even sadder
"denied" face, or a **Labrador** for a friendlier look).

## Visual style: retro 16-bit anime sprite (pixel art)

SNES/JRPG sprite with an anime touch: crisp pixels, limited palette, clean dark
outline, simple cel shading (one light + one shadow tone), expressive eyes.
Noble and readable at small size. **Not** smooth vector, not 3D, not painterly.

## Why a collar instead of recoloring the dog

A dog that turns green/blue looks wrong. So the **fur stays the same** in every
frame and we change the **collar color** (plus the face, ears and tail) to
signal the state. The colors below go on the **collar**, not the fur.

## The 5 states (collar color + body language)

| State      | Collar color (hex) | Face + ears + tail                                          |
|------------|--------------------|-------------------------------------------------------------|
| `idle`     | `#34cdd6` cyan     | calm but alert, neutral mouth, ears up, tail neutral        |
| `alert`    | `#ff9f43` orange   | attentive/surprised, ears sharply perked, a bold `!` mark   |
| `approved` | `#2ecc71` green    | happy, open smile / tongue out, ears up, tail wagging       |
| `denied`   | `#ff6b6b` coral    | stern disapproval, ears back, frown, head lowered           |
| `pr`       | `#539bf5` blue     | curious head-tilt, friendly look (a notification arrived)   |

Outline / eye detail color: very dark teal `#08343a`. Fur: tan `#caa46a` with a
black saddle/back `#2b2b2b`.

## Output format (the important part)

- **5 separate files**, transparent background (PNG with alpha).
- **Square canvas, 1:1.** Generate at 1024×1024, then **downscale to 256×256
  with nearest-neighbor** so the pixels stay crisp (don't let it get blurry).
- **Identical pose, scale and camera** across all 5 frames — only the collar
  color, face, ears and tail change. They are swapped in place, so they must
  line up.
- **Crisp pixels, limited palette, dark outline.** No baked-in shadow under the
  dog and no background (the app adds its own drop-shadow; the window is
  transparent).
- **File names exactly** (drop them in `companiond/ui/sprites/dog/`):
  `idle.png`, `alert.png`, `approved.png`, `denied.png`, `pr.png`

## Paste-ready prompt (English works best in Leonardo)

> 16-bit pixel art sprite, retro JRPG / anime style, German Shepherd guard dog
> mascot, black and tan fur, erect pointed ears, noble alert posture, expressive
> eyes, wearing a colored collar, front three-quarter view, thick dark outline,
> limited color palette, crisp clean pixels, simple cel shading, centered,
> isolated on a transparent background, no background, no drop shadow, game
> sprite asset.
>
> Same dog in 5 variants, identical pose / size / position, fur unchanged, only
> the COLLAR color and the expression change:
> 1) calm but alert, neutral mouth, ears up, CYAN #34cdd6 collar;
> 2) attentive surprised, ears sharply perked, mouth slightly open, a bold exclamation mark, ORANGE #ff9f43 collar;
> 3) happy, open smile with tongue out, tail wagging, GREEN #2ecc71 collar;
> 4) stern disapproval, ears back, frowning, head lowered, CORAL #ff6b6b collar;
> 5) curious head tilt, friendly look, BLUE #539bf5 collar.

**Breed swap:** replace "German Shepherd, black and tan fur, erect pointed ears"
with "bloodhound, long floppy ears, wrinkled face" or "Labrador retriever, solid
yellow fur, floppy ears" to try other dogs with the same setup.

**Negative prompt** (paste in the negative field):

> blurry, smooth gradients, antialiased, 3d render, realistic photo, painterly,
> watercolor, soft focus, extra limbs, deformed, text, watermark, jpeg artifacts,
> background, drop shadow, low contrast, recolored fur

## Leonardo settings (how to drive it)

- **Model:** `Leonardo Anime XL` (nice expressive face). To push toward pure
  pixel art, add the **"Pixel Art" Element** with the slider around 0.4–0.6, or
  use a pixel-art finetune / community model.
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
   collar color + expression words. Same seed + same prompt skeleton = much
   closer match.
3. Even better: use **Image Guidance → Character Reference** (or Image-to-Image
   at strength ~0.35) pointing at your chosen idle frame, so frames 2–5 keep the
   same dog and only the collar/face change.
4. Generate at 1024, then **downscale to 256 with nearest-neighbor** (any image
   editor, or a pixelator) to lock in clean pixels.

## Integration

The app is already wired for this: drop the 5 PNGs in
`companiond/ui/sprites/dog/` with the names above and pick the **Dog** character
in the app — it just works (it falls back to the blob placeholder until the
files are there).
