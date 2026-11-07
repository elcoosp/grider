# 0002 — Logo format: keep high-resolution raster, no SVG tracing

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** branding, logo pipeline

## Context

The only brand asset in the repository is `docs/logo.png` — a **3D-rendered metallic glyph**
(1264 × 1264, RGBA, ~14 000 distinct colors, smooth specular gradients, transparent background).
There is no SVG original anywhere in the repository.

The pixelation rule (never scale a raster asset beyond its native resolution; never upscale a PNG
to fake a larger logo) is honored by construction: the header renders the logo at roughly 28 px
against a 512 px master — a 45× headroom margin. No asset is ever upscaled.

Tracing to SVG was evaluated and rejected: flat-color tracers (potrace is monochrome; vtracer
posterizes) destroy the smooth metallic gradients that *are* the logo — a traced SVG would replace
the brand asset with blobby flat shapes. Embedding the raster inside an SVG wrapper for the header
would add nothing (the pixels stay raster) and was not done for the header; the favicon uses a
viewBox-carrying SVG wrapper because favicons render at fixed tiny sizes where the embedded 64 px
raster is at or above native resolution.

## Decision

1. Keep a raster pipeline. Master is **downscaled only**: 1264 → 512 px (`logo-light.png`,
   `logo-dark.png` in `src/assets/`).
2. Two tone variants:
   - `logo-dark.png` — the original silver glyph, which reads perfectly on dark surfaces.
   - `logo-light.png` — a linear darkening (RGB × ≈0.52) so the silver stays visible on white.
3. `public/favicon.svg` wraps the real 64 px glyph in an SVG with `viewBox="0 0 64 64"`;
   `favicon.png` (64 px) and `apple-touch-icon.png` (180 px) ship alongside.
4. Starlight renders the light/dark variants via the `logo.light` / `logo.dark` config.

## Consequences

- The logo is verified sharp at 2× zoom in both themes during visual QA (see `docs/qa-report.md`).
- If the project later ships a vector original, replace the raster pipeline with the SVG and delete
  this exception.
