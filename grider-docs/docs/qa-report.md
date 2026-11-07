# Visual QA report

Automated pass: `pnpm qa` (Playwright) — every route × 2 viewports × 2 themes,
failing on console errors, page errors, HTTP ≥ 400, horizontal overflow, missing or
duplicate `<h1>`, main text < 200 chars, theme-application failure, or broken `<img>`.

Final result: **QA PASSED — 22 routes × 2 viewports × 2 themes, 0 failures**
(`.qa/failures.json` is empty). Zero-warning gate: `pnpm build`, `pnpm check` and
`pnpm lint` all clean (see `.qa/*.log`). Pagefind search probe
(`scripts/search-probe.mjs`): “Sauvola” → 16 results, “filter_biggest_rows” → 4 results.

Eyeball pass notes are per route below; every screenshot lives in `.qa/screenshots/`.

| Route | desktop-light | desktop-dark | mobile-light | mobile-dark | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| /404 | ✓ | ✓ | ✓ | ✓ | Custom 404 from docs/404.md; LinkCards render; excluded from per-route walk, screenshotted explicitly. |
| /explanation/extraction-pipeline | ✓ | ✓ | ✓ | ✓ | ASCII pipeline diagram in code block; stage headings. |
| /explanation/performance | ✓ | ✓ | ✓ | ✓ | Benchmark tables fit; memory table. |
| /guides/choose-binarization | ✓ | ✓ | ✓ | ✓ | Real Sauvola JSON sample; note aside styled. |
| /guides/configure-extraction | ✓ | ✓ | ✓ | ✓ | Knob table fits; clamping code highlighted. |
| /guides/draw-overlays | ✓ | ✓ | ✓ | ✓ | Real overlay figure; renderer semantics warning list. |
| /guides/filter-lines | ✓ | ✓ | ✓ | ✓ | Coordinate-lookup table fits; note aside on default tolerance. |
| /guides/install | ✓ | ✓ | ✓ | ✓ | Tabs (cargo add / Cargo.toml) render; feature table fits. |
| /guides/serialize-grids | ✓ | ✓ | ✓ | ✓ | Real JSON excerpt; caution aside for struct-literal break. |
| /guides/use-the-cli | ✓ | ✓ | ✓ | ✓ | Real transcripts incl. ansi error case. |
| / | ✓ | ✓ | ✓ | ✓ | Splash hero with real CLI overlay; logo sharp 2× both themes; steel accent on action buttons. |
| /reference/binarization | ✓ | ✓ | ✓ | ✓ | Constants table; memory table. |
| /reference/cli | ✓ | ✓ | ✓ | ✓ | Full flag table fits; exit-code table. |
| /reference/drawing | ✓ | ✓ | ✓ | ✓ | Defaults table; builder table. |
| /reference/errors | ✓ | ✓ | ✓ | ✓ | non_exhaustive badge renders on Variants heading (was raw text in title — fixed, see ADR 0009/qa notes). |
| /reference/feature-flags | ✓ | ✓ | ✓ | ✓ | Flags table; CLI-without-features table. |
| /reference/grid | ✓ | ✓ | ✓ | ✓ | Method tables fit; compatibility wrappers table. |
| /reference/grid-config | ✓ | ✓ | ✓ | ✓ | Field/constraint table fits both themes. |
| /reference/lines-and-cells | ✓ | ✓ | ✓ | ✓ | Field tables; LineTrait example highlighted. |
| /reference/macros | ✓ | ✓ | ✓ | ✓ | Heading bang (make_line!/make_grid!) allowed via MD026 config. |
| /reference/pipeline | ✓ | ✓ | ✓ | ✓ | Signatures with types; merge example. |
| /tutorials/extract-your-first-grid | ✓ | ✓ | ✓ | ✓ | Real input/overlay images; snapshot-verified 3×3 example; Steps render; pagination correct. |

## Eyeball checklist (final state)

- Logo sharp at 2× zoom in both themes (header crop inspected; no blur, halo or clipping).
- Accent is the steel-slate derived from the logo (ADR 0007) — visible on action buttons, links,
  active sidebar item and TOC; not Starlight's default blue; WCAG AA+ (see ADR 0007).
- Code blocks: Shiki tokens colored in both themes, copy button present, long lines scroll
  inside the block without page overflow; ansi terminal blocks render in color.
- Tables fit or wrap; no column cut off on desktop or mobile.
- Asides, CardGrid, Tabs, Steps, LinkCards and heading badges render with correct icons/spacing.
- Sidebar: correct groups, active state highlighted (`aria-current="page"`), no missing items.
- On-page TOC highlights the current heading; pagination links match sidebar order.
- Mobile: hamburger opens nav, content does not clip, tap targets usable.
- Dark mode: no white panels, code backgrounds distinct, no invisible text.
- Search modal returns sensible Pagefind results for domain terms.

## Defects found and fixed during QA

1. `:badge[...]` directive in the Errors page **title** rendered as raw text (badges only work
   in markdown headings) — moved to the `## Variants` heading; re-verified.
2. `locales: { en: … }` created a /en/-prefixed locale: sidebar hrefs pointed at missing pages,
   `isCurrent` never matched (no pagination, no active highlight) — fixed with the reserved
   `root` locale key; pagination and active states verified in the rebuilt HTML (ADR 0009).
3. Slug-based sidebar items threw during the /404 render — sidebar uses the `link` form (ADR 0009).
4. `/404` double-route conflict warning — `disable404Route: true` (ADR 0009).
5. QA script enumerated the home page with an empty slug and skipped `dist/404.html` — script
   fixed, full QA re-run after every fix above.

Every fix was followed by a full `pnpm build` → `pnpm preview` → `pnpm qa` re-run;
the final state of the repository passed QA as the last action before packaging.
