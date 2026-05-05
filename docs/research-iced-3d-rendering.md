# 3D Rendering Research: Rust/Iced Natal Chart Redesign

**Date:** 2026-04-30
**Context:** v11.1 video review identified natal chart quality as a key improvement area

---

## Iced 0.14 Status

**Released:** ~December 2025, available on crates.io as `iced = "0.14.0"`

### Key New Features for This Project

| Feature | Impact |
|---|---|
| `pin` widget | Absolute (x,y) positioning — place planet labels over shader |
| `float` widget | Floating overlays with dynamic positioning — tooltips |
| `stack` improvements | `push_under` for cleaner shader-behind-UI layering |
| New `Pipeline` trait | Cleaner shader API, breaking change from 0.13 |
| wgpu 27.0 | Major GPU backend upgrade from 22.0 |
| Animation API | Built-in animation primitives for hover transitions |
| cosmic-text 0.15 | Better Unicode symbol rendering (zodiac glyphs) |
| Headless mode | Enables automated testing of shader widgets |

### Breaking Changes from 0.13

- `shader::Program` API changed — `Pipeline` trait replaces `Storage`
- `Widget::update` takes `Event` by reference
- `Primitive::prepare()` takes `&mut Self::Pipeline` instead of raw Storage

---

## Recommended Architecture (Priority Order)

### 1. Iced 0.14 Overlay Approach — EASY (recommended)

Overlay native Iced text widgets on top of the shader canvas:

```rust
stack![
    shader(natal_chart),
    pin(text("☉").font(ASTRO_FONT).size(16)).x(planet_x).y(planet_y),
    pin(tooltip(mouse_area(container("").width(24).height(24))
        .on_enter(Message::PlanetHovered(id)),
        text("Sun in Aries 15°32'"), tooltip::Position::Top,
    )).x(planet_x).y(planet_y),
]
```

**Solves:** Planet symbols, hover tooltips, click interactivity
**Requires:** Iced 0.14 upgrade, astrology font loaded via `iced::font::load()`

### 2. Vertex Buffer Pipeline for Finer Lines — MODERATE

Switch aspect lines from pure-fragment SDF to tessellated geometry:

- Use `lyon` crate to tessellate lines as thin triangle strips
- Sub-pixel width control (0.5px-3px, much finer than SDF)
- Keep zodiac wheel + galaxy as fragment SDF
- Iced `custom_shader` example provides template

**Solves:** Aspect lines too thick, better visual quality
**Requires:** Rewrite aspect line section of shader to vertex+fragment pipeline

### 3. MSDF Font Atlas — MODERATE-HARD (only if overlays insufficient)

For symbols that rotate/tilt with 3D perspective:

- Generate atlas with `msdf-atlas-gen` from astrology font
- Pass as wgpu texture via bind group in Pipeline
- Render as textured quads in shader

**Solves:** In-shader text that follows perspective transform
**Requires:** Texture binding setup, MSDF sampling shader code

### 4. 3D Model Assets — HARD (optional decorative)

- Load glTF models with `gltf` or `easy-gltf` crate
- Free zodiac models on Sketchfab, Free3D, IconScout
- Requires mini 3D renderer (projection, lighting) inside shader widget

**Solves:** Downloadable 3D zodiac decorations
**Requires:** Full vertex pipeline, projection math, material system

---

## Available Crates

| Crate | Purpose | Use For |
|---|---|---|
| `lyon` | 2D path tessellation | Aspect lines as geometry |
| `image` | Load PNG/JPEG | Texture atlas loading |
| `gltf` / `easy-gltf` | Load glTF 3D models | Optional 3D assets |
| `wgpu` (direct) | GPU API | Already using via Iced |

## Available Astrology Fonts

| Font | Contents | Format |
|---|---|---|
| Astronomicon Fonts | Complete astrology set (zodiac, planets, aspects, houses) | TTF |
| ASTROGADGET | Star signs + planets + aspects | TTF |
| Unicode range | U+2648-U+2653 (zodiac), U+2609/263D/2640/2642 (planets) | System fonts |

## What NOT To Do

- Don't integrate Bevy/rend3/three-d/kiss3d (incompatible architectures)
- Don't use glyphon directly inside shader (use Iced native text instead)
- Don't render text procedurally in WGSL (atlas or overlays are better)
- Don't try to share wgpu device between Iced and external engines

---

## Implementation Plan

1. ~~**Upgrade Iced 0.13 -> 0.14**~~ — **DONE** (v11.2, 2026-04-30). 19 breaking API changes across 13 files: Pipeline trait, wgpu 27, canvas Action, widget renames, application boot.
2. ~~**Add `stack![shader, pin(labels)]`** for planet symbols~~ — **DONE** (v11.3, Wave 3e). Unicode astrology glyphs (☉☽☿♀♂♃♄⛢♆♇☊☋⚷). Custom font not needed — Inter renders Unicode 0x2609 onward correctly.
3. ~~**Add `tooltip` + `mouse_area`** for hover interactivity~~ — **DONE** (v11.3, Wave 3f). Each pinned glyph wrapped in `tooltip()` showing planet+sign+degree.
4. ~~**Load Astronomicon font**~~ — **NOT NEEDED** — system Unicode coverage sufficient.
5. **Optionally: `lyon` tessellation** for finer aspect lines — **DEFERRED** — current 0.003 base width acceptable per video re-review.
6. **Optionally: MSDF atlas** if tilted text is needed — **DEFERRED** — pin overlay approach validated.
