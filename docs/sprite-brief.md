# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. CHRIS *guards the gate* of your actions (it
approves/denies what the coding agent wants to do), so the character is a
**black guardian dog with glowing eyes**. Each state = the **same dog, same
pose and size**; what changes is the **glowing eye color + an exaggerated
expression**.

## Character: all-black guardian dog 🖤🐕

A **solid jet-black dog / wolfdog** (German Shepherd–ish silhouette, pointed
ears) with **big glowing eyes**. Stylized cartoon/anime — **not realistic** —
chunky, a little chibi, bold and expressive. A subtle **rim light / outline**
keeps the black silhouette visible on dark wallpapers. Easy to swap the breed
later (same prompt, change the shape line).

## Visual style: stylized 16-bit anime sprite (pixel art)

Bold cartoon/anime sprite: crisp pixels, very limited palette (mostly black +
the glowing eye color), clean outline, bold cel shading, **exaggerated** poses
and faces. **Not** realistic, not 3D, not painterly. Readable and punchy at
small size.

## How the state is shown: glowing eyes + exaggerated reaction

The **fur stays jet black** in every frame. What changes is the **eye glow
color** (below) and the expression, which should be **over-the-top** — big
surprise, huge grin, dramatic anger, etc. The app also adds motion per state
(shake/hop/wobble), so lean into exaggerated poses.

## The 5 states (eye glow color + exaggerated reaction)

| State      | Eye glow (hex)     | Exaggerated expression / pose                                  |
|------------|--------------------|----------------------------------------------------------------|
| `idle`     | `#34cdd6` cyan     | calm but alert, neutral, ears up, relaxed                      |
| `alert`    | `#ff9f43` orange   | SHOCKED: eyes huge and wide, ears shot straight up, big `!`     |
| `approved` | `#2ecc71` green    | OVERJOYED: huge grin, tongue out, tail wagging hard, sparkles   |
| `denied`   | `#ff6b6b` coral    | FURIOUS: deep frown, bared teeth/growl, ears flattened back     |
| `pr`       | `#539bf5` blue     | super curious dramatic head-tilt, big sparkling eyes           |

Fur: jet black `#0e0e12`. Subtle rim/outline: dark grey `#3a3a44` (so it reads on
dark backgrounds). The eyes are the main color and should **glow**.

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

> 16-bit pixel art sprite, stylized cartoon anime style, NOT realistic, chunky
> exaggerated proportions, cute but fierce all-black guardian dog / wolfdog
> mascot, solid jet-black fur, pointed ears, big glowing eyes, subtle rim light
> outline so it stays visible on dark backgrounds, front three-quarter view,
> thick outline, very limited color palette, crisp clean pixels, bold cel
> shading, centered, isolated on a transparent background, no background, no
> drop shadow, game sprite asset.
>
> Same black dog in 5 variants, identical body pose / size / position, fur stays
> jet black, only the GLOWING EYE color and the exaggerated expression change:
> 1) calm and alert, neutral face, ears up, eyes glowing CYAN #34cdd6;
> 2) shocked and startled, eyes huge and wide, ears shot straight up, a big bold exclamation mark, eyes glowing ORANGE #ff9f43;
> 3) overjoyed, huge open grin with tongue out, tail wagging hard, little sparkles, eyes glowing GREEN #2ecc71;
> 4) furious and stern, deep frown with bared teeth, ears flattened back, eyes glowing CORAL RED #ff6b6b;
> 5) super curious with a dramatic head tilt and big sparkling eyes, eyes glowing BLUE #539bf5.

**Negative prompt** (paste in the negative field):

> realistic, photorealistic, detailed realistic fur, 3d render, photo, smooth
> gradients, blurry, antialiased, soft focus, brown fur, colored fur, white fur,
> extra limbs, deformed, text, watermark, jpeg artifacts, background, drop
> shadow, low contrast

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
