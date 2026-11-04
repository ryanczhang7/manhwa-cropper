# App icon

**Decided 2026-11-03 by the user**, who picked "Crop corners" out of three
sketches. It reverses `layout.md`'s v1 decision "Icon: none (the exe's
default)". Delivered by [MC-086](../../backlog/stories/MC-086.md) (window and
taskbar, at runtime) and [MC-087](../../backlog/stories/MC-087.md) (embedded
in the exe, for File Explorer and shortcuts).

## The artwork - source of truth

The SVG below is the artwork. Every raster of the icon is produced from it by
the build; nobody draws, exports or pastes a PNG or ICO by hand, and nobody
changes the artwork during GREEN. A change to the drawing is a design decision
and comes back here first.

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="2" y="2" width="60" height="60" rx="14" fill="#0067C0"/><rect x="22" y="14" width="20" height="36" rx="2" fill="#FFFFFF" opacity="0.35"/><path d="M14 22V14h8M50 22V14h-8M14 42v8h8M50 42v8h-8" fill="none" stroke="#FFFFFF" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/></svg>
```

| Part | Geometry (64-unit grid) | Fill |
|---|---|---|
| Tile | rounded square 2..62 on both axes, corner radius 14; transparent outside it | `primary` light, `#0067C0` (`tokens.md`) |
| Panel | 22..42 across, 14..50 down, radius 2 | white at 35 % opacity, i.e. about `#599CD6` over the tile |
| Crop corners | four L-shaped brackets, arms 8 units long, from (14,14), (50,14), (14,50), (50,50); stroke 4, round caps and joins | white `#FFFFFF`, opaque |

One icon for both themes. The tile is the light-theme `primary` in dark mode
too: an app icon sits on the taskbar and in Explorer, not inside the window,
and a blue tile with white marks reads on both a light and a dark taskbar.

## Rasters

| Size (px, square) | Where Windows uses it | Produced by |
|---|---|---|
| 16 | title bar, Explorer details view | MC-087 |
| 24 | taskbar at some scales, Explorer | MC-087 |
| 32 | taskbar, alt-tab at 100 % | MC-087 |
| 48 | Explorer medium icons | MC-087 |
| 256 | Explorer large and extra-large icons; also the runtime window icon | MC-086 |

- Each size is rendered from the SVG at that size, not shrunk from the 256.
- In the exe's icon resource every size is stored PNG-compressed (32-bit RGBA
  with alpha). Windows has read PNG entries at every size since Vista; this
  app targets Windows 11 only.
- The runtime icon handed to the window (`ViewportBuilder::with_icon`) is the
  256 raster, and MC-087 requires the exe's 256 entry to be the same pixels.
- The build renders the rasters (the SVG is checked in under `crates/app/`,
  and a build script rasterises it). Generated rasters live in the build's
  output directory, not in the repository.

## Sample points

The tests read these values out of the geometry above; they are not tuned.
At 256 px one unit is 4 px. Each sample sits well inside one part, away from
anti-aliased edges.

| Pixel (x, y) at 256 | Unit | Expected |
|---|---|---|
| (0,0), (255,0), (0,255), (255,255) | outside the tile's rounded corners | alpha 0 |
| (32,128), (128,32), (224,128), (128,224) | tile, clear of panel and brackets | `#0067C0`, alpha 255 |
| (56,72), (200,72), (56,184), (200,184) | the vertical arm of each bracket | `#FFFFFF`, alpha 255 |
| (56,128), (128,52) | the gap between brackets on the left edge (unit 14,32) and the top edge (unit 32,13, just above the panel) | `#0067C0`, alpha 255 |
| (128,128) | the panel | about (89,156,214), each channel within 2; alpha 255 |

The gap samples are what tell crop corners apart from a full white frame; the
panel sample is what tells this icon from one with the panel left out.

## What the icon is not

`voice.md` says "no icons standing in for words". That rule is about the
window's contents: no glyph replaces a label inside the app. The app icon is
the exe's identity in Windows and is not covered by it. The window itself
still shows no icons.
