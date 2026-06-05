# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. CHRIS *guards the gate* of your actions (it
approves/denies what the coding agent wants to do), so the character is a
**black guardian dog with glowing eyes**. Each state = the **same dog, same
pose and size**; what changes is the **glowing eye color + an exaggerated
expression**.

## Character: all-black guardian dog 🖤🐕

A **solid jet-black dog / wolfdog** (German Shepherd–ish silhouette, pointed
ears) with **big glowing eyes**. **Semi-realistic stylized anime** — a middle
ground: detailed and noble, **not** chibi/over-cartoon, but **not** photoreal
either. Balanced proportions. Easy to swap the breed later (same prompt, change
the shape line).

## Visual style: semi-realistic stylized anime sprite (pixel art)

Clean pixel-art sprite with **balanced, slightly realistic proportions** (not
chibi, not photorealistic). Crisp pixels, clean outline, bold cel shading,
expressive but believable. Punchy and readable at small size.

## How the state is shown: state-colored GLOW (eyes + rim light + aura)

This is the important fix: changing only the eye color is **too subtle**. So the
state color must dominate the whole sprite. The **fur stays jet black**, but in
each frame the **eyes, a rim light around the whole body, and a soft glowing
aura** all take the **state color** — so the state is **obvious at a glance**,
even tiny. The expression also changes (clearly, but not cartoonishly).

> The app reinforces this with a matching colored glow + motion per state, so
> readability is guaranteed in-app regardless of how strong the art's glow is.

## The 5 states (glow color + expression)

| State      | Glow color (hex)   | Expression / pose (clear, not over-the-top)               |
|------------|--------------------|-----------------------------------------------------------|
| `idle`     | `#34cdd6` cyan     | calm but alert, neutral, ears up                          |
| `alert`    | `#ff9f43` orange   | startled, eyes wide, ears perked, a bold `!`             |
| `approved` | `#2ecc71` green    | happy, open smile / tongue out, ears up, tail up         |
| `denied`   | `#ff6b6b` coral    | stern/angry, frown with bared teeth, ears back           |
| `pr`       | `#539bf5` blue     | curious head-tilt, bright attentive eyes                 |

Fur: jet black `#0e0e12`. The **eyes + rim light + aura glow the state color**
(this is the main signal). Keep a subtle rim so the black shape stays visible on
dark wallpapers.

## Output format (the important part)

- **5 separate files**, transparent background (PNG with alpha).
- **Square canvas, 1:1.** Generate at 1024×1024, then **downscale to 256×256
  with nearest-neighbor** so the pixels stay crisp (don't let it get blurry).
- **Identical body pose, scale and camera** across all 5 frames — only the eye
  glow color and the expression change. They are swapped in place, so they must
  line up.
- **Crisp pixels, limited palette, clean outline.** No baked-in shadow under the
  dog and no background (the app adds its own drop-shadow; the window is
  transparent). Keep the subtle rim light so the black shape stays visible.
- **File names exactly** (drop them in `companiond/ui/sprites/dog/`):
  `idle.png`, `alert.png`, `approved.png`, `denied.png`, `pr.png`

## Paste-ready prompt (English works best in Leonardo)

> 16-bit pixel art sprite, semi-realistic stylized anime style, balanced
> proportions (not chibi, not photorealistic), fierce noble all-black guardian
> dog / wolfdog mascot, solid jet-black fur, pointed ears, big glowing eyes, a
> glowing colored rim light around the whole body and a soft matching aura,
> front three-quarter view, clean outline, crisp clean pixels, bold cel shading,
> centered, isolated on a transparent background, no background, no drop shadow,
> game sprite asset.
>
> Same black dog in 5 variants, identical body pose / size / position, fur stays
> jet black; in each variant the EYES, the RIM LIGHT around the body and the
> AURA all GLOW the state color, and the expression changes — make each state
> obvious at a glance:
> 1) idle — calm and alert, neutral face, ears up; glow CYAN #34cdd6;
> 2) alert — startled, eyes wide, ears perked, a bold exclamation mark; glow ORANGE #ff9f43;
> 3) approved — happy, open smile with tongue out, ears up, tail up; glow GREEN #2ecc71;
> 4) denied — stern and angry, frown with bared teeth, ears back; glow RED #ff6b6b;
> 5) pr — curious head tilt, bright attentive eyes; glow BLUE #539bf5.

**Negative prompt** (paste in the negative field):

> chibi, super deformed, overly cartoonish, photorealistic, realistic photo, 3d
> render, smooth gradients, blurry, antialiased, soft focus, brown fur, colored
> fur, white fur, extra limbs, deformed, text, watermark, jpeg artifacts,
> background, drop shadow, low contrast, dull colors, no glow

**Breed swap:** replace "all-black guardian dog / wolfdog, ... pointed ears" with
another all-black dog shape if you want (e.g. "all-black hound with long floppy
ears"). Keep "solid jet-black fur" + "big glowing eyes".

## Leonardo settings (how to drive it)

- **Model:** `Leonardo Anime XL` (expressive faces). To push toward pure pixel
  art, add the **"Pixel Art" Element** with the slider around 0.4–0.6, or use a
  pixel-art finetune / community model.
- **Preset Style:** `Pixel Art` if available, otherwise `Anime` / `Dynamic`.
- **Orientation / aspect ratio:** **Square (1:1)** — a sprite is square. Set
  dimensions to **1024×1024** (then downscale to 256 as above).
- **Transparency:** turn **ON** (transparent PNG / "Foreground" mode). If the
  model doesn't support it, generate on a flat magenta `#ff00ff` background and
  remove it in Leonardo's Canvas → Remove Background.
- **Alchemy:** OFF (keeps pixels crisp; Alchemy tends to smooth them).
- **PhotoReal / High Contrast:** OFF.
- **Guidance Scale:** 7–9 (higher = follows the prompt more).
- **Images per generation:** 4, so you can pick the best base.

### Keeping the 5 frames consistent (important)

AI won't naturally draw the exact same dog five times. Do this:

1. Generate the **idle** frame first. Pick the best one.
2. **Copy its Seed** and reuse that same seed for the other 4, changing only the
   eye glow color + expression words. Same seed + same prompt skeleton = much
   closer match.
3. Even better: use **Image Guidance → Character Reference** (or Image-to-Image
   at strength ~0.35) pointing at your chosen idle frame, so frames 2–5 keep the
   same dog and only the eyes/expression change.
4. Generate at 1024, then **downscale to 256 with nearest-neighbor** (any image
   editor, or a pixelator) to lock in clean pixels.

## Integration

The app is already wired for this: drop the 5 PNGs in
`companiond/ui/sprites/dog/` with the names above and pick the **Dog** character
in the app — it just works (it falls back to the blob placeholder until the
files are there).
