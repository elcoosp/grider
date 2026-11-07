# 0004 — Visual QA: screenshot every route, both viewports, both themes

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** release process

## Context

A green build does not prove the site renders correctly. The release gate therefore includes an
automated browser pass over **every built route**, not a sample:

- Routes are enumerated by walking `dist/` for every `index.html` — nothing is hand-maintained.
- Each route is rendered at 1440 × 900 (desktop) and 390 × 844 (mobile), in light and dark themes
  (theme forced via `localStorage.starlight-theme`, verified against `documentElement.dataset.theme`).
- The pass fails on: console errors, uncaught page errors, HTTP ≥ 400 responses, horizontal
  overflow, missing or duplicate `<h1>`, main text shorter than 200 characters, theme-application
  failure, or broken `<img>` elements.
- Screenshots land in `.qa/` (git-ignored, excluded from the deliverable zip).
- After the automated pass, a human-style eyeball check inspects the screenshots (home page, one
  page per Diátaxis section, the longest page, the 404 page) and records the outcome per route in
  `docs/qa-report.md`. Logo sharpness is confirmed from a 2× zoom crop.

`@axe-core/playwright` (zero critical/serious violations per route) was considered as extra
hardening and is not added in this pass; the automated + eyeball checks above cover the release
blockers.

## Decision

`scripts/qa-screenshots.mjs` implements the gate and `pnpm qa` must exit `0` immediately before the
deliverable is packaged. Any config, CSS, logo, theme or content change invalidates the QA state
and forces a full re-run (`pnpm build` → `pnpm preview` → `pnpm qa`).

## Consequences

- `docs/qa-report.md` records one row per route (desktop-light, desktop-dark, mobile-light,
  mobile-dark, notes).
- Known visual defects block the release; fixes are re-verified with a fresh `pnpm qa` run.
