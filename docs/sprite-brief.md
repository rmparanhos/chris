# CHRIS — Sprite brief (for Leonardo)

The companion ("CHRIS") is a small mascot that floats on the desktop and reacts
to events with **5 states**. Each state = the **same character, same pose and
size**, only the **body color + facial expression** change. We need each state
as its own image so it drops straight into the app.

## Output format (the important part)

- **5 separate files per character**, transparent background (PNG with alpha).
- **Square canvas, 512×512 px**, character centered, with ~12% empty padding
  around it (so it never touches the edges).
- **Identical scale, position and camera** across all 5 frames — only color and
  expression differ. They will be swapped in place, so they must line up.
- **Flat colors, clean thick outline, cute/kawaii mascot** style. Front-facing.
- **No baked-in shadow and no background** (the app adds its own drop-shadow and
  the window is transparent).
- File names exactly:
  `blob-idle.png`, `blob-alert.png`, `blob-approved.png`, `blob-denied.png`, `blob-pr.png`
  (and the same set with the `cat-` prefix for the second character).
- Optional but nice: also deliver a single horizontal **sprite sheet**
  (5 frames in a row, 2560×512) for previewing.

## The 5 states (exact colors)

| State      | Body color (hex) | Face / expression                                  |
|------------|------------------|----------------------------------------------------|
| `idle`     | `#34cdd6` cyan   | calm, neutral mouth (straight line), relaxed eyes  |
| `alert`    | `#ff9f43` orange | surprised: small round "o" mouth + a bold `!` mark  |
| `approved` | `#2ecc71` green  | happy, big smile, cheerful eyes                    |
| `denied`   | `#ff6b6b` coral  | sad/disapproving, frown (mouth curved down)        |
| `pr`       | `#539bf5` blue   | pleased, friendly smile (a notification arrived)   |

Eyes/outline detail color: very dark teal `#08343a`.

## Paste-ready prompt (English works best in Leonardo.AI)

> Cute minimalist desktop mascot character, front view, simple rounded blob
> shape with two big friendly eyes, thick clean vector outline, flat colors,
> kawaii style, centered, isolated on a fully transparent background, no shadow,
> soft and rounded, app icon quality, high resolution.
>
> Generate the SAME character in 5 expression variants, keeping identical pose,
> size and position, changing only body color and face:
> 1) calm with a neutral straight mouth, body color cyan #34cdd6;
> 2) surprised with a small round open mouth and a bold exclamation mark, body color orange #ff9f43;
> 3) very happy with a big smile, body color green #2ecc71;
> 4) sad/disapproving with a downturned frown, body color coral #ff6b6b;
> 5) pleased with a friendly smile, body color blue #539bf5.

For the **second character (cat)**, reuse the same prompt but replace
"rounded blob shape" with "rounded cat with two triangular ears and small
whiskers", and keep everything else identical.

## Style references already in the app

The current placeholder art is built from these shapes, if Leonardo wants to
match the existing look: a soft ellipse body, two dark circular eyes, a thin
rounded mouth, and (for the cat) two triangular ears with pink inner ears, a
small triangular nose and three whiskers per side. Keep it friendly and very
simple — it is shown at ~150 px on screen, so fine detail is lost.
