# 0007 — Accent color derivation

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** theming

## Context

Starlight's default accent (hsl(224, 100%, 60%) dark / hsl(234, 90%, 60%) light) must not ship.
The brand anchor is the logo — a brushed-steel glyph whose dominant dark anchor averages
rgb(77, 78, 80) with hue ≈ 220–240° at very low saturation (0.11 on the darkest pixels; only
0.07 % of opaque pixels exceed 0.15 saturation).

## Decision

Keep the logo's steel hue family (~205–211°) and raise the saturation to produce a clearly
identifiable accent — a steel slate, not the default azure, and not a gray (links must stay
distinguishable from body text).

| Variable | Light mode | Dark mode |
| :--- | :--- | :--- |
| `--sl-color-accent-low` | hsl(210, 42%, 93%) | hsl(205, 35%, 16%) |
| `--sl-color-accent` | hsl(211, 45%, 40%) | hsl(204, 45%, 58%) |
| `--sl-color-accent-high` | hsl(211, 50%, 32%) | hsl(204, 55%, 82%) |

Measured WCAG 2.1 relative-luminance contrast (page background: white in light mode,
hsl(224, 10%, 10%) ≈ rgb(18, 19, 23) in dark mode):

| Pair | Ratio |
| :--- | ---: |
| light `accent` (link text) on white | **6.14:1** (AA ✓, AAA for large text) |
| light `accent-high` on white | **8.32:1** (AAA ✓) |
| dark `accent` on dark background | **6.39:1** (AA ✓) |
| dark `accent-high` (link text) on dark background | **12.24:1** (AAA ✓) |

`accent-low` is a surface tint (not text), so its low contrast against the page background is
intentional. The built-in dark/light theme toggle stays enabled; both themes get the same accent
hue with mode-appropriate lightness and saturation.
