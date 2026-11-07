# 0009 — Root locale config, sidebar link form and pagination

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** i18n configuration, sidebar, pagination

## Context

Three defects were caught during the visual QA pass and traced to two configuration pitfalls of the
current Starlight (0.42.x):

1. **Locale key naming.** `locales: { en: { label: 'English' } }` with `defaultLocale: 'en'` does
   **not** make English the unprefixed default in current Starlight — the key `en` creates a
   locale **prefixed at `/en/…`**. Instrumenting the build showed every sidebar entry receiving an
   `href` of `/en/guides/install/…` while the built routes live at `/guides/install/…`. Every
   sidebar link therefore pointed at a missing page (404 on click), `isCurrent` never matched (no
   active sidebar highlight), and `getPrevNextLinks` returned nothing (empty pagination on every
   page). The automated QA pass could not catch this — sidebar links are anchors, never fetched —
   and it surfaced through the manual eyeball pass on the screenshots (missing pagination).
   The unprefixed default locale is declared with the reserved **`root`** key:
   `locales: { root: { label: 'English', lang: 'en' } }`.
2. **Slug-based sidebar items + the 404 render.** With `404.md` present, the `/404` render path
   builds the sidebar from config and resolves `slug:` items through a route cache that is not
   populated in that context, throwing “The slug `"guides/install"` … does not exist”. Sidebar
   items that point at internal pages therefore use the explicit `link: '/…/'` form (with a
   `label`), which needs no route lookup. The only exception-free external URL would be the GitHub
   link, which lives in `social`, not the sidebar.
3. **Two routes for `/404`.** With `404.md` in the docs collection, Starlight injects a dedicated
   `/404` route **and** the `[...slug]` route wants to render the entry, producing the build
   warning “Could not render `/404` from route `/[...slug]` …”. `disable404Route: true` hands
   `/404` exclusively to `[...slug]`, which still renders `404.md` content with correct 404
   semantics for static hosts.

## Decision

```js
defaultLocale: 'en',
locales: { root: { label: 'English', lang: 'en' } },
disable404Route: true,
// sidebar: internal pages use { label, link: '/…/' } (link form), not slug form
```

## Consequences

- All internal URLs are unprefixed; sidebar links match the real routes.
- `aria-current="page"` marks the active sidebar item; prev/next pagination renders on all doc
  pages (verified in the built HTML after the fix).
- `pnpm build` runs with zero warnings; `docs/404.md` is served for unknown paths.
- If a second language is added later, revisit `locales` (the root key stays the unprefixed
  default) and re-run the full QA gate.
